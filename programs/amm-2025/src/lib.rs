use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod state;
pub mod error;
pub mod utils;

declare_id!("38FqduQst5K8zRo62mBo4sduwxydZosuZs6v3AdAmKm6");

#[program]
pub mod amm_2025 {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Pubkey
    ) -> Result<()> {
        ctx.accounts.init(ctx.bumps, seed, fee, authority)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64,
        max_x: u64,
        max_y: u64,
        expiration: i64
    ) -> Result<()>{
        ctx.accounts.deposit(amount, max_x, max_y, expiration)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
        min_x: u64,
        min_y: u64,
        expiration: i64
    ) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y, expiration)
    }

    pub fn swap(
        ctx: Context<Swap>,
        x: bool,
        amount: u64,
        min: u64,
        expiration: i64
    ) -> Result<()>{
        ctx.accounts.swap(x, amount, min, expiration)
    }
}

