use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to_checked, Mint, MintToChecked, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority},
    ID as MPL_CORE_ID,
};

use crate::error::ErrorCode;
use crate::state::Config;

const SECONDS_PER_DAY: i64 = 86400;
const BURN_BONUS_MULTIPLIER: u64 = 5;

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidOwner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(
        mut,
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: PDA used for signing Metaplex Core CPI updates
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = rewards_mint,
        associated_token::authority = owner,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core Program ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BurnStakedNft>) -> Result<()> {
    // 1. Fetch and validate asset staking attributes
    let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_, attrs, _)| attrs);

    require!(attributes_fetched.is_some(), ErrorCode::AssetNotStaked);
    let attributes = attributes_fetched.unwrap();

    let current_timestamp = Clock::get()?.unix_timestamp;
    let mut staked_timestamp: i64 = 0;
    let mut is_staked = false;

    for attribute in &attributes.attribute_list {
        if attribute.key == "staked" && attribute.value == "true" {
            is_staked = true;
        } else if attribute.key == "staked_at" {
            staked_timestamp = attribute
                .value
                .parse::<i64>()
                .map_err(|_| ErrorCode::InvalidTimestamp)?;
        }
    }

    require!(is_staked, ErrorCode::AssetNotStaked);

    // 2. Calculate Staked Duration and Rewards + Burn Bonus
    let staked_duration_seconds = current_timestamp
        .checked_sub(staked_timestamp)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    // Total staked days (at least 1 day unit or pro-rated as needed)
    let staked_days = (staked_duration_seconds.checked_div(SECONDS_PER_DAY).unwrap_or(0)).max(1) as u64;

    // Base reward calculation
    let base_reward = staked_days
        .checked_mul(ctx.accounts.config.rewards_bps as u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_mul(10u64.pow(ctx.accounts.rewards_mint.decimals as u32))
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_div(10_000u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

    // Calculate Burn Bonus (e.g. 5x bonus reward on top)
    let total_burn_payout = base_reward
        .checked_mul(BURN_BONUS_MULTIPLIER)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

    // 3. Prepare Update Authority PDA Signer Seeds
    let collection_key = ctx.accounts.collection.key();
    let update_authority_seeds = &[
        b"update_authority",
        collection_key.as_ref(),
        &[ctx.bumps.update_authority],
    ];

    // 4. Thaw the asset (set FreezeDelegate frozen to false so it can be burned)
    UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
        .invoke_signed(&[update_authority_seeds])?;

    // 5. Burn the Metaplex Core Asset via CPI
    BurnV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.owner.to_account_info()) // Receives refunded rent lamports
        .authority(Some(&ctx.accounts.owner.to_account_info())) // Owner signs the burn
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke()?;

    // 6. Collection-level staking stats: Decrement "total_staked" on Collection Attributes
    let collection_attributes_fetched: Option<Attributes> = fetch_plugin::<BaseCollectionV1, Attributes>(
        &ctx.accounts.collection.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_, attrs, _)| attrs);

    if let Some(col_attrs) = collection_attributes_fetched {
        let mut collection_attributes_list: Vec<Attribute> = Vec::new();
        let mut new_total_staked: u64 = 0;

        for attr in col_attrs.attribute_list {
            if attr.key == "total_staked" {
                let current: u64 = attr.value.parse().unwrap_or(0);
                new_total_staked = current.saturating_sub(1);
            } else {
                collection_attributes_list.push(attr);
            }
        }
        collection_attributes_list.push(Attribute {
            key: "total_staked".to_string(),
            value: new_total_staked.to_string(),
        });

        UpdateCollectionPluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .collection(&ctx.accounts.collection.to_account_info())
            .payer(&ctx.accounts.owner.to_account_info())
            .authority(Some(&ctx.accounts.update_authority.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: collection_attributes_list,
            }))
            .invoke_signed(&[update_authority_seeds])?;
    }

    // 7. Mint Rewards + Bonus to User's ATA
    let config_seeds = &[
        b"config",
        collection_key.as_ref(),
        &[ctx.accounts.config.bump],
    ];

    mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.rewards_mint.to_account_info(),
                to: ctx.accounts.user_rewards_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            &[&config_seeds[..]],
        ),
        total_burn_payout,
        ctx.accounts.rewards_mint.decimals,
    )?;

    Ok(())
}