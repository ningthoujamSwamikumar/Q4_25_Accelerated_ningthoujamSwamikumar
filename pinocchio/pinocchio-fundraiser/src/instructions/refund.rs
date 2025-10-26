use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, msg, program_error::ProgramError, pubkey::find_program_address, sysvars::{clock::Clock, Sysvar}, ProgramResult
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{error::FundraiserError, instructions::TOKEN_2022_PROGRAM_ID, state::{contributor::ContributorAccount, fundraiser::FundRaiser, HasLen}};

// get refund when change of mind before time out, or amount to raise is reached
pub fn process_refund(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        contributor,
        mint,
        maker,
        fundraiser,
        contributor_ata,
        vault,
        contribution, //contribution account
        system_program, //to close the contribution account
        token_program,
        associated_token_program        
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    //validate accounts

    //validate signer
    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };
    //validate owners
    if !pinocchio_system::check_id(system_program.key())
        || !(pinocchio_token::check_id(token_program.key())
            || (token_program.key() == &TOKEN_2022_PROGRAM_ID))
        || !pinocchio_associated_token_account::check_id(associated_token_program.key())
    {
        return Err(ProgramError::InvalidAccountData);
    };
    if !contributor.is_owned_by(system_program.key())
        || !mint.is_owned_by(token_program.key())
        || !maker.is_owned_by(system_program.key())
    {
        return Err(ProgramError::InvalidAccountOwner);
    };
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
    //contributor ata
    let seeds = [
        contributor.key().as_ref(),
        token_program.key().as_ref(),
        mint.key().as_ref(),
    ];
    let contributor_ata_pda = find_program_address(&seeds, &pinocchio_associated_token_account::ID);
    if *contributor_ata.key() != contributor_ata_pda.0 {
        return Err(ProgramError::InvalidAccountData);
    };
    //contribution account
    msg!("validating contribution account.");
    let seeds = [
        b"contribution",
        fundraiser.key().as_ref(),
        contributor.key().as_ref(),
    ];
    let contribution_pda = find_program_address(&seeds, &crate::ID);
    if contribution_pda.0 != *contribution.key() {
        return Err(ProgramError::InvalidAccountData);
    };
    msg!("contribution account validated successfully");

    //needn't to validate data because all the amount contributed will be refunded

    //check time constraints
    let fundraiser_data_mut = FundRaiser::from_account_info_mut(fundraiser)?;
    let current_time = Clock::get()?.unix_timestamp;
    if fundraiser_data_mut.start_time() > current_time {
        return Err(ProgramError::from(FundraiserError::TooEarly));
    };
    if fundraiser_data_mut.start_time().checked_add(fundraiser_data_mut.duration() as i64).unwrap() < current_time {
        return Err(ProgramError::from(FundraiserError::TooLate));
    };
    msg!("passed time check for fundraising duration");
    if fundraiser_data_mut.current_amount().ge(&fundraiser_data_mut.amount_to_raise()) {
        return Err(ProgramError::from(FundraiserError::TargetAmountRaised));
    };
    msg!("passed amount check: amount to raised is not met");

    if contribution.data_len() != ContributorAccount::LEN {
        return Err(ProgramError::InvalidAccountData);
    };
    let contribution_account = unsafe {
        &*(contribution.borrow_data_unchecked().as_ptr() as *const ContributorAccount)
    };

    let bump = [fundraiser_pda.1];
    let seed = [Seed::from(b"fundraiser"), Seed::from(maker.key().as_ref()), Seed::from(&bump)];
    let signer_seed = Signer::from(&seed);
    Transfer {
        from: vault,
        to: contributor_ata,
        amount: u64::from_le_bytes(contribution_account.contribution),
        authority: fundraiser
    }.invoke_signed(&[signer_seed])?;
    msg!("contribution refund trasferred successfully");

    //close contribution account
    let lamports = contribution.lamports();
    unsafe {
        *contributor.borrow_mut_lamports_unchecked() += lamports;
        *contribution.borrow_mut_lamports_unchecked() = 0;
        //zero fill the account data for security reasons
        contribution.borrow_mut_data_unchecked().fill(0);
    }
    //contribution account will be garbage collected

    Ok(())
}
