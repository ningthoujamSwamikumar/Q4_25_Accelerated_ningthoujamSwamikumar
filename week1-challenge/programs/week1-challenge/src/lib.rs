#![allow(deprecated, unexpected_cfgs)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2wsfuBAYRiG1XHEB3NPZ8ZKdTg98gMUxyfWumkLJbVeJ");

const TRANSFER_HOOK_ID: Pubkey = Pubkey::from_str_const("7ETfaPkuhFxGFs25NTPk1idtJXHQ2WSdHbDtwGHcupV1");

#[program]
pub mod week1_challenge {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.initialize_vault(ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_deposit(amount, ctx.bumps)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_withdraw(amount)
    }
}
