use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultPda {
    pub bump: u8,
    /// authority who can close the vault
    pub close_authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct DepositPda {
    pub user: Pubkey,
    pub amount: u64,
    pub bump: u8,
    pub is_initialized: bool,
}
