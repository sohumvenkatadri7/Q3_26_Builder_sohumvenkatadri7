use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    vault::{STATE, VAULT_SEED},
};

const DEPOSIT_LAMPORTS: u64 = 500_000_000;

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).unwrap_or_else(|err| {
        panic!("tx failed: {err:?}\nlogs: {logs:#?}", logs = err.meta.logs);
    });
}

fn setup() -> (LiteSVM, Keypair, Pubkey, Pubkey, Pubkey) {
    let program_id = vault::id();
    let user = Keypair::new();
    let (vault_state, _) =
        Pubkey::find_program_address(&[STATE, user.pubkey().as_ref()], &program_id);
    let (vault, _) =
        Pubkey::find_program_address(&[VAULT_SEED, user.pubkey().as_ref()], &program_id);

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/vault.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();

    // 1. Initialize
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

    // 2. Deposit
    send(
        &mut svm,
        &user,
        Instruction::new_with_bytes(
            program_id,
            &vault::instruction::Deposit {
                amount: DEPOSIT_LAMPORTS,
            }
            .data(),
            vault::accounts::Deposit {
                user: user.pubkey(),
                vault_state,
                vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    (svm, user, vault_state, vault, program_id)
}

#[test]
fn test_close() {
    let (mut svm, user, vault_state, vault, program_id) = setup();

    let vault_balance_before = svm.get_balance(&vault).unwrap();
    let state_balance_before = svm.get_balance(&vault_state).unwrap();
    let user_balance_before = svm.get_balance(&user.pubkey()).unwrap();

    // 1. Close instruction
    send(
        &mut svm,
        &user,
        Instruction::new_with_bytes(
            program_id,
            &vault::instruction::Close {}.data(),
            vault::accounts::Close {
                user: user.pubkey(),
                vault_state,
                vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 2. Assert vault_state account is closed (data removed / does not exist)
    assert!(
        svm.get_account(&vault_state).is_none(),
        "vault_state should be completely closed and deleted"
    );

    // 3. Assert vault balance is 0
    assert_eq!(
        svm.get_balance(&vault).unwrap_or(0),
        0,
        "vault should be completely drained (balance = 0)"
    );

    // 4. Assert user received both vault balance and refunded state rent (minus 5000 tx fee)
    let user_balance_after = svm.get_balance(&user.pubkey()).unwrap();
    assert_eq!(
        user_balance_after,
        user_balance_before + vault_balance_before + state_balance_before - 5_000,
        "user balance should receive vault funds + state rent refund minus tx fee"
    );
}
