use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    litesvm_token::{spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, MintTo},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_deposit_ix(
    mut svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
) -> Instruction {
    let user = payer.pubkey();

    let user_x = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_x)
        .owner(&user)
        .send()
        .unwrap();
    MintTo::new(&mut svm, &payer, &mint_x, &user_x, 1000000000)
        .send()
        .unwrap();

    let user_y = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_y)
        .owner(&user)
        .send()
        .unwrap();
    MintTo::new(&mut svm, &payer, &mint_y, &user_y, 1000000000)
        .send()
        .unwrap();

    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);

    Instruction::new_with_bytes(
        amm_q3_26::id(),
        &amm_q3_26::instruction::Deposit {
            amount: 100_000_000,
            max_x: 200_000_000,
            max_y: 200_000_000,
        }
        .data(),
        amm_q3_26::accounts::Deposit {
            user,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            user_x,
            user_y,
            user_lp,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
