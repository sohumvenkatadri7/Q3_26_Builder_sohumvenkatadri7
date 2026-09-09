use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("The escrow has already expired.")]
    EscrowExpired,
    #[msg("The escrow expiration time must be in the future.")]
    InvalidExpiration,
    #[msg("The escrow has not expired yet.")]
    EscrowNotExpired,
}
