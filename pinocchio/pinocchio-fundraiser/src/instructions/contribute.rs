use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::Mint};

use crate::{
    error::FundraiserError,
    instructions::{MAX_CONTRIBUTION_PC, MIN_CONTRIBUTION_TOKEN_AMOUNT, TOKEN_2022_PROGRAM_ID},
    state::{contributor::ContributorAccount, fundraiser::FundRaiser, HasLen},
};

pub fn process_contribution(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, mint, maker, fundraiser, contributor_ata, vault, contribution_account, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
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
    if contribution_pda.0 != *contribution_account.key() {
        return Err(ProgramError::InvalidAccountData);
    };
    msg!("contribution account validated successfully");

    //data validation
    if data.len() != 8 {
        //8 for contributing amount
        return Err(ProgramError::InvalidInstructionData);
    };
    let contributing_amount: u64 = u64::from_le_bytes(data.try_into().unwrap());

    if fundraiser.data_len() != FundRaiser::LEN {
        return Err(ProgramError::InvalidAccountData);
    };
    let fundraiser_data_acc_mut = FundRaiser::from_account_info_mut(fundraiser)?;
    //validate the time contraints
    let current_time = Clock::get()?.unix_timestamp;
    if current_time < fundraiser_data_acc_mut.start_time() {
        return Err(ProgramError::from(FundraiserError::TooEarly));
    };
    let end_time = fundraiser_data_acc_mut
        .start_time()
        .saturating_add(fundraiser_data_acc_mut.duration() as i64);
    if current_time > end_time {
        return Err(ProgramError::from(FundraiserError::TooLate));
    };

    let mint_acc_data =
        unsafe { Mint::from_bytes_unchecked(&mint.borrow_data_unchecked()[0..Mint::LEN]) };

    if contributing_amount < (MIN_CONTRIBUTION_TOKEN_AMOUNT * mint_acc_data.decimals()) as u64 {
        return Err(ProgramError::from(FundraiserError::MinContribution));
    };
    let max_contribution_scalar = (MAX_CONTRIBUTION_PC as u64)
        .checked_mul(fundraiser_data_acc_mut.amount_to_raise())
        .unwrap()
        .checked_div(100u64)
        .unwrap();

    //update fundraiser state, even though few constraint checks are left to not get borrow error
    fundraiser_data_acc_mut.add_current_amount(&contributing_amount);

    if contribution_account.lamports() == 0 && contribution_account.owner() == &pinocchio_system::ID
    {
        // account is not initialized, and so initialized the account
        //check the MAX CONTRIBUTION constraint
        if contributing_amount > max_contribution_scalar {
            return Err(ProgramError::from(FundraiserError::MaxContribution));
        };
        let bump = [contribution_pda.1];
        let seeds = [
            Seed::from(b"contribution"),
            Seed::from(fundraiser.key().as_ref()),
            Seed::from(contributor.key().as_ref()),
            Seed::from(&bump),
        ];
        let signer_seeds = Signer::from(&seeds);
        msg!("creating contributing account");
        //initialize the account, and check the contributing amount
        let lamports = Rent::get()?.minimum_balance(ContributorAccount::LEN);
        CreateAccount {
            from: contributor,
            lamports,
            owner: &crate::ID,
            to: contribution_account,
            space: ContributorAccount::LEN as u64,
        }
        .invoke_signed(&[signer_seeds])?;
        msg!("created contributing account");
        //set the data
        let contributor_acc_data_mut =
            ContributorAccount::from_account_info_mut(contribution_account)?;
        contributor_acc_data_mut.contribution = contributing_amount.to_le_bytes();
        contributor_acc_data_mut.contributor = *contributor.key();
    } else {
        //check the contributing amount + already contributed amount against the MAX CONTRIBUTION constraint
        let contributor_acc_data_mut =
            ContributorAccount::from_account_info_mut(contribution_account)?;
        let total_amnt = contributing_amount
            .checked_add(u64::from_le_bytes(contributor_acc_data_mut.contribution))
            .unwrap();
        if total_amnt > max_contribution_scalar {
            return Err(ProgramError::from(FundraiserError::MaxContribution));
        };
        contributor_acc_data_mut.contribution = total_amnt.to_le_bytes();
    };
    msg!("tranferring contribution amount to vault");
    //transfer the contributing amount to vault
    Transfer {
        from: contributor_ata,
        to: vault,
        amount: contributing_amount,
        authority: contributor,
    }
    .invoke()
}
