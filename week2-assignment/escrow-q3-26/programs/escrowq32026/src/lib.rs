pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;



pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8WdKqANswW8EFvQxcjL6rGeeM6oFsuyTPtDymGJY4oH5");

// Two parties — a maker and a taker — can swap tokens without trusting each other or a third party.
// The maker deposits token A into a program-controlled vault and specifies how much of token B they want in return.
// Any taker who holds token B can complete the swap atomically. If no taker appears, the maker can reclaim their tokens at any time.

// Maker deposits token A  →  vault (PDA-owned)
//                                       ↓  taker sends token B to maker
//                                       ↓  vault releases token A to taker
//                                       ↓  escrow + vault accounts closed, rent returned

#[program]
pub mod escrowq32026 {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        deposit: u64,
        receive: u64,
        expiration: i64,
    ) -> Result<()> {
        ctx.accounts
            .init_escrow(seed, receive, &ctx.bumps, expiration)?;
        ctx.accounts.deposit(deposit)
    }

    //take instruction
    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.transfer_to_maker()?;
        ctx.accounts.withdraw_and_close_vault()
    }
    

    //Refund
    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    //Update

    #[instruction(discriminator = 3)]
    pub fn update(ctx: Context<Update>, expiration: i64) -> Result<()> {
        ctx.accounts.update_escrow(expiration)
    }

}
