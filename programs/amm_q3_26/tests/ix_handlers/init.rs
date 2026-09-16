use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token::ID as ASSOCIATED_TOKEN_PROGRAM_ID,
    litesvm::LiteSVM,
    litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_initialise_ix(
    mut _svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    fee: u16,
    protocol_fee: u16,
    treasury: Pubkey,
) -> Instruction {
    let maker = payer.pubkey();

    Instruction::new_with_bytes(
        amm_q3_26::id(),
        &amm_q3_26::instruction::Initialize {
            seed: 123,
            fee,
            protocol_fee,
            treasury,
            authority: Some(maker),
        }
        .data(),
        amm_q3_26::accounts::Initialize {
            initializer: maker,
            mint_x,
            mint_y,
            mint_lp,
            vault_x,
            vault_y,
            config,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
