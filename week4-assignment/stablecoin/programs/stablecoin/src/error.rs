use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("Failed to calculate transfer fee for the current epoch")]
    FeeCalculationFailed,
    #[msg("Account length calculation failed for the requested extensions")]
    AccountLenCalculationFailed,
    #[msg("Invalid extension data or extension missing")]
    InvalidExtension,
    #[msg("Account is frozen until KYC verification completes")]
    AccountFrozen,
    #[msg("Invalid authority provided for this operation")]
    InvalidAuthority,
    #[msg("Permanent delegate cannot decrypt or seize confidential balances")]
    ConfidentialBalanceCannotBeSeized,
}
