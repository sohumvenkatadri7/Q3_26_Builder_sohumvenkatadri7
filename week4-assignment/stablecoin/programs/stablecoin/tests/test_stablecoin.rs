use {
    anchor_lang::{
        solana_program::{system_instruction, system_program},
        Id, InstructionData, ToAccountMetas,
    },
    anchor_spl::token_2022::{
        spl_token_2022::{
            extension::{
                confidential_transfer::{
                    ConfidentialTransferAccount, ConfidentialTransferMint,
                },
                default_account_state::DefaultAccountState,
                metadata_pointer::MetadataPointer,
                mint_close_authority::MintCloseAuthority,
                permanent_delegate::PermanentDelegate,
                transfer_fee::{TransferFeeAmount, TransferFeeConfig},
                BaseStateWithExtensions, BaseStateWithExtensionsMut, ExtensionType, StateWithExtensions,
                StateWithExtensionsMut,
            },
            instruction::{initialize_account3, mint_to},
            state::{Account as TokenAccount, AccountState, Mint},
        },
        Token2022,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn setup_svm() -> (LiteSVM, Pubkey, Keypair) {
    let program_id = stablecoin::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/stablecoin.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
    (svm, program_id, payer)
}

fn send_tx(svm: &mut LiteSVM, instructions: &[Instruction], payer: &Keypair, extra_signers: &[&Keypair]) {
    let mut signers = vec![payer];
    signers.extend_from_slice(extra_signers);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        instructions,
        Some(&payer.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &signers,
    )
    .unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "Transaction failed: {:?}", res.err());
}

#[test]
fn test_task_1_initialize_stablecoin_mint() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();

    let decimals = 6u8;
    let fee_basis_points = 250u16; // 2.5%
    let max_fee = 5_000_000u64;

    let instruction = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::InitializeStablecoin {
            decimals,
            fee_basis_points,
            max_fee,
        }
        .data(),
        stablecoin::accounts::InitializeStablecoin {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    send_tx(&mut svm, &[instruction], &payer, &[&mint]);

    // Verify state exclusively using StateWithExtensions
    let mint_account = svm.get_account(&mint.pubkey()).unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();

    assert_eq!(mint_state.base.decimals, decimals);
    assert_eq!(mint_state.base.mint_authority.unwrap(), mint_authority.pubkey());
    assert_eq!(mint_state.base.freeze_authority.unwrap(), freeze_authority.pubkey());

    // Verify TransferFeeConfig extension
    let fee_config = mint_state.get_extension::<TransferFeeConfig>().unwrap();
    let epoch_fee = fee_config.calculate_epoch_fee(0, 10_000).unwrap();
    assert_eq!(epoch_fee, 250); // 2.5% of 10_000 = 250

    // Verify DefaultAccountState (Frozen for KYC)
    let default_state = mint_state.get_extension::<DefaultAccountState>().unwrap();
    assert_eq!(u8::from(default_state.state), AccountState::Frozen as u8);

    // Verify MetadataPointer
    let meta_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
    assert_eq!(Option::<Pubkey>::from(meta_pointer.metadata_address).unwrap(), mint.pubkey());

    // Verify MintCloseAuthority
    let close_auth = mint_state.get_extension::<MintCloseAuthority>().unwrap();
    assert_eq!(Option::<Pubkey>::from(close_auth.close_authority).unwrap(), close_authority.pubkey());
}

#[test]
fn test_task_4_thaw_kyc_account() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();
    let user = Keypair::new();

    // 1. Initialize stablecoin mint (default state is Frozen)
    let init_mint_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::InitializeStablecoin {
            decimals: 6,
            fee_basis_points: 250,
            max_fee: 5_000_000,
        }
        .data(),
        stablecoin::accounts::InitializeStablecoin {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[init_mint_ix], &payer, &[&mint]);

    // 2. Create user token account
    let user_ta = Keypair::new();
    let ta_space = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferFeeAmount,
    ])
    .unwrap();
    let rent = svm.minimum_balance_for_rent_exemption(ta_space);

    let create_ta_ix = system_instruction::create_account(
        &payer.pubkey(),
        &user_ta.pubkey(),
        rent,
        ta_space as u64,
        &Token2022::id(),
    );
    let init_ta_ix = initialize_account3(
        &Token2022::id(),
        &user_ta.pubkey(),
        &mint.pubkey(),
        &user.pubkey(),
    )
    .unwrap();
    send_tx(&mut svm, &[create_ta_ix, init_ta_ix], &payer, &[&user_ta]);

    // 3. Verify that account is Frozen by default
    let ta_account = svm.get_account(&user_ta.pubkey()).unwrap();
    let ta_state = StateWithExtensions::<TokenAccount>::unpack(&ta_account.data).unwrap();
    assert_eq!(ta_state.base.state, AccountState::Frozen);

    // 4. Thaw the account via KYC Thaw instruction
    let thaw_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ThawKycAccount {}.data(),
        stablecoin::accounts::ThawKycAccount {
            account: user_ta.pubkey(),
            mint: mint.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[thaw_ix], &payer, &[&freeze_authority]);

    // 5. Verify account is now Initialized (Thawed)
    let thawed_account = svm.get_account(&user_ta.pubkey()).unwrap();
    let thawed_state = StateWithExtensions::<TokenAccount>::unpack(&thawed_account.data).unwrap();
    assert_eq!(thawed_state.base.state, AccountState::Initialized);
}

#[test]
fn test_task_2_3_transfer_with_fee() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();

    // 1. Initialize stablecoin mint with 2.5% fee
    let init_mint_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::InitializeStablecoin {
            decimals: 6,
            fee_basis_points: 250, // 2.5%
            max_fee: 5_000_000,
        }
        .data(),
        stablecoin::accounts::InitializeStablecoin {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[init_mint_ix], &payer, &[&mint]);

    // 2. Create Alice and Bob token accounts
    let ta_space = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferFeeAmount,
    ])
    .unwrap();
    let rent = svm.minimum_balance_for_rent_exemption(ta_space);

    let alice_ta = Keypair::new();
    let bob_ta = Keypair::new();

    let create_alice = system_instruction::create_account(
        &payer.pubkey(),
        &alice_ta.pubkey(),
        rent,
        ta_space as u64,
        &Token2022::id(),
    );
    let init_alice = initialize_account3(&Token2022::id(), &alice_ta.pubkey(), &mint.pubkey(), &alice.pubkey()).unwrap();

    let create_bob = system_instruction::create_account(
        &payer.pubkey(),
        &bob_ta.pubkey(),
        rent,
        ta_space as u64,
        &Token2022::id(),
    );
    let init_bob = initialize_account3(&Token2022::id(), &bob_ta.pubkey(), &mint.pubkey(), &bob.pubkey()).unwrap();

    send_tx(&mut svm, &[create_alice, init_alice, create_bob, init_bob], &payer, &[&alice_ta, &bob_ta]);

    // 3. Thaw both accounts
    let thaw_alice = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ThawKycAccount {}.data(),
        stablecoin::accounts::ThawKycAccount {
            account: alice_ta.pubkey(),
            mint: mint.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    let thaw_bob = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ThawKycAccount {}.data(),
        stablecoin::accounts::ThawKycAccount {
            account: bob_ta.pubkey(),
            mint: mint.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[thaw_alice, thaw_bob], &payer, &[&freeze_authority]);

    // 4. Mint 10,000 tokens to Alice
    let mint_to_alice = mint_to(
        &Token2022::id(),
        &mint.pubkey(),
        &alice_ta.pubkey(),
        &mint_authority.pubkey(),
        &[],
        10_000,
    )
    .unwrap();
    send_tx(&mut svm, &[mint_to_alice], &payer, &[&mint_authority]);

    // 5. Transfer 10,000 tokens with fee from Alice to Bob
    let transfer_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::TransferWithFee { amount: 10_000 }.data(),
        stablecoin::accounts::TransferWithFee {
            source: alice_ta.pubkey(),
            mint: mint.pubkey(),
            destination: bob_ta.pubkey(),
            authority: alice.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[transfer_ix], &payer, &[&alice]);

    // 6. Verify balances and withheld fee
    let alice_acct = svm.get_account(&alice_ta.pubkey()).unwrap();
    let alice_state = StateWithExtensions::<TokenAccount>::unpack(&alice_acct.data).unwrap();
    assert_eq!(alice_state.base.amount, 0);

    let bob_acct = svm.get_account(&bob_ta.pubkey()).unwrap();
    let bob_state = StateWithExtensions::<TokenAccount>::unpack(&bob_acct.data).unwrap();
    // 10_000 - 250 fee = 9_750 credited
    assert_eq!(bob_state.base.amount, 9_750);

    // Verify withheld fee on destination
    let fee_amount = bob_state.get_extension::<TransferFeeAmount>().unwrap();
    assert_eq!(u64::from(fee_amount.withheld_amount), 250);
}

#[test]
fn test_task_5_reissue_confidential_mint() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();
    let permanent_delegate = Keypair::new();
    let ct_authority = Keypair::new();

    let decimals = 6u8;
    let fee_basis_points = 100u16; // 1%
    let max_fee = 1_000_000u64;

    let instruction = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ReissueConfidentialMint {
            decimals,
            fee_basis_points,
            max_fee,
        }
        .data(),
        stablecoin::accounts::ReissueConfidentialMint {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            permanent_delegate: permanent_delegate.pubkey(),
            confidential_transfer_authority: ct_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    send_tx(&mut svm, &[instruction], &payer, &[&mint]);

    // Verify all stacked extensions via StateWithExtensions
    let mint_account = svm.get_account(&mint.pubkey()).unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();

    // Check PermanentDelegate
    let pd = mint_state.get_extension::<PermanentDelegate>().unwrap();
    assert_eq!(Option::<Pubkey>::from(pd.delegate).unwrap(), permanent_delegate.pubkey());

    // Check ConfidentialTransferMint (auto_approve_new_accounts = false / manual approval policy)
    let ct = mint_state.get_extension::<ConfidentialTransferMint>().unwrap();
    assert_eq!(Option::<Pubkey>::from(ct.authority).unwrap(), ct_authority.pubkey());
    assert!(!bool::from(ct.auto_approve_new_accounts));
}

#[test]
fn test_task_6_approve_confidential_account() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();
    let permanent_delegate = Keypair::new();
    let ct_authority = Keypair::new();
    let user = Keypair::new();

    // 1. Re-issue confidential mint with manual approval
    let init_mint_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ReissueConfidentialMint {
            decimals: 6,
            fee_basis_points: 100,
            max_fee: 1_000_000,
        }
        .data(),
        stablecoin::accounts::ReissueConfidentialMint {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            permanent_delegate: permanent_delegate.pubkey(),
            confidential_transfer_authority: ct_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[init_mint_ix], &payer, &[&mint]);

    // 2. Create user token account directly in memory with ConfidentialTransferAccount extension (approved = false)
    let user_ta = Keypair::new();
    let ta_space = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferFeeAmount,
        ExtensionType::ConfidentialTransferAccount,
    ])
    .unwrap();
    let rent = svm.minimum_balance_for_rent_exemption(ta_space);

    let mut data = vec![0u8; ta_space];
    let mut state = StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut data).unwrap();
    state.init_account_type().unwrap();
    state.base.mint = mint.pubkey();
    state.base.owner = user.pubkey();
    state.base.state = AccountState::Initialized;
    state.pack_base();
    
    let ct_ext = state.init_extension::<ConfidentialTransferAccount>(true).unwrap();
    ct_ext.approved = false.into();

    svm.set_account(
        user_ta.pubkey(),
        Account {
            lamports: rent,
            data,
            owner: Token2022::id(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // 3. Verify account is not approved yet
    let ta_account = svm.get_account(&user_ta.pubkey()).unwrap();
    let ta_state = StateWithExtensions::<TokenAccount>::unpack(&ta_account.data).unwrap();
    let ct_ext = ta_state.get_extension::<ConfidentialTransferAccount>().unwrap();
    assert!(!bool::from(ct_ext.approved));

    // 4. Approve the confidential account via ApproveConfidentialAccount instruction
    let approve_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::ApproveConfidentialAccount {}.data(),
        stablecoin::accounts::ApproveConfidentialAccount {
            account: user_ta.pubkey(),
            mint: mint.pubkey(),
            confidential_transfer_authority: ct_authority.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[approve_ix], &payer, &[&ct_authority]);

    // 5. Verify account is now approved
    let approved_account = svm.get_account(&user_ta.pubkey()).unwrap();
    let approved_state = StateWithExtensions::<TokenAccount>::unpack(&approved_account.data).unwrap();
    let approved_ct = approved_state.get_extension::<ConfidentialTransferAccount>().unwrap();
    assert!(bool::from(approved_ct.approved));
}

#[test]
fn test_close_mint() {
    let (mut svm, program_id, payer) = setup_svm();
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();
    let fee_authority = Keypair::new();
    let close_authority = Keypair::new();
    let refund_recipient = Keypair::new();

    // 1. Initialize mint
    let init_mint_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::InitializeStablecoin {
            decimals: 6,
            fee_basis_points: 250,
            max_fee: 5_000_000,
        }
        .data(),
        stablecoin::accounts::InitializeStablecoin {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            mint_authority: mint_authority.pubkey(),
            freeze_authority: freeze_authority.pubkey(),
            fee_authority: fee_authority.pubkey(),
            close_authority: close_authority.pubkey(),
            token_2022_program: Token2022::id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[init_mint_ix], &payer, &[&mint]);

    let mint_balance_before = svm.get_balance(&mint.pubkey()).unwrap();
    assert!(mint_balance_before > 0);

    // 2. Close mint using close_mint instruction
    let close_ix = Instruction::new_with_bytes(
        program_id,
        &stablecoin::instruction::CloseMint {}.data(),
        stablecoin::accounts::CloseMint {
            mint: mint.pubkey(),
            destination: refund_recipient.pubkey(),
            close_authority: close_authority.pubkey(),
            token_2022_program: Token2022::id(),
        }
        .to_account_metas(None),
    );
    send_tx(&mut svm, &[close_ix], &payer, &[&close_authority]);

    // 3. Verify mint is closed and refund destination received lamports
    let mint_acct = svm.get_account(&mint.pubkey());
    assert!(mint_acct.is_none() || mint_acct.unwrap().lamports == 0);
    assert_eq!(svm.get_balance(&refund_recipient.pubkey()).unwrap(), mint_balance_before);
}
