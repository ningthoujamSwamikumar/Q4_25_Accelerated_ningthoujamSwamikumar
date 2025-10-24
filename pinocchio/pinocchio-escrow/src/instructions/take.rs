use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing Take Instruction.");
    let [
        taker, //payer
        maker,  //escrow account creator
        _mint_a,
        _mint_b,
        escrow_account, //escrow pda account
        taker_ata_a, //receiving token account of taker for mint a
        taker_ata_b, //sending token account of taker for mint b
        maker_ata_b, //transfer destination for token mint b
        escrow_ata, //vault, stores token for mint_a from maker
        _system_program,
        _token_program,
        _associated_token_program,
        _rest @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    msg!("extracted accounts");
    {
        //validate account owners
        let taker_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(taker_ata_a)?;
        let taker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_info(taker_ata_b)?;
        let escrow_ata_state = pinocchio_token::state::TokenAccount::from_account_info(escrow_ata)?;
        if taker_ata_a_state.owner() != taker.key()
            || taker_ata_b_state.owner() != taker.key()
            || escrow_ata_state.owner() != escrow_account.key()
            || escrow_account.owner() != &crate::ID
        {
            return Err(ProgramError::IllegalOwner);
        };
    }

    msg!("validated account owners");

    //validate pda
    let bump = data[0];
    let escrow_account_seed = [b"escrow".as_ref(), maker.key().as_slice(), &[bump]]; 
    let escrow_account_pda = derive_address(&escrow_account_seed, None, &crate::ID);
    assert_eq!(
        *escrow_account.key(),
        escrow_account_pda,
        "Invalid PDA provided"
    );

    msg!("escrow account validated!");

    //transfer the desired amount of
    let escrow_account_state = Escrow::from_account_info(escrow_account)?;
    pinocchio_token::instructions::Transfer {
        amount: escrow_account_state.amount_to_receive(),
        authority: taker,
        from: taker_ata_b,
        to: maker_ata_b,
    }
    .invoke()?;

    msg!("token b deposited to maker ata b");

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let signer_seeds = Signer::from(&seed);
    pinocchio_token::instructions::Transfer {
        amount: escrow_account_state.amount_to_give(),
        authority: escrow_account,
        from: escrow_ata,
        to: taker_ata_a,
    }
    .invoke_signed(&[signer_seeds])
}
