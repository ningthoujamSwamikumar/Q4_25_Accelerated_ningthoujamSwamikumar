use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::DepositPda;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + DepositPda::INIT_SPACE,
        seeds = [b"deposit_pda", user.key().as_ref()],
        bump,
    )]
    pub deposit_pda: Account<'info, DepositPda>,

    #[account(
        mut,
        
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
}
