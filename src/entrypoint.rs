use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instruction::{self, DelegateProgram};

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

    match DelegateProgram::try_from(ix_disc)? {
        DelegateProgram::Delegate => instruction::process_delegate(accounts, instruction_data),
        DelegateProgram::Undelegate => instruction::process_undelegate(accounts, instruction_data),
        DelegateProgram::CommitAccounts => instruction::process_commit_accounts(accounts),
        DelegateProgram::CommitAndUndelegateAccounts => {
            instruction::process_commit_and_undelegate_accounts(accounts)
        }
    }
}
