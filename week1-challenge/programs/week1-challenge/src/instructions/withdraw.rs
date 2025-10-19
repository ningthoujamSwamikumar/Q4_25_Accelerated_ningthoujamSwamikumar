use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::ErrorCode, DepositPda, VaultPda, TRANSFER_HOOK_ID};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"deposit_account", user.key().as_ref()],
        bump,
        has_one = user
    )]
    pub deposit_account: Account<'info, DepositPda>,

    #[account(
        extensions::transfer_hook::program_id = TRANSFER_HOOK_ID
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault_pda,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"vault_pda"],
        bump = vault_pda.bump
    )]
    pub vault_pda: Account<'info, VaultPda>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn transfer_withdraw(&mut self, amount: u64) -> Result<()> {
        require!(
            self.deposit_account.amount >= amount,
            ErrorCode::InsufficientAmount
        );

        self.deposit_account.amount -= amount;
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    authority: self.vault_pda.to_account_info(),
                    from: self.vault.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.user_token.to_account_info(),
                },
                &[&[b"vault_pda", &[self.vault_pda.bump]]], //slice of byte array ref // [b"..", &[bump]] //this is byte array
            ),
            amount,
            self.mint.decimals,
        )
    }
}
