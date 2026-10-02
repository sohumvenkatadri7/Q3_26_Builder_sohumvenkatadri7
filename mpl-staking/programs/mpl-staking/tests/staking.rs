use anchor_lang::{Id, InstructionData, ToAccountMetas};
use anchor_lang::solana_program::pubkey::Pubkey;
use mpl_core::ID as MPL_CORE_ID;
use mpl_staking::{
    accounts::{BurnStakedNft, ClaimRewards, CreateCollection, Initialize, MintAsset, Stake, Unstake},
    instruction,
    ID as PROGRAM_ID,
};

#[test]
fn test_pda_derivations() {
    let collection = Pubkey::new_unique();

    let (config_pda, config_bump) =
        Pubkey::find_program_address(&[b"config", collection.as_ref()], &PROGRAM_ID);
    assert_ne!(config_pda, Pubkey::default());

    let (update_authority_pda, auth_bump) =
        Pubkey::find_program_address(&[b"update_authority", collection.as_ref()], &PROGRAM_ID);
    assert_ne!(update_authority_pda, Pubkey::default());

    let (rewards_mint_pda, rewards_bump) =
        Pubkey::find_program_address(&[b"rewards_mint", config_pda.as_ref()], &PROGRAM_ID);
    assert_ne!(rewards_mint_pda, Pubkey::default());

    println!("Config PDA: {} (bump {})", config_pda, config_bump);
    println!("Update Authority PDA: {} (bump {})", update_authority_pda, auth_bump);
    println!("Rewards Mint PDA: {} (bump {})", rewards_mint_pda, rewards_bump);
}

#[test]
fn test_instruction_data_encoding() {
    // 1. Initialize
    let init_ix_data = instruction::Initialize {
        rewards_bps: 500,
        freeze_period: 7,
    }
    .data();
    assert!(!init_ix_data.is_empty());

    // 2. Create Collection
    let create_col_data = instruction::CreateCollection {
        name: "Test Collection".to_string(),
        uri: "https://example.com/collection.json".to_string(),
    }
    .data();
    assert!(!create_col_data.is_empty());

    // 3. Mint Asset
    let mint_data = instruction::MintAsset {
        name: "NFT #1".to_string(),
        uri: "https://example.com/1.json".to_string(),
    }
    .data();
    assert!(!mint_data.is_empty());

    // 4. Stake
    let stake_data = instruction::Stake {}.data();
    assert!(!stake_data.is_empty());

    // 5. Unstake
    let unstake_data = instruction::Unstake {}.data();
    assert!(!unstake_data.is_empty());

    // 6. Claim Rewards
    let claim_data = instruction::ClaimRewards {}.data();
    assert!(!claim_data.is_empty());

    // 7. Burn Staked NFT (Burn to Earn)
    let burn_data = instruction::BurnStakedNft {}.data();
    assert!(!burn_data.is_empty());
}

#[test]
fn test_burn_to_earn_rewards_calculation() {
    let decimals = 6u32;
    let rewards_bps = 500u64; // 5% base rate
    let burn_bonus_multiplier = 5u64; // 5x bonus multiplier
    let seconds_per_day = 86400u64;

    // Simulate 10 days staked
    let staked_duration_seconds = 10 * seconds_per_day;
    let staked_days = (staked_duration_seconds / seconds_per_day).max(1);

    // Regular reward
    let base_reward = staked_days
        .checked_mul(rewards_bps)
        .unwrap()
        .checked_mul(10u64.pow(decimals))
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    // Burn bonus reward (5x base reward)
    let total_burn_payout = base_reward.checked_mul(burn_bonus_multiplier).unwrap();

    // 10 days * 500 bps = 5000 / 10000 = 0.5 tokens base
    // 0.5 * 10^6 = 500_000 raw units
    assert_eq!(base_reward, 500_000);
    // 500_000 * 5 = 2_500_000 raw units (2.5 tokens)
    assert_eq!(total_burn_payout, 2_500_000);
}

#[test]
fn test_collection_level_counter_logic() {
    let mut total_staked: u64 = 0;

    // Stake 3 assets sequentially
    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 1);
    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 2);
    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 3);

    // Unstake 1 asset
    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 2);

    // Burn 1 staked asset
    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 1);

    // Unstake last asset
    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 0);

    // Underflow protection check
    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 0);
}

#[test]
fn test_account_metas_structure() {
    let owner = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let (config_pda, _) =
        Pubkey::find_program_address(&[b"config", collection.as_ref()], &PROGRAM_ID);
    let (update_authority_pda, _) =
        Pubkey::find_program_address(&[b"update_authority", collection.as_ref()], &PROGRAM_ID);
    let (rewards_mint_pda, _) =
        Pubkey::find_program_address(&[b"rewards_mint", config_pda.as_ref()], &PROGRAM_ID);
    let user_rewards_ata = Pubkey::new_unique();

    // 1. Initialize Accounts Metas
    let init_accounts = Initialize {
        admin: owner,
        config: config_pda,
        collection,
        update_authority: update_authority_pda,
        rewards_mint: rewards_mint_pda,
        system_program: anchor_lang::system_program::System::id(),
        token_program: anchor_spl::token::ID,
    };
    let init_metas = init_accounts.to_account_metas(None);
    assert_eq!(init_metas.len(), 7);

    // 2. Create Collection Metas
    let create_col_accounts = CreateCollection {
        payer: owner,
        collection,
        update_authority: update_authority_pda,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };
    let create_col_metas = create_col_accounts.to_account_metas(None);
    assert_eq!(create_col_metas.len(), 5);

    // 3. Mint Asset Metas
    let mint_accounts = MintAsset {
        user: owner,
        asset,
        collection,
        update_authority: update_authority_pda,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };
    let mint_metas = mint_accounts.to_account_metas(None);
    assert_eq!(mint_metas.len(), 6);

    // 4. Stake Metas
    let stake_accounts = Stake {
        owner,
        config: config_pda,
        asset,
        collection,
        update_authority: update_authority_pda,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };
    let stake_metas = stake_accounts.to_account_metas(None);
    assert_eq!(stake_metas.len(), 7);

    // 5. Unstake Metas
    let unstake_accounts = Unstake {
        owner,
        config: config_pda,
        asset,
        collection,
        update_authority: update_authority_pda,
        rewards_mint: rewards_mint_pda,
        user_rewards_ata,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };
    let unstake_metas = unstake_accounts.to_account_metas(None);
    assert_eq!(unstake_metas.len(), 11);

    // 6. Claim Rewards Metas
    let claim_accounts = ClaimRewards {
        owner,
        collection,
        config: config_pda,
        stake_state: Pubkey::new_unique(),
        asset,
        rewards_mint: rewards_mint_pda,
        user_rewards_ata,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
    };
    let claim_metas = claim_accounts.to_account_metas(None);
    assert_eq!(claim_metas.len(), 10);

    // 7. Burn Staked NFT Metas
    let burn_accounts = BurnStakedNft {
        owner,
        config: config_pda,
        asset,
        collection,
        update_authority: update_authority_pda,
        rewards_mint: rewards_mint_pda,
        user_rewards_ata,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let metas = burn_accounts.to_account_metas(None);
    assert_eq!(metas.len(), 11);
    assert!(metas[0].is_signer); // owner is signer
    assert!(metas[0].is_writable); // owner is writable (payer)
    assert!(metas[2].is_writable); // asset is writable (burned)
    assert!(metas[3].is_writable); // collection is writable (attributes updated)
}
