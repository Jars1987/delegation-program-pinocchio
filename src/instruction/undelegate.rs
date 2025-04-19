use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::utils::{deserialize_undelegate_ix_data, get_seeds};

pub fn process_undelegate(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer, delegated_acc, owner_program, buffer_acc, _system_program, _rest @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !buffer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //according to the SDK they receive a Vec<Vec<u8>> this is not possible here as IX comes in &[u8]
    let seeds_data = deserialize_undelegate_ix_data(data)?;

    //get buffer seeds
    let delegate_pda_seeds: Vec<&[u8]> = seeds_data.iter().map(|s| s.as_slice()).collect();

    //Find delegate
    let (_, delegate_account_bump) = pubkey::find_program_address(&delegate_pda_seeds, &crate::ID);

    //Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let mut delegate_seeds = get_seeds(delegate_pda_seeds)?;
    delegate_seeds.extend_from_slice(&[delegate_bump]);
    let delegate_signer_seeds = Signer::from(delegate_seeds.as_slice());

    //we create the original PDA Account Delegated
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: delegated_acc,
        lamports: Rent::get()?.minimum_balance(buffer_acc.data_len()),
        space: buffer_acc.data_len() as u64, //PDA acc length
        owner: &owner_program.key(),
    }
    .invoke_signed(&[delegate_signer_seeds.clone()])?;

    let mut data = delegated_acc.try_borrow_mut_data()?;
    let buffer_data = buffer_acc.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);

    //they don't close the buffer but shouldn't it be closed?

    Ok(())
}
