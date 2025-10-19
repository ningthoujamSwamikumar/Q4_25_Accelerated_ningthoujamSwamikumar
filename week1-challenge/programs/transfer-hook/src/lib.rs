#![allow(unexpected_cfgs, deprecated)]

#[cfg(test)]
pub mod tests;

pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

use anchor_lang::prelude::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
};

declare_id!("7ETfaPkuhFxGFs25NTPk1idtJXHQ2WSdHbDtwGHcupV1");

#[program]
pub mod transfer_hook {

    use spl_tlv_account_resolution::state::ExtraAccountMetaList;

    use super::*;

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_transfer_hook_accounts(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_account_metas)?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }

    pub fn init_mint(_ctx: Context<InitMint>) -> Result<()> {
        Ok(())
    }

    pub fn init_whitelist(ctx: Context<InitWhitelist>) -> Result<()> {
        ctx.accounts.init_whitelist(ctx.bumps)
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>) -> Result<()> {
        ctx.accounts.add_to_whitelist(ctx.bumps.whitelist)
    }

    pub fn remove_from_whitelist(_ctx: Context<RemoveFromWhitelist>) -> Result<()> {
        Ok(())
    }
}
