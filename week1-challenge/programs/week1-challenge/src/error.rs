use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Insufficient token amount!")]
    InsufficientAmount,

    #[msg("Inconsistent bump used!")]
    InconsistentBump,
}
