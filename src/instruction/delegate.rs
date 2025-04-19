use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{
    consts::{BUFFER, DELEGATION_PROGRAM_ID},
    state::{close_pda_acc, cpi_delegate, deserialize_ix_data, get_seeds, DelegateAccountArgs},
};

pub fn process_delegate(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata, system_program, _rest @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (seeds_data, config) = deserialize_ix_data(data)?;

    //Clone is unavoidable fore now
    let delegate_pda_seeds = seeds_data.clone();

    //get buffer seeds
    let buffer_seeds: &[&[u8]] = &[BUFFER, pda_acc.key().as_ref()];
    let pda_seeds: Vec<&[u8]> = seeds_data.iter().map(|s| s.as_slice()).collect();

    //Find pdas
    let (_, delegate_account_bump) = pubkey::find_program_address(&pda_seeds, &crate::ID);
    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, &crate::ID);

    //Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let mut delegate_seeds = get_seeds(pda_seeds)?;
    delegate_seeds.extend_from_slice(&[delegate_bump]);
    let delegate_signer_seeds = Signer::from(delegate_seeds.as_slice());

    //Get Buffer signer seeds
    let bump = [buffer_pda_bump];
    let seed_b = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&bump),
    ];

    let buffer_signer_seeds = Signer::from(&seed_b);

    //Create Buffer PDA account
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: Rent::get()?.minimum_balance(pda_acc.data_len()),
        space: pda_acc.data_len() as u64, //PDA acc length
        owner: &crate::ID,
    }
    .invoke_signed(&[buffer_signer_seeds.clone()])?;

    // Copy the date to the buffer PDA
    let mut buffer_data = buffer_acc.try_borrow_mut_data()?;
    let new_data = pda_acc.try_borrow_data()?.to_vec().clone();
    (*buffer_data).copy_from_slice(&new_data);
    drop(buffer_data);

    //Close Delegate PDA in preparation for CPI Delegate
    close_pda_acc(payer, pda_acc, system_program)?;

    //we create account with Delegation Account
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: pda_acc,
        lamports: Rent::get()?.minimum_balance(pda_acc.data_len()),
        space: pda_acc.data_len() as u64, //PDA acc length
        owner: &DELEGATION_PROGRAM_ID,
    }
    .invoke_signed(&[delegate_signer_seeds.clone()])?;

    //preprare delegate args
    //struct DelegateConfig comes from IX data
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds: delegate_pda_seeds,
        validator: config.validator,
    };

    cpi_delegate(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        delegate_signer_seeds,
    )?;

    close_pda_acc(payer, buffer_acc, system_program)?;

    Ok(())
}
