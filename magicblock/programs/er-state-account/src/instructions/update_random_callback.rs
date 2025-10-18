use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct RandomCallback<'info>{
    /// This check ensure vrf_program_identity (which is a PDA) is a signer
    /// enforcing the callback is executed by the VRF program through CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl <'info> RandomCallback<'info> {
    pub fn random_callback(&mut self, randomness: [u8; 32])->Result<()>{
        let rnd_u64 = ephemeral_vrf_sdk::rnd::random_u64(&randomness);

        self.user_account.data = rnd_u64; //update the user account data
        Ok(())
    }
}
