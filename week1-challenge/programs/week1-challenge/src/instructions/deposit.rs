use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::ErrorCode, TRANSFER_HOOK_ID};
use crate::DepositPda;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + DepositPda::INIT_SPACE,
        seeds = [b"deposit_account", user.key().as_ref()],
        bump,
        has_one = user
    )]
    pub deposit_account: Account<'info, DepositPda>,

    #[account(
        extensions::transfer_hook::program_id = TRANSFER_HOOK_ID
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Deposit<'info> {
    pub fn transfer_deposit(&mut self, amount: u64, bumps: DepositBumps) -> Result<()> {
        require!(
            self.user_token.amount >= amount,
            ErrorCode::InsufficientAmount
        );

        if !self.deposit_account.is_initialized {
            self.deposit_account.set_inner(DepositPda {
                user: self.user.key(),
                amount,
                bump: bumps.deposit_account,
                is_initialized: true,
            });
        } else {
            require!(
                self.deposit_account.bump == bumps.deposit_account,
                ErrorCode::InconsistentBump
            );
            self.deposit_account.amount += amount;
        };

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    authority: self.user.to_account_info(),
                    from: self.user_token.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.vault.to_account_info(),
                },
            ),
            amount,
            self.mint.decimals,
        )
    }
}
