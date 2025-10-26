use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::state::HasLen;

#[repr(C)]
#[derive(Debug)]
pub struct FundRaiser {
    authority: [u8; 32], //maker
    mint_to_raise: [u8; 32],
    amount_to_raise: [u8; 8],
    current_amount: [u8; 8],
    start_time: [u8; 8],
    duration: u8, //in days
    pub bump: u8,
}

impl HasLen for FundRaiser {
    const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1;
}

impl FundRaiser {
    pub fn from_account_info_mut(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;
        if data.len() != FundRaiser::LEN {
            return Err(ProgramError::InvalidAccountData);
        };

        // ensure alignment to be predictable
        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        };

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn authority(&self) -> Pubkey {
        Pubkey::from(self.authority)
    }

    pub fn set_authority(&mut self, authority: &Pubkey) {
        self.authority = *authority;
    }

    pub fn mint_to_raise(&self) -> Pubkey {
        Pubkey::from(self.mint_to_raise)
    }

    pub fn set_mint_to_raise(&mut self, mint: &Pubkey) {
        self.mint_to_raise = *mint;
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount: &u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn current_amount(&self) -> u64 {
        u64::from_le_bytes(self.current_amount)
    }

    pub fn add_current_amount(&mut self, amount: &u64) {
        pinocchio_log::log!("amount adding {}", *amount);
        self.current_amount = (self.current_amount() + amount).to_le_bytes();
    }

    pub fn start_time(&self) -> i64 {
        i64::from_le_bytes(self.start_time)
    }

    pub fn set_start_time(&mut self, time: &i64) {
        self.start_time = time.to_le_bytes();
    }

    pub fn duration(&self) -> u8 {
        self.duration
    }

    pub fn set_duration(&mut self, duration: &u8) {
        self.duration = *duration;
    }
}
