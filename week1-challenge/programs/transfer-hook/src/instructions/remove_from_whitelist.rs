use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::Whitelist;

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: this account is the token account authority. It doesn't need to check
    pub user: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"whitelist", mint.key().as_ref(), user.key().as_ref()],
        bump = whitelist.bump,
        constraint = whitelist.token_account == token_account.key(),
        close = admin,
    )]
    pub whitelist: Account<'info, Whitelist>,

    pub system_program: Program<'info, System>,
}
