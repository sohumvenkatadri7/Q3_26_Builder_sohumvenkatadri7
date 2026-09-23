pub mod error;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_interface::{
    close_account, initialize_mint2, mint_close_authority_initialize, thaw_account,
    transfer_fee_initialize, CloseAccount, InitializeMint2, MintCloseAuthorityInitialize,
    ThawAccount, Token2022, TransferFeeInitialize,
    spl_token_2022::{
        extension::{
            confidential_transfer::instruction as ct_ix,
            confidential_transfer_fee::instruction as ctf_ix,
            default_account_state::instruction as das_ix,
            metadata_pointer::instruction as mp_ix,
            transfer_fee::{instruction as tf_ix, TransferFeeConfig},
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        },
        state::{AccountState, Mint},
    },
};

declare_id!("EnnHttbzyZoJGWDsguM2NJFLDgZUfqwAXAvF12aUGSLy");

#[program]
pub mod stablecoin {
    use super::*;

    pub fn initialize_stablecoin(
        ctx: Context<InitializeStablecoin>,
        decimals: u8,
        fee_basis_points: u16,
        max_fee: u64,
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let mint = &ctx.accounts.mint;
        let freeze_authority = &ctx.accounts.freeze_authority;
        let fee_authority = &ctx.accounts.fee_authority;
        let close_authority = &ctx.accounts.close_authority;
        let mint_authority = &ctx.accounts.mint_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;
        let system_program = &ctx.accounts.system_program;

        let extensions = [
            ExtensionType::TransferFeeConfig,
            ExtensionType::MetadataPointer,
            ExtensionType::DefaultAccountState,
            ExtensionType::MintCloseAuthority,
        ];

        let space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)
            .map_err(|_| error!(crate::error::StablecoinError::AccountLenCalculationFailed))?;

        let rent_lamports = Rent::get()?.minimum_balance(space);

        anchor_lang::system_program::create_account(
            CpiContext::new(
                system_program.key(),
                anchor_lang::system_program::CreateAccount {
                    from: payer.to_account_info(),
                    to: mint.to_account_info(),
                },
            ),
            rent_lamports,
            space as u64,
            &token_2022_program.key(),
        )?;

        transfer_fee_initialize(
            CpiContext::new(
                token_2022_program.key(),
                TransferFeeInitialize {
                    token_program_id: token_2022_program.to_account_info(),
                    mint: mint.to_account_info(),
                },
            ),
            Some(&fee_authority.key()),
            Some(&fee_authority.key()),
            fee_basis_points,
            max_fee,
        )?;

        invoke(
            &mp_ix::initialize(
                token_2022_program.key,
                mint.key,
                Some(*mint_authority.key),
                Some(*mint.key),
            )?,
            &[mint.to_account_info()],
        )?;

        invoke(
            &das_ix::initialize_default_account_state(
                token_2022_program.key,
                mint.key,
                &AccountState::Frozen,
            )?,
            &[mint.to_account_info()],
        )?;

        mint_close_authority_initialize(
            CpiContext::new(
                token_2022_program.key(),
                MintCloseAuthorityInitialize {
                    token_program_id: token_2022_program.to_account_info(),
                    mint: mint.to_account_info(),
                },
            ),
            Some(&close_authority.key()),
        )?;

        initialize_mint2(
            CpiContext::new(
                token_2022_program.key(),
                InitializeMint2 {
                    mint: mint.to_account_info(),
                },
            ),
            decimals,
            &mint_authority.key(),
            Some(&freeze_authority.key()),
        )?;

        Ok(())
    }

    pub fn transfer_with_fee(
        ctx: Context<TransferWithFee>,
        amount: u64,
    ) -> Result<()> {
        let source = &ctx.accounts.source;
        let mint = &ctx.accounts.mint;
        let destination = &ctx.accounts.destination;
        let authority = &ctx.accounts.authority;
        let token_2022_program = &ctx.accounts.token_2022_program;

        let mint_data = mint.data.borrow();
        let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data)
            .map_err(|_| error!(crate::error::StablecoinError::InvalidExtension))?;
        let fee_config = mint_state
            .get_extension::<TransferFeeConfig>()
            .map_err(|_| error!(crate::error::StablecoinError::InvalidExtension))?;

        let current_epoch = Clock::get()?.epoch;
        let fee = fee_config
            .calculate_epoch_fee(current_epoch, amount)
            .ok_or(error!(crate::error::StablecoinError::FeeCalculationFailed))?;

        let decimals = mint_state.base.decimals;

        invoke(
            &tf_ix::transfer_checked_with_fee(
                token_2022_program.key,
                source.key,
                mint.key,
                destination.key,
                authority.key,
                &[],
                amount,
                decimals,
                fee,
            )?,
            &[
                source.to_account_info(),
                mint.to_account_info(),
                destination.to_account_info(),
                authority.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn thaw_kyc_account(ctx: Context<ThawKycAccount>) -> Result<()> {
        let account = &ctx.accounts.account;
        let mint = &ctx.accounts.mint;
        let freeze_authority = &ctx.accounts.freeze_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;

        thaw_account(
            CpiContext::new(
                token_2022_program.key(),
                ThawAccount {
                    account: account.to_account_info(),
                    mint: mint.to_account_info(),
                    authority: freeze_authority.to_account_info(),
                },
            ),
        )?;

        Ok(())
    }

    pub fn reissue_confidential_mint(
        ctx: Context<ReissueConfidentialMint>,
        decimals: u8,
        fee_basis_points: u16,
        max_fee: u64,
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let mint = &ctx.accounts.mint;
        let mint_authority = &ctx.accounts.mint_authority;
        let freeze_authority = &ctx.accounts.freeze_authority;
        let fee_authority = &ctx.accounts.fee_authority;
        let close_authority = &ctx.accounts.close_authority;
        let permanent_delegate = &ctx.accounts.permanent_delegate;
        let ct_authority = &ctx.accounts.confidential_transfer_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;
        let system_program = &ctx.accounts.system_program;

        let extensions = [
            ExtensionType::TransferFeeConfig,
            ExtensionType::MetadataPointer,
            ExtensionType::DefaultAccountState,
            ExtensionType::MintCloseAuthority,
            ExtensionType::PermanentDelegate,
            ExtensionType::ConfidentialTransferMint,
            ExtensionType::ConfidentialTransferFeeConfig,
        ];

        let space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)
            .map_err(|_| error!(crate::error::StablecoinError::AccountLenCalculationFailed))?;

        let rent_lamports = Rent::get()?.minimum_balance(space);

        anchor_lang::system_program::create_account(
            CpiContext::new(
                system_program.key(),
                anchor_lang::system_program::CreateAccount {
                    from: payer.to_account_info(),
                    to: mint.to_account_info(),
                },
            ),
            rent_lamports,
            space as u64,
            &token_2022_program.key(),
        )?;

        transfer_fee_initialize(
            CpiContext::new(
                token_2022_program.key(),
                TransferFeeInitialize {
                    token_program_id: token_2022_program.to_account_info(),
                    mint: mint.to_account_info(),
                },
            ),
            Some(&fee_authority.key()),
            Some(&fee_authority.key()),
            fee_basis_points,
            max_fee,
        )?;

        invoke(
            &mp_ix::initialize(
                token_2022_program.key,
                mint.key,
                Some(*mint_authority.key),
                Some(*mint.key),
            )?,
            &[mint.to_account_info()],
        )?;

        invoke(
            &das_ix::initialize_default_account_state(
                token_2022_program.key,
                mint.key,
                &AccountState::Frozen,
            )?,
            &[mint.to_account_info()],
        )?;

        mint_close_authority_initialize(
            CpiContext::new(
                token_2022_program.key(),
                MintCloseAuthorityInitialize {
                    token_program_id: token_2022_program.to_account_info(),
                    mint: mint.to_account_info(),
                },
            ),
            Some(&close_authority.key()),
        )?;

        invoke(
            &anchor_spl::token_2022::spl_token_2022::instruction::initialize_permanent_delegate(
                token_2022_program.key,
                mint.key,
                permanent_delegate.key,
            )?,
            &[mint.to_account_info()],
        )?;

        invoke(
            &ct_ix::initialize_mint(
                token_2022_program.key,
                mint.key,
                Some(*ct_authority.key),
                false,
                None,
            )?,
            &[mint.to_account_info()],
        )?;

        invoke(
            &ctf_ix::initialize_confidential_transfer_fee_config(
                token_2022_program.key,
                mint.key,
                Some(*fee_authority.key),
                &[0u8; 32].into(),
            )?,
            &[mint.to_account_info()],
        )?;

        initialize_mint2(
            CpiContext::new(
                token_2022_program.key(),
                InitializeMint2 {
                    mint: mint.to_account_info(),
                },
            ),
            decimals,
            &mint_authority.key(),
            Some(&freeze_authority.key()),
        )?;

        Ok(())
    }

    pub fn approve_confidential_account(ctx: Context<ApproveConfidentialAccount>) -> Result<()> {
        let account = &ctx.accounts.account;
        let mint = &ctx.accounts.mint;
        let authority = &ctx.accounts.confidential_transfer_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;

        invoke(
            &ct_ix::approve_account(
                token_2022_program.key,
                account.key,
                mint.key,
                authority.key,
                &[],
            )?,
            &[
                account.to_account_info(),
                mint.to_account_info(),
                authority.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn close_mint(ctx: Context<CloseMint>) -> Result<()> {
        let mint = &ctx.accounts.mint;
        let destination = &ctx.accounts.destination;
        let close_authority = &ctx.accounts.close_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;

        close_account(
            CpiContext::new(
                token_2022_program.key(),
                CloseAccount {
                    account: mint.to_account_info(),
                    destination: destination.to_account_info(),
                    authority: close_authority.to_account_info(),
                },
            ),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeStablecoin<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Mint account to be created and initialized
    #[account(mut, signer)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: Mint authority
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    /// CHECK: Fee config authority
    pub fee_authority: UncheckedAccount<'info>,

    /// CHECK: Mint close authority
    pub close_authority: UncheckedAccount<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(mut)]
    /// CHECK: Source token account
    pub source: UncheckedAccount<'info>,

    /// CHECK: Mint account
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Destination token account
    pub destination: UncheckedAccount<'info>,

    pub authority: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct ThawKycAccount<'info> {
    #[account(mut)]
    /// CHECK: Token account to thaw
    pub account: UncheckedAccount<'info>,

    /// CHECK: Mint account
    pub mint: UncheckedAccount<'info>,

    pub freeze_authority: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct ReissueConfidentialMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Mint account to be created and initialized
    #[account(mut, signer)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: Mint authority
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    /// CHECK: Fee authority
    pub fee_authority: UncheckedAccount<'info>,

    /// CHECK: Mint close authority
    pub close_authority: UncheckedAccount<'info>,

    /// CHECK: Permanent delegate authority
    pub permanent_delegate: UncheckedAccount<'info>,

    /// CHECK: Confidential transfer authority
    pub confidential_transfer_authority: UncheckedAccount<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveConfidentialAccount<'info> {
    #[account(mut)]
    /// CHECK: Token account to approve
    pub account: UncheckedAccount<'info>,

    /// CHECK: Mint account
    pub mint: UncheckedAccount<'info>,

    pub confidential_transfer_authority: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct CloseMint<'info> {
    #[account(mut)]
    /// CHECK: Mint account to close
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Destination for refunded rent lamports
    pub destination: UncheckedAccount<'info>,

    pub close_authority: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}
