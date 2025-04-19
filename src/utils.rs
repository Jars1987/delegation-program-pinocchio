use borsh::BorshDeserialize;
use bytemuck::from_bytes;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::PUBKEY_BYTES,
};

use crate::{
    consts::DELEGATION_PROGRAM_ID,
    error::MyProgramError,
    types::{DelegateAccountArgs, DelegateConfig},
};

//helper to deserialize using bytemuck
pub fn _parse_delegate_config(data: &[u8]) -> Result<DelegateConfig, ProgramError> {
    if data.len() < 4 {
        return Err(MyProgramError::SerializationFailed.into());
    }

    let commit_frequency_ms = *from_bytes::<u32>(&data[..4]);

    let validator = if data.len() >= 36 {
        Some(data[4..36].try_into().unwrap())
    } else {
        None
    };

    Ok(DelegateConfig {
        commit_frequency_ms,
        validator,
    })
}

//helper to serialize using bytemuck (providing slice length descriminators)
pub fn serialize_delegate_account_args(args: &DelegateAccountArgs) -> Vec<u8> {
    let mut data = Vec::new();

    // Serialize commit_frequency_ms (4 bytes)
    data.extend_from_slice(&args.commit_frequency_ms.to_le_bytes());

    // Serialize seeds (Vec<Vec<u8>>)
    // First, serialize the number of seeds (as a u8)
    let num_seeds = args.seeds.len() as u8;
    data.extend_from_slice(&num_seeds.to_le_bytes());

    // Then, serialize each seed (each &[u8])
    for seed in &args.seeds {
        let seed_len = seed.len() as u32;
        data.extend_from_slice(&seed_len.to_le_bytes()); // Seed length
        data.extend_from_slice(&seed); // Seed content
    }

    // Serialize validator (32 bytes)
    if let Some(pubkey) = args.validator {
        data.extend_from_slice(&pubkey);
    }
    //if they use a u8 to check if it is Some or None we need to extend_from_slice that byte

    data
}

//Deserialize data using borsh and some assumptions
//we need the array length descriminator and another
//descriminator for the length of the inner arrays
pub fn deserialize_delegate_ix_data(
    ix_data: &[u8],
) -> Result<(Vec<Vec<u8>>, DelegateConfig), ProgramError> {
    let mut offset = 0;

    // First byte provides total number of seeds
    if ix_data.len() < 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let num_seeds = ix_data[0] as usize;
    offset += 1;

    // Extract the seeds
    let mut seeds = Vec::with_capacity(num_seeds);

    for _ in 0..num_seeds {
        if ix_data.len() < offset + 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        //first byte is out seed length
        let seed_len = ix_data[offset] as usize;
        offset += 1;

        let seed = ix_data[offset..offset + seed_len].to_vec();
        seeds.push(seed);
        offset += seed_len;
    }

    // Borsh Deserialize DelegateConfig (we might change this to bytemuck see parse_delegate_config)
    let config = DelegateConfig::try_from_slice(&ix_data[offset..])
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    Ok((seeds, config))
}

pub fn deserialize_undelegate_ix_data(ix_data: &[u8]) -> Result<Vec<Vec<u8>>, ProgramError> {
    let mut offset = 0;

    // First byte provides total number of seeds
    if ix_data.len() < 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let num_seeds = ix_data[0] as usize;
    offset += 1;

    // Extract the seeds
    let mut seeds = Vec::with_capacity(num_seeds);

    for _ in 0..num_seeds {
        if ix_data.len() < offset + 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        //first byte is out seed length
        let seed_len = ix_data[offset] as usize;
        offset += 1;

        let seed = ix_data[offset..offset + seed_len].to_vec();
        seeds.push(seed);
        offset += seed_len;
    }

    Ok(seeds)
}

#[inline(always)]
pub fn get_seeds<'a>(seeds_vec: Vec<&'a [u8]>) -> Result<Vec<Seed<'a>>, ProgramError> {
    let mut seeds: Vec<Seed<'a>> = Vec::with_capacity(seeds_vec.len() + 1);

    // Add the regular seeds from the provided slice
    for seed in seeds_vec {
        seeds.push(Seed::from(seed));
    }

    Ok(seeds)
}

pub fn close_pda_acc(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    // Step 1 - Lamports to zero
    unsafe {
        *payer.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }

    // Step 2 - Empty the data
    pda_acc.realloc(0, false).unwrap();

    // Step 3 - Send to System Program
    unsafe { pda_acc.assign(system_program.key()) };

    Ok(())
}

pub fn cpi_delegate(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    owner_program: &AccountInfo,
    buffer_acc: &AccountInfo,
    delegation_record: &AccountInfo,
    delegation_metadata: &AccountInfo,
    system_program: &AccountInfo,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    let account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::readonly(owner_program.key()),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let data: Vec<u8> = serialize_delegate_account_args(&delegate_args);

    //call Instruction
    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: &account_metas,
        data: &data,
    };

    let acc_infos = [
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[signer_seeds])?;
    Ok(())
}

pub struct CommitIx<'a> {
    pub program_id: &'a [u8; PUBKEY_BYTES],
    pub data: Vec<u8>,
    pub accounts: Vec<AccountMeta<'a>>,
}

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountInfo,
    account_infos: &'a [AccountInfo],
    magic_context: &'a AccountInfo,
    magic_program: &'a AccountInfo,
    allow_undelegation: bool,
) -> CommitIx<'a> {
    let instruction_data: Vec<u8> = if allow_undelegation {
        vec![2, 0, 0, 0]
    } else {
        vec![1, 0, 0, 0]
    };
    let mut account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(magic_context.key(), true, false),
    ];
    account_metas.extend(
        account_infos
            .iter()
            .map(|acc| AccountMeta::new(acc.key(), true, true)),
    );
    let instruction = CommitIx {
        program_id: magic_program.key(),
        data: instruction_data,
        accounts: account_metas,
    };
    instruction
}
