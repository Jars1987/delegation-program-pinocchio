pub mod delegate;
pub mod undelegate;

pub use delegate::*;
pub use undelegate::*;

use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum DelegateProgram {
    Delegate,
    Undelegate,
}

impl TryFrom<&u8> for DelegateProgram {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(DelegateProgram::Delegate),
            1 => Ok(DelegateProgram::Undelegate),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
