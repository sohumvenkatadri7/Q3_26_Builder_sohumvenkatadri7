use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token_2022::{
    spl_token_2022::{
        extension::{
            confidential_transfer::instruction as ct_ix,
            default_account_state::instruction as das_ix,
            metadata_pointer::instruction as mp_ix,
            transfer_fee::{instruction as tf_ix, TransferFeeConfig},
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        },
        instruction as token_ix,
        state::{AccountState, Mint},
    },
    Token2022,
};

declare_id!("EnnHttbzyZoJGWDsguM2NJFLDgZUfqwAXAvF12aUGSLy");

#[program]
pub mod stablecoin_practice {
    use super::*;

    // =========================================================================
    // TASK 1: Initialize Stablecoin Mint with Stacked Extensions
    // =========================================================================
    pub fn initialize_stablecoin(
        ctx: Context<InitializeStablecoin>,
        decimals: u8,
        fee_basis_points: u16,
        max_fee: u64,
    ) -> Result<()> {
        // HINT 1: Grab references to the accounts from ctx.accounts
        // (payer, mint, freeze_authority, fee_authority, close_authority, mint_authority, token_2022_program, system_program)
        let payer = &ctx.accounts.payer;
        let mint = &ctx.accounts.mint;
        let freeze_authority = &ctx.accounts.freeze_authority;
        let fee_authority = &ctx.accounts.fee_authority;
        let close_authority = &ctx.accounts.close_authority;
        let mint_authority = &ctx.accounts.mint_authority;
        let token_2022_program = &ctx.accounts.token_2022_program;
        let system_program = &ctx.accounts.system_program;

       

        // HINT 2: Define an array of the 4 extensions to stack:
        // [TransferFeeConfig, MetadataPointer, DefaultAccountState, MintCloseAuthority]
        let extensions: &[ExtensionType] = &[
            ExtensionType::TransferFeeConfig,
            ExtensionType::MetadataPointer,
            ExtensionType::DefaultAccountState,
            ExtensionType::MintCloseAuthority
        ];

        // HINT 3: Calculate the TLV space using ExtensionType::try_calculate_account_len::<Mint>(&extensions)
        let extension_space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)
            .map_err(|_| error!(crate::error::StablecoinError::AccountLenCalculationFailed))?;
        
        // HINT 4: Calculate rent with Rent::get()?.minimum_balance(space)
        let rent = Rent::get()?.minimum_balance(extension_space);
        // HINT 5: Invoke system_instruction::create_account
        invoke(
            &system_instruction::create_account(
                payer.key, 
                mint.key, 
                rent, 
                extension_space as u64, 
                token_2022_program.key,
            ),
            &[
                payer.to_account_info(),
                mint.to_account_info(),
                system_program.to_account_info()
                ],

        )?;
            
        // HINT 6: Invoke tf_ix::initialize_transfer_fee_config
        invoke(
            &tf_ix::initialize_transfer_fee_config(
                token_2022_program.key, 
                mint.key, 
                Some(fee_authority.key), 
                Some(fee_authority.key), 
                fee_basis_points, 
                max_fee,
            )?, 
            &[mint.to_account_info()],
        )?;

        // HINT 7: Invoke mp_ix::initialize (pointing to mint itself)
        invoke(
            &mp_ix::initialize(
                token_2022_program.key,
                mint.key,
                Some(*mint_authority.key),
                Some(*mint.key),
            )?,
            &[mint.to_account_info()],
        )?;
        // HINT 8: Invoke das_ix::initialize_default_account_state (&AccountState::Frozen)
        invoke(
            &das_ix::initialize_default_account_state(
                token_2022_program.key,
                mint.key,
                &AccountState::Frozen,
            )?,
            &[mint.to_account_info()],
        )?;
        // HINT 9: Invoke token_ix::initialize_mint_close_authority
        invoke(
            &token_ix::initialize_mint_close_authority(
                token_2022_program.key,
                mint.key,
                Some(close_authority.key),
            )?,
            &[mint.to_account_info()],
        )?;
        // HINT 10: Invoke token_ix::initialize_mint2 (MUST BE LAST!)

        Ok(())
    }

    // =========================================================================
    // TASK 2 & 3: Dynamic Fee Transfer & Safe State Reading
    // =========================================================================
    pub fn transfer_with_fee(
        ctx: Context<TransferWithFee>,
        amount: u64,
    ) -> Result<()> {
        // HINT 1: Borrow mint data: mint.data.borrow()

        // HINT 2: Unpack using StateWithExtensions::<Mint>::unpack(&mint_data)

        // HINT 3: Get the extension: mint_state.get_extension::<TransferFeeConfig>()

        // HINT 4: Get current epoch: Clock::get()?.epoch

        // HINT 5: Calculate fee: fee_config.calculate_epoch_fee(current_epoch, amount)

        // HINT 6: Invoke tf_ix::transfer_checked_with_fee CPI

        Ok(())
    }

    // =========================================================================
    // TASK 4: KYC Thaw Individual Account
    // =========================================================================
    pub fn thaw_kyc_account(ctx: Context<ThawKycAccount>) -> Result<()> {
        // HINT 1: Invoke token_ix::thaw_account CPI signed by freeze_authority

        Ok(())
    }

    // =========================================================================
    // TASK 5: Re-issue Mint with PermanentDelegate & Confidential Transfers
    // =========================================================================
    pub fn reissue_confidential_mint(
        ctx: Context<ReissueConfidentialMint>,
        decimals: u8,
        fee_basis_points: u16,
        max_fee: u64,
    ) -> Result<()> {
        // HINT 1: Define array of 6 extensions (4 previous + PermanentDelegate + ConfidentialTransferMint)

        // HINT 2: Sizing via ExtensionType::try_calculate_account_len::<Mint>

        // HINT 3: Create account CPI

        // HINT 4: Initialize all 6 extensions in order

        // HINT 5: For ConfidentialTransferMint: auto_approve_new_accounts = false (manual policy)

        // HINT 6: InitializeMint2 (LAST)

        Ok(())
    }
}

// =============================================================================
// Account Contexts
// =============================================================================

#[derive(Accounts)]
pub struct InitializeStablecoin<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: Mint authority
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    /// CHECK: Fee config & withdraw authority
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
    /// CHECK: Individual token account
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

    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: Mint authority
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    /// CHECK: Fee authority
    pub fee_authority: UncheckedAccount<'info>,

    /// CHECK: Mint close authority
    pub close_authority: UncheckedAccount<'info>,

    /// CHECK: Permanent delegate (seizure authority)
    pub permanent_delegate: UncheckedAccount<'info>,

    /// CHECK: Confidential transfer authority
    pub confidential_transfer_authority: UncheckedAccount<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
