#![allow(deprecated, unexpected_cfgs)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_and_undelegate_accounts};

declare_id!("8M1NkwKGkrz8GrbWyCrvuEsHqrrt4UNnDaJsbhghCYyv");

#[ephemeral]
#[program]
pub mod counter {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.counter.set_inner(Counter { count: 0 });
        Ok(())
    }

    pub fn increment(ctx: Context<IncrementDecrement>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;

        Ok(())
    }

    pub fn decrement(ctx: Context<IncrementDecrement>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count -= 1;

        Ok(())
    }

    /// Delegate the account to the delegate program
    /// IMPORTANT: set specific validator based on ER
    pub fn delegate(ctx: Context<DelegateInput>) -> Result<()> {
        ctx.accounts.delegate_pda(
            &ctx.accounts.payer,
            &[b"counter"],
            DelegateConfig {
                //validator: ctx.remaining_accounts.first().map(|acc| acc.key()),
                validator: ctx.accounts.validator.clone().map(|acc| acc.key()),
                ..Default::default()
            }
        )?;
        Ok(())
    }

    /// undelegate the account from the delegation program
    pub fn commit_and_undelegate(ctx: Context<IncrementAndCommit>)->Result<()>{
        commit_and_undelegate_accounts(
            &ctx.accounts.user, 
            vec![&ctx.accounts.counter.to_account_info()], 
            &ctx.accounts.magic_context, 
            &ctx.accounts.magic_program
        )?;

        Ok(())
    }

    pub fn close_pda(_ctx: Context<ClosePda>)->Result<()>{
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + 8,
        seeds = [b"counter"],
        bump
    )]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementDecrement<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"counter"],
        bump,
    )]
    pub counter: Account<'info, Counter>,
}

#[account]
pub struct Counter {
    pub count: u64,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Checked by the delegate program
    pub validator: Option<AccountInfo<'info>>,

    /// CHECK the pda to delegate
    #[account(
        mut,
        del
    )]
    pub pda: AccountInfo<'info>,
}

/// Accounts for the increment instruction + manual commit
#[commit]
#[derive(Accounts)]
pub struct IncrementAndCommit<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut, 
        seeds = [b"counter"],
        bump
    )]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct ClosePda<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"counter"],
        bump,
        close = user
    )]
    pub counter: Account<'info, Counter>,
}
