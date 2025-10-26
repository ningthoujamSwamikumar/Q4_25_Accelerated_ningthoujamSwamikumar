use pinocchio::program_error::{ProgramError, ToStr};

pub enum FundraiserError {
    TooEarly,
    TooLate,
    MaxContribution,
    MinContribution,
    TargetAmountRaised,
    TargetAmountNotMet,
}

impl From<FundraiserError> for ProgramError {
    fn from(value: FundraiserError) -> Self {
        Self::Custom(value as u32)
    }
}

impl TryFrom<u32> for FundraiserError {
    type Error = ProgramError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let err = match value {
            0 => Self::TooEarly,
            1 => Self::TooLate,
            2 => Self::MaxContribution,
            3 => Self::MinContribution,
            4 => Self::TargetAmountRaised,
            5 => Self::TargetAmountNotMet,
            _ => return Err(ProgramError::InvalidArgument),
        };
        Ok(err)
    }
}

impl ToStr for FundraiserError {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + ToStr + TryFrom<u32>,
    {
        match self {
            FundraiserError::TooEarly => "Error: Contribution too early.",
            FundraiserError::TooLate => "Error: Contribution too late.",
            FundraiserError::MaxContribution => "Error: Max limit reached.",
            FundraiserError::MinContribution => "Error: Need to pass Min limit.",
            FundraiserError::TargetAmountRaised => "Error: Target amount is met.",
            FundraiserError::TargetAmountNotMet => "Error: Target amount is not met.",
        }
    }
}
