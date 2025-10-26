use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_system::instructions::CreateAccount;

use crate::{
    instructions::TOKEN_2022_PROGRAM_ID,
    state::{fundraiser::FundRaiser, HasLen},
};

pub fn process_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, vault, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //account validations

    //program account check
    if !pinocchio_system::check_id(system_program.key())
        || !pinocchio_associated_token_account::check_id(associated_token_program.key())
        || !(pinocchio_token::check_id(token_program.key())
            || (token_program.key() == &TOKEN_2022_PROGRAM_ID))
    {
        return Err(ProgramError::InvalidAccountData);
    };
    //maker should be signer
    if !maker.is_signer() {
        msg!("maker should be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    };
    //owner checks
    if !maker.is_owned_by(&pinocchio_system::ID) {
        msg!("maker should be a system account");
        return Err(ProgramError::InvalidAccountOwner);
    };
    if !mint.is_owned_by(token_program.key()) {
        msg!("mint should be a token program account");
        return Err(ProgramError::InvalidAccountOwner);
    };
    //vault and fundraiser are not yet initialized, and need not check thier owners

    //fundraiser validation
    let seeds = [b"fundraiser", maker.key().as_ref()];
    let fundraiser_pda = find_program_address(&seeds, &crate::id());
    if fundraiser_pda.0 != *fundraiser.key() {
        return Err(ProgramError::InvalidAccountData);
    };
    //vault validation
    let seeds = [
        fundraiser.key().as_ref(),
        token_program.key().as_ref(),
        mint.key().as_ref(),
    ];
    let vault_pda = find_program_address(&seeds, &pinocchio_associated_token_account::ID);
    if *vault.key() != vault_pda.0 {
        return Err(ProgramError::InvalidAccountData);
    };

    //validate data
    if data.len() != 8 + 8 + 1 {
        // amount to raise + start time + duration
        return Err(ProgramError::InvalidInstructionData);
    };

    msg!("all validations passed");

    //rent check
    let lamports = Rent::get()?.minimum_balance(FundRaiser::LEN);

    //create fundraiser account
    let bump = [fundraiser_pda.1];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seeds);
    CreateAccount {
        from: maker,
        lamports,
        owner: &crate::ID,
        space: FundRaiser::LEN as u64,
        to: fundraiser,
    }
    .invoke_signed(&[seeds])?;
    msg!("created fundraiser account");
    //set values to the fundraiser pda account
    let amount_to_raise = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let start_time = i64::from_le_bytes(data[8..16].try_into().unwrap());
    let duration = data[16];
    let fundraiser_account = FundRaiser::from_account_info_mut(fundraiser)?;
    fundraiser_account.set_amount_to_raise(&amount_to_raise);
    fundraiser_account.set_authority(maker.key());
    fundraiser_account.set_duration(&duration);
    fundraiser_account.set_mint_to_raise(mint.key());
    fundraiser_account.set_start_time(&start_time);

    //create vault
    Create {
        account: vault,
        funding_account: maker,
        mint,
        system_program,
        token_program,
        wallet: fundraiser, //owner
    }
    .invoke()?;

    Ok(())
}
