use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::{
    anchor::vrf,
    instructions::{create_request_randomness_ix, RequestRandomnessParams},
    types::SerializableAccountMeta,
};

use crate::{instruction, state::UserAccount};

#[vrf]
#[derive(Accounts)]
pub struct UpdateRandomDelegated<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: The oracle queue
    #[account(
        mut,
        address = ephemeral_vrf_sdk::consts::DEFAULT_EPHEMERAL_QUEUE // This is the only difference between update_random and update_random_delegated
    )]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info> UpdateRandomDelegated<'info> {
    pub fn request_random_delegated(&self, client_seed: u8) -> Result<()> {
        msg!("Requesting randomness...");
        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator: instruction::UpdateRandomCallback::DISCRIMINATOR.to_vec(),
            caller_seed: [client_seed; 32],
            // specify any account that is required by the callback
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });

        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;

        Ok(())
    }
}
