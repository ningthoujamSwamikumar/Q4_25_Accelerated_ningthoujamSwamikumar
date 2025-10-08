use anchor_lang::prelude::*;

use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        bump = whitelist.bump,
        close = admin,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist() -> Result<()> {
        Ok(())
    }
}
