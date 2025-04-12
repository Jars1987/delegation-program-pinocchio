use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{from_bytes, Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

use crate::{
    error::MyProgramError,
    state::{DataLen, Escrow},
};
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct DelegateAccountArgs {
    pub commit_frequency_ms: u32,
    pub seeds: Vec<Vec<u8>>,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateAccountArgs {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: vec![],
            validator: None,
        }
    }
}

pub const DELEGATION_ACCOUNT: Pubkey =
    pinocchio_pubkey::pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

pub fn process_delegate(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, pda_acc, magic_acc, buffer_acc, delegation_record, delegation_metadata, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //get buffer seeds
    let buffer_seeds: &[&[u8]] = &[b"buffer", pda_acc.key().as_ref()];
    let escrow_seeds = &["escrow".as_bytes(), maker.key().as_ref()];
    let delegation_rec_seeds = &[b"delegation", delegation_record.key().as_ref()];
    let delegation_met_seeds = &[b"delegation-metadata", delegation_metadata.key().as_ref()];

    //find pdas
    let (_, delegate_account_bump) = pubkey::find_program_address(escrow_seeds, &crate::ID);

    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, &crate::ID);

    //get signer seeds

    let bump = [delegate_account_bump];
    let seed_a = [
        Seed::from(b"escrow"),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump),
    ];
    let pda_signer_seeds = Signer::from(&seed_a);

    let bump = [buffer_pda_bump];
    let seed_b = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&bump),
    ];

    let buffer_signer_seeds = Signer::from(&seed_b);

    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: buffer_acc,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64, //PDA acc length
        owner: &crate::ID,
    }
    .invoke_signed(&[buffer_signer_seeds.clone()])?;

    // Copy the date to the buffer PDA
    let mut buffer_data = buffer_acc.try_borrow_mut_data()?;
    let new_data = pda_acc.try_borrow_data()?.to_vec().clone();
    (*buffer_data).copy_from_slice(&new_data);
    drop(buffer_data);

    //acc needs to be closed to be delagated

    //zeroed lamports
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0
    };

    //empty data
    pda_acc.realloc(0, false).unwrap();
    //send to System Program
    unsafe { pda_acc.assign(system_program.key()) };

    //we create account with Delegation Account
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: pda_acc,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64, //PDA acc length
        owner: &DELEGATION_ACCOUNT,
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    let account_metas = vec![
        AccountMeta::new(maker.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::readonly(&crate::ID),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    //args are the DelegateAccountArgs serialized

    //preprare delegate args
    let seeds_vec: Vec<Vec<u8>> = escrow_seeds.iter().map(|&slice| slice.to_vec()).collect();

    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: 30_000,
        seeds: seeds_vec,
        validator: None,
    };

    //seriliaze the args
    let mut data: Vec<u8> = vec![0u8; 8];
    let serialized_seeds = borsh::to_vec(&delegate_args)
    .map_err(|e| MyProgramError::SerializationFailed)?;
    data.extend_from_slice(&serialized_seeds);

    //call Instruction
    let instruction = Instruction {
        program_id: &DELEGATION_ACCOUNT,
        accounts: &account_metas,
        data: &data,
    };

    let acc_infos = [
        maker,
        pda_acc,
        magic_acc,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[pda_signer_seeds]);
    Ok(())
}
