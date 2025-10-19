use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensions, PodStateWithExtensions,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount},
};

use crate::Whitelist;

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner
    /// This account is not being checked because it is used for ownership validation within the `transfer_hook` instruction
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList Account
    /// This account is not being checked because it is used dynamically within the program logic.
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    // Accounts to add to the extra-account-metas
    #[account(
        seeds = [b"whitelist", source_token.key().as_ref()],
        bump = whitelist.bump,
        constraint = whitelist.token_account == source_token.key(),
    )]
    pub whitelist: Account<'info, Whitelist>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&self, _amount: u64) -> Result<()> {
        self.check_is_transferring()?;
        // whitelist check is done in the accounts only 
        Ok(())
    }

    /// Check if the transfer hook is being called during token transfer
    fn check_is_transferring(&self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let account_data = source_token_info.try_borrow_mut_data()?;
        let account = PodStateWithExtensions::<PodAccount>::unpack(&account_data)?;
        let extension = account.get_extension::<TransferHookAccount>()?;
        if !bool::from(extension.transferring) {
            panic!("TransferHook: Not transferring");
        };

        Ok(())
    }
}
