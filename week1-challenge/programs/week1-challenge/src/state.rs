use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultPda {
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct DepositPda {
    pub amount: u64,
    pub bump: u8,
}
