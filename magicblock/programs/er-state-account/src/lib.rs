#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod instructions;
mod state;

use instructions::*;

declare_id!("AZdi44i7DALnmupX5MZPoKCzvJ8s9CAUz15emtBm1aZ5");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;

        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>, new_data: u64) -> Result<()> {
        ctx.accounts.update(new_data)?;

        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)?;

        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;

        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;

        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;

        Ok(())
    }

    /// request randomness
    pub fn update_random(ctx: Context<UpdateRandom>, client_seed: u8) -> Result<()> {
        ctx.accounts.request_random_update(client_seed)
    }

    pub fn update_random_delegated(
        ctx: Context<UpdateRandomDelegated>,
        client_seed: u8,
    ) -> Result<()> {
        ctx.accounts.request_random_delegated(client_seed)
    }

    /// consume randomness callback
    pub fn update_random_callback(
        ctx: Context<RandomCallback>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.random_callback(randomness)
    }
}
