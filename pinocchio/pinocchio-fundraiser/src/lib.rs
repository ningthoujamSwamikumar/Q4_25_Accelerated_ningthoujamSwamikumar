#![allow(unexpected_cfgs)]

use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::{
    contribute::process_contribution, initialize::process_initialize, FundraiserInstruction,
};

pub mod error;
pub mod instructions;
pub mod state;

#[cfg(test)]
pub mod tests;

program_entrypoint!(process_instruction);
no_allocator!();
default_panic_handler!();

pinocchio_pubkey::declare_id!("J18Rbg2x2mFoirYByYemaaiddj3CntggBWGsBBeLmnTM");

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstruction::try_from(discriminator)? {
        FundraiserInstruction::Initialize => process_initialize(accounts, data),
        FundraiserInstruction::Contribute => process_contribution(accounts, data),
        // FundraiserInstruction::Refund => todo!(),
        // FundraiserInstruction::CheckContributions => todo!(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
