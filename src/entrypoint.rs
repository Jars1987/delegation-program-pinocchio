use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instruction::{self, MyProgramInstrution};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the default panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix_disc, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match MyProgramInstrution::try_from(ix_disc)? {
        MyProgramInstrution::Make => instruction::process_make(accounts, instruction_data),
        MyProgramInstrution::Take => instruction::process_take(accounts),
        MyProgramInstrution::Refund => instruction::process_refund(accounts),
        MyProgramInstrution::Delegate => instruction::process_delegate(accounts),
    }
}
