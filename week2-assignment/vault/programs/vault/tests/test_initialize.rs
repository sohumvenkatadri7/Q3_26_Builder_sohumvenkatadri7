use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    vault::{state::VaultState, STATE, VAULT_SEED},
};

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).unwrap_or_else(|err| {
        panic!("tx failed: {err:?}\nlogs: {logs:#?}", logs = err.meta.logs);
    });
}

#[test]
fn test_initialize() {
    let program_id = vault::id();
    let user = Keypair::new();
    let (vault_state, state_bump) =
        Pubkey::find_program_address(&[STATE, user.pubkey().as_ref()], &program_id);
    let (vault, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, user.pubkey().as_ref()], &program_id);

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/vault.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();

    // 1. Initialize instruction
    send(
        &mut svm,
        &user,
        Instruction::new_with_bytes(
            program_id,
            &vault::instruction::Initialize {}.data(),
            vault::accounts::Initialize {
                user: user.pubkey(),
                vault_state,
                vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 2. Assert vault_state was created with correct bumps
    let state_account = svm.get_account(&vault_state).expect("vault_state should exist after initialize");
    let mut state_data_slice = &state_account.data[..];
    let decoded_state = VaultState::try_deserialize(&mut state_data_slice).expect("Failed to deserialize VaultState");
    assert_eq!(decoded_state.vault_bump, vault_bump, "vault bump should match canonical bump");
    assert_eq!(decoded_state.state_bump, state_bump, "state bump should match canonical bump");

    // 3. Assert vault received rent-exemption balance
    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);
    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt,
        "vault should be rent-exempt after initialize"
    );
}