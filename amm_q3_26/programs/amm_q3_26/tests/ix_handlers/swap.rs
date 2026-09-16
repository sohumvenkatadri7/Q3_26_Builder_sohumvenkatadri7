use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    litesvm_token::{spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_swap_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    treasury: Pubkey,
) -> Instruction {
    create_custom_swap_ix(
        svm,
        payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
        treasury,
        true,
        10_000_000,
        5_000_000,
    )
}

pub fn create_custom_swap_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    treasury: Pubkey,
    is_x: bool,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    // Initialize treasury ATAs if they don't exist
    let _ = CreateAssociatedTokenAccount::new(svm, payer, &mint_x)
        .owner(&treasury)
        .send();
    let _ = CreateAssociatedTokenAccount::new(svm, payer, &mint_y)
        .owner(&treasury)
        .send();

    Instruction::new_with_bytes(
        amm_q3_26::id(),
        &amm_q3_26::instruction::Swap {
            is_x,
            amount_in,
            min_amount_out,
        }
        .data(),
        amm_q3_26::accounts::Swap {
            user,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            user_x,
            user_y,
            treasury_x,
            treasury_y,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
