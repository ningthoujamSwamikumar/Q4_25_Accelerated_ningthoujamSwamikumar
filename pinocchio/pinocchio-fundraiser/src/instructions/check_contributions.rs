use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    error::FundraiserError, instructions::TOKEN_2022_PROGRAM_ID, state::fundraiser::FundRaiser,
};

pub fn process_check_contributions(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, maker_ata, vault, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
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
    //maker ata validation
    let seeds = [
        maker.key().as_ref(),
        token_program.key().as_ref(),
        mint.key().as_ref(),
    ];
    let maker_ata_pda = find_program_address(&seeds, &pinocchio_associated_token_account::ID);
    if *maker_ata.key() != maker_ata_pda.0 {
        return Err(ProgramError::InvalidAccountData);
    };
    msg!("Account validations successfull");

    //i believe we need to check if the duration has end, otherwise error

    //other logic checks
    let fundraiser_account = FundRaiser::from_account_info_mut(fundraiser)?;
    if fundraiser_account.current_amount() < fundraiser_account.amount_to_raise() {
        return Err(ProgramError::from(FundraiserError::TargetAmountNotMet));
    };

    //transfer all the amount to maker
    let bump = [fundraiser_pda.1];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump),
    ];
    let signer_seed = Signer::from(&seed);
    Transfer {
        from: vault,
        to: maker_ata,
        amount: fundraiser_account.current_amount(),
        authority: fundraiser,
    }
    .invoke_signed(&[signer_seed])?;

    Ok(())
}
