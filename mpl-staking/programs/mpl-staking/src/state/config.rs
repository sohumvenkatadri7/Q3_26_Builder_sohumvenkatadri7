use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub rewards_bps: u16,       // rewards percentage in basis points
    pub freeze_period: u16,     // minimum freeze period in days
    pub rewards_bump: u8,       // Bumps for the rewards mint account
    pub bump: u8,               // Bumps for the config account
}


#[account]
#[derive(InitSpace)]
pub struct StakeState {
    pub owner: Pubkey,       // Owner of the stake
    pub asset: Pubkey,       // Asset being staked
    pub staked_at: u64,       // Timestamp of when the stake was staked
    pub last_claimed_at: u64, // Timestamp of when the last reward was claimed
    pub bump: u8,               // Bumps for the StakeState account
}