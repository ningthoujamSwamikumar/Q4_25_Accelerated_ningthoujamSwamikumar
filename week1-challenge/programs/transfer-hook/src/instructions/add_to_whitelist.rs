use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::Whitelist;

#[derive(Accounts)]
pub struct AddToWhitelist<'info> {
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
        init,
        payer = admin,
        space = 8 + Whitelist::INIT_SPACE,
        seeds = [b"whitelist", mint.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, whitelist_bump: u8) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            token_account: self.token_account.key(),
            bump: whitelist_bump,
        });

        Ok(())
    }
}
