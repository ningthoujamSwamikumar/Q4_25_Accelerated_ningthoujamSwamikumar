use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub token_account: Pubkey,
    pub bump: u8,
}
