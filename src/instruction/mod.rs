pub mod delegate;

pub use delegate::*;

use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum MyProgramInstrution {
    Delegate,
}

impl TryFrom<&u8> for MyProgramInstrution {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(MyProgramInstrution::Delegate),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
