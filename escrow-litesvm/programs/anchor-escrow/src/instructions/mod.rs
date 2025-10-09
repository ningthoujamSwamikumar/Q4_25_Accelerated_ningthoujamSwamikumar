pub mod make;
pub mod refund;
pub mod refund_before_time;
pub mod take;
pub mod take_after_time;

use anchor_lang::error_code;
pub use make::*;
pub use refund::*;
pub use refund_before_time::*;
pub use take::*;
pub use take_after_time::*;

#[error_code]
pub enum EscrowError {
    #[msg("Too early to take offer!")]
    TooEarlyToTakeOffer,
    #[msg("Too late to refund and withdraw offer!")]
    TooLateToRefund,
}
