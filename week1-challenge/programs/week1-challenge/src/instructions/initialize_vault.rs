use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{initialize_vault, VaultPda};
use transfer_hook::ID as transfer_hook_id;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = vault_pda,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        space = 8 + VaultPda::INIT_SPACE,
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: Account<'info, VaultPda>,

    #[account(
        extensions::transfer_hook::program_id = transfer_hook_id,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitializeVault<'info> {
    pub fn initialize_vault(&mut self, bumps: InitializeVaultBumps) -> Result<()> {
        self.vault_pda.set_inner(VaultPda {
            authority: self.admin.key(),
            bump: bumps.vault_pda,
        });
        Ok(())
    }
}
