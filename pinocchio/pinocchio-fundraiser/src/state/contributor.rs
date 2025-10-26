use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::HasLen;

#[repr(C)]
pub struct ContributorAccount {
    pub contributor: [u8; 32],
    pub contribution: [u8; 8],
}

impl HasLen for ContributorAccount {
    const LEN: usize = 32 + 8;
}

impl ContributorAccount {
    pub fn from_account_info_mut(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self) })
    }
}
