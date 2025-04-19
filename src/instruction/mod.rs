pub mod commit;
pub mod commit_and_undelegate;
pub mod delegate;
pub mod undelegate;

pub use commit::*;
pub use commit_and_undelegate::*;
pub use delegate::*;
pub use undelegate::*;

use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum DelegateProgram {
    Delegate,
    Undelegate,
    CommitAccounts,
    CommitAndUndelegateAccounts,
}

impl TryFrom<&u8> for DelegateProgram {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(DelegateProgram::Delegate),
            1 => Ok(DelegateProgram::Undelegate),
            2 => Ok(DelegateProgram::CommitAccounts),
            3 => Ok(DelegateProgram::CommitAndUndelegateAccounts),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
