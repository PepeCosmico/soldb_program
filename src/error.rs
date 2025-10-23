use solana_program::program_error::ProgramError;
use solana_program_error::ToStr;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SolDbError {
    WrongOwner = 0,
    PdaMismatch = 1,
    NotTable = 2,
    GrowthTooLarge = 3,
    WrongError = 4,
}

impl From<SolDbError> for ProgramError {
    fn from(value: SolDbError) -> Self {
        ProgramError::Custom(value as u32)
    }
}

impl TryFrom<u32> for SolDbError {
    type Error = Self;
    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::WrongOwner,
            1 => Self::PdaMismatch,
            2 => Self::NotTable,
            3 => Self::GrowthTooLarge,
            _ => Self::WrongError,
        })
    }
}

impl ToStr for SolDbError {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + ToStr + TryFrom<u32>,
    {
        match self {
            Self::WrongOwner => "Error: Account is not owned by this program",
            Self::PdaMismatch => "Error: PDA Account is not as expected",
            Self::NotTable => "Error: Not a SolTable Account",
            Self::GrowthTooLarge => {
                "Error: The growth of the account has exceeded the maximum of 10KB"
            }
            Self::WrongError => "Error: Wrong error value",
        }
    }
}

pub type Result<T> = std::result::Result<T, ProgramError>;
