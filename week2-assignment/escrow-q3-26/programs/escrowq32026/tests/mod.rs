use {
    anchor_lang::{
        prelude::msg, solana_program::instruction::Instruction, solana_program::program_pack::Pack,
        system_program::ID as SYSTEM_PROGRAM_ID, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
        token::spl_token,
    },
    litesvm::LiteSVM,
    litesvm_token::{
        spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
    },
    solana_clock::Clock,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

// Setup function to initialize LiteSVM and deploy the escrow program
fn setup() -> (LiteSVM, Keypair) {
    let program_id = escrowq32026::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/escrowq32026.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_make_and_refund() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    // 1. Create Mint A and Mint B
    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}", mint_a);

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}", mint_b);

    // 2. Create maker ATA for Mint A and mint 1,000 tokens
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}", maker_ata_a);

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    // 3. Derive PDA addresses
    let seed = 123u64;
    let (escrow, _) = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrowq32026::id(),
    );
    msg!("Escrow PDA: {}", escrow);

    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    msg!("Vault PDA: {}\n", vault);

    // 4. Send "Make" instruction (deposit 10 tokens, require 10 token B, expires at 17780206209)
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 10_000_000,
            expiration: 17780206209,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Make Transaction Successful ---");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    // Verify and print vault and escrow account state after Make
    let vault_account = program.get_account(&vault).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
    msg!("Vault Balance: {} (Mint: {})", vault_data.amount, vault_data.mint);
    assert_eq!(vault_data.amount, 10_000_000);
    assert_eq!(vault_data.owner, escrow);

    let escrow_account = program.get_account(&escrow).unwrap();
    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    msg!("Escrow State -> Seed: {}, Maker: {}, Receive: {}, Expiration: {}, Bump: {}",
        escrow_data.seed, escrow_data.maker, escrow_data.receive, escrow_data.expiration, escrow_data.bump
    );
    assert_eq!(escrow_data.seed, seed);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.receive, 10_000_000);

    // 5. Advance clock past the expiration time to allow Refund
    let mut clock: Clock = program.get_sysvar();
    clock.unix_timestamp = 17780206210;
    program.set_sysvar(&clock);
    msg!("\nClock advanced to: {}", clock.unix_timestamp);

    // 6. Send "Refund" instruction
    let refund_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Refund {
            maker,
            mint_a,
            maker_ata_a,
            escrow,
            vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Refund {}.data(),
    };

    let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Refund Transaction Successful ---");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    // Verify escrow and vault accounts are closed
    assert!(program.get_account(&escrow).is_none());
    assert!(program.get_account(&vault).is_none());
    msg!("Escrow Account Closed: {}", program.get_account(&escrow).is_none());
    msg!("Vault Account Closed: {}", program.get_account(&vault).is_none());

    // Verify Maker received their 1,000 tokens back in full
    let maker_account = program.get_account(&maker_ata_a).unwrap();
    let maker_data = spl_token::state::Account::unpack(&maker_account.data).unwrap();
    msg!("Maker ATA A Balance Restored: {}", maker_data.amount);
    assert_eq!(maker_data.amount, 1000_000_000);
}

#[test]
fn test_make_and_take() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    // Create a separate taker keypair and fund it with SOL
    let taker = Keypair::new();
    program.airdrop(&taker.pubkey(), 1_000_000_000).unwrap();

    // 1. Create Mint A and Mint B
    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}", mint_a);

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}", mint_b);

    // 2. Maker mints 1,000 tokens of Mint A
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}", maker_ata_a);

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    // 3. Taker creates ATA for Mint B and receives 1,000 tokens of Mint B
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    msg!("Taker ATA B: {}", taker_ata_b);

    MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000_000_000)
        .send()
        .unwrap();

    // 4. Derive PDAs and ATAs
    let seed = 456u64;
    let (escrow, _) = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrowq32026::id(),
    );
    msg!("Escrow PDA: {}", escrow);

    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    msg!("Vault PDA: {}", vault);

    let taker_ata_a = associated_token::get_associated_token_address(&taker.pubkey(), &mint_a);
    let maker_ata_b = associated_token::get_associated_token_address(&maker, &mint_b);

    // 5. Maker creates Escrow (deposit 10 Token A, receive 10 Token B, expires at 10_000)
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 10_000_000,
            expiration: 10_000,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let make_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Make Transaction Successful ---");
    msg!("CUs Consumed: {}", make_tx.compute_units_consumed);
    msg!("Tx Signature: {}", make_tx.signature);

    // 6. Taker executes "Take" instruction before expiration (clock is at 0)
    let take_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Take {
            taker: taker.pubkey(),
            maker,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Take {}.data(),
    };

    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&taker], message, recent_blockhash);
    let take_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Take Transaction Successful ---");
    msg!("CUs Consumed: {}", take_tx.compute_units_consumed);
    msg!("Tx Signature: {}", take_tx.signature);

    // 7. Assert and log swap results:
    assert!(program.get_account(&escrow).is_none());
    assert!(program.get_account(&vault).is_none());
    msg!("Escrow Account Closed: {}", program.get_account(&escrow).is_none());
    msg!("Vault Account Closed: {}", program.get_account(&vault).is_none());

    let taker_a_account = program.get_account(&taker_ata_a).unwrap();
    let taker_a_data = spl_token::state::Account::unpack(&taker_a_account.data).unwrap();
    msg!("Taker ATA A Balance (Received Token A): {}", taker_a_data.amount);
    assert_eq!(taker_a_data.amount, 10_000_000);

    let maker_b_account = program.get_account(&maker_ata_b).unwrap();
    let maker_b_data = spl_token::state::Account::unpack(&maker_b_account.data).unwrap();
    msg!("Maker ATA B Balance (Received Token B): {}", maker_b_data.amount);
    assert_eq!(maker_b_data.amount, 10_000_000);

    let taker_b_account = program.get_account(&taker_ata_b).unwrap();
    let taker_b_data = spl_token::state::Account::unpack(&taker_b_account.data).unwrap();
    msg!("Taker ATA B Balance (Remaining Token B): {}", taker_b_data.amount);
    assert_eq!(taker_b_data.amount, 990_000_000);
}

#[test]
fn test_update_expiration() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    let seed = 789u64;
    let (escrow, _) = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrowq32026::id(),
    );
    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

    // 1. Create Escrow with expiration 500
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 10_000_000,
            expiration: 500,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let make_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Make Transaction Successful ---");
    msg!("CUs Consumed: {}", make_tx.compute_units_consumed);
    msg!("Tx Signature: {}", make_tx.signature);

    let escrow_account = program.get_account(&escrow).unwrap();
    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    msg!("Initial Escrow Expiration: {}", escrow_data.expiration);

    // 2. Maker calls "Update" to extend expiration to 1500
    let update_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Update { maker, escrow }.to_account_metas(None),
        data: escrowq32026::instruction::Update { expiration: 1500 }.data(),
    };

    let message = Message::new(&[update_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let update_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Update Transaction Successful ---");
    msg!("CUs Consumed: {}", update_tx.compute_units_consumed);
    msg!("Tx Signature: {}", update_tx.signature);

    // 3. Verify and print escrow expiration was updated to 1500
    let escrow_account = program.get_account(&escrow).unwrap();
    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    msg!("Updated Escrow Expiration: {}", escrow_data.expiration);
    assert_eq!(escrow_data.expiration, 1500);
}

#[test]
fn test_refund_before_expiration_fails() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    let seed = 999u64;
    let (escrow, _) = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrowq32026::id(),
    );
    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

    // Create Escrow with expiration 10_000
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 10_000_000,
            expiration: 10_000,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let make_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Make Transaction Successful ---");
    msg!("CUs Consumed: {}", make_tx.compute_units_consumed);
    msg!("Tx Signature: {}", make_tx.signature);

    // Attempt refund at clock time = 0 (before expiration of 10_000)
    let refund_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Refund {
            maker,
            mint_a,
            maker_ata_a,
            escrow,
            vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Refund {}.data(),
    };

    let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let result = program.send_transaction(transaction);
    assert!(result.is_err(), "Refund before expiration should fail");
    msg!("\nRefund before expiration rejected as expected (EscrowNotExpired)");
}

#[test]
fn test_take_after_expiration_fails() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    let taker = Keypair::new();
    program.airdrop(&taker.pubkey(), 1_000_000_000).unwrap();

    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000_000_000)
        .send()
        .unwrap();

    let seed = 111u64;
    let (escrow, _) = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrowq32026::id(),
    );
    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    let taker_ata_a = associated_token::get_associated_token_address(&taker.pubkey(), &mint_a);
    let maker_ata_b = associated_token::get_associated_token_address(&maker, &mint_b);

    // Create Escrow with expiration 500
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 10_000_000,
            expiration: 500,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let make_tx = program.send_transaction(transaction).unwrap();

    msg!("\n--- Make Transaction Successful ---");
    msg!("CUs Consumed: {}", make_tx.compute_units_consumed);
    msg!("Tx Signature: {}", make_tx.signature);

    // Warp clock past expiration (e.g. 501)
    let mut clock: Clock = program.get_sysvar();
    clock.unix_timestamp = 501;
    program.set_sysvar(&clock);
    msg!("Clock advanced past expiration to: {}", clock.unix_timestamp);

    // Attempt Take after expiration
    let take_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Take {
            taker: taker.pubkey(),
            maker,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Take {}.data(),
    };

    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = program.latest_blockhash();
    let transaction = Transaction::new(&[&taker], message, recent_blockhash);

    let result = program.send_transaction(transaction);
    assert!(result.is_err(), "Take after expiration should fail");
    msg!("Take after expiration rejected as expected (EscrowExpired)");
}
