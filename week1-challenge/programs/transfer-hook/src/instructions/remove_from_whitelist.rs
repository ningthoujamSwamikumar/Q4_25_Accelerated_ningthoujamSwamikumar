use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{Whitelist, WhitelistConfig};

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(
        mut,
        constraint = *admin.key == whitelist_config.admin
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"whitelist_config"],
        bump = whitelist_config.bump,
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,

    /// CHECK: this account is the token account authority. It doesn't need to check
    pub user: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
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
    pub token_program: Interface<'info, TokenInterface>
}
