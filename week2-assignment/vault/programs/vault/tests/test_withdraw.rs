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
const WITHDRAW_LAMPORTS: u64 = 200_000_000;

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).unwrap_or_else(|err| {
        panic!("tx failed: {err:?}\nlogs: {logs:#?}", logs = err.meta.logs);
    });
}

fn setup() -> (LiteSVM, Keypair, Pubkey, Pubkey, Pubkey, u64) {
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

    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);
    (svm, user, vault_state, vault, program_id, rent_exempt)
}

#[test]
fn test_withdraw() {
    let (mut svm, user, vault_state, vault, program_id, rent_exempt) = setup();

    let user_balance_before = svm.get_balance(&user.pubkey()).unwrap();

    // 1. Withdraw instruction
    send(
        &mut svm,
        &user,
        Instruction::new_with_bytes(
            program_id,
            &vault::instruction::Withdraw {
                amount: WITHDRAW_LAMPORTS,
            }
            .data(),
            vault::accounts::Withdraw {
                user: user.pubkey(),
                vault_state,
                vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 2. Assert vault balance decreased
    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt + DEPOSIT_LAMPORTS - WITHDRAW_LAMPORTS,
        "vault should decrease by the withdraw amount"
    );

    // 3. Assert user balance increased by withdraw amount (minus 5000 lamports tx fee)
    let user_balance_after = svm.get_balance(&user.pubkey()).unwrap();
    assert_eq!(
        user_balance_after,
        user_balance_before + WITHDRAW_LAMPORTS - 5_000,
        "user balance should increase by the withdraw amount minus tx fee"
    );
}

#[test]
fn test_withdraw_zero_fails() {
    let (mut svm, user, vault_state, vault, program_id, _) = setup();

    let blockhash = svm.latest_blockhash();
    let ix = Instruction::new_with_bytes(
        program_id,
        &vault::instruction::Withdraw { amount: 0 }.data(),
        vault::accounts::Withdraw {
            user: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let msg = Message::new_with_blockhash(&[ix], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();

    // Withdrawing 0 should fail with InvalidAmount
    assert!(svm.send_transaction(tx).is_err(), "withdraw with 0 amount should fail");
}
