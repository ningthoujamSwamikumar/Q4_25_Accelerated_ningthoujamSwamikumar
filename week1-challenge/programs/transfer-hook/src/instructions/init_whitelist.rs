use anchor_lang::prelude::*;

use crate::WhitelistConfig;

#[derive(Accounts)]
pub struct InitWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + WhitelistConfig::INIT_SPACE,
        seeds = [b"whitelist_config"],
        bump
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitWhitelist<'info> {
    pub fn init_whitelist(&mut self, bumps: InitWhitelistBumps) -> Result<()> {
        self.whitelist_config.set_inner(WhitelistConfig {
            admin: self.admin.key(),
            bump: bumps.whitelist_config,
        });

        Ok(())
    }
}
