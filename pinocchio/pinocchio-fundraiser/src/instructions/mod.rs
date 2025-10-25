use pinocchio::program_error::ProgramError;

pub(crate) mod check_contributions;
pub(crate) mod contribute;
pub(crate) mod initialize;
pub(crate) mod refund;

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];

#[repr(u8)]
pub(crate) enum FundraiserInstruction {
    Initialize,
    Contribute,
    Refund,
    CheckContributions,
}

impl TryFrom<&u8> for FundraiserInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        let instruction = match *value {
            0 => Self::Initialize,
            1 => Self::Contribute,
            2 => Self::Refund,
            3 => Self::CheckContributions,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(instruction)
    }
}
