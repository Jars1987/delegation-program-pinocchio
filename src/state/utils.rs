use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::from_bytes;
use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{consts::DELEGATION_PROGRAM_ID, error::MyProgramError};

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

#[inline(always)]
pub fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_acc_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

#[inline(always)]
pub fn load_acc_mut<T: DataLen + Initialized>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    load_acc_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub fn load_acc_mut_unchecked<T: DataLen>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut T) })
}

#[inline(always)]
pub fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(MyProgramError::InvalidInstructionData.into());
    }
    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

pub fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(data as *const T as *const u8, T::LEN) }
}

pub fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN) }
}

//Create close_pda, close_pda_with_system_transfer, create_pda, seeds_with_bump

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

//why do need lifetimes here?
pub struct DelegateAccounts<'a> {
    pub payer: &'a AccountInfo,
    pub pda: &'a AccountInfo,
    pub owner_program: &'a AccountInfo,
    pub buffer: &'a AccountInfo,
    pub delegation_record: &'a AccountInfo,
    pub delegation_metadata: &'a AccountInfo,
    pub delegation_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DelegateConfig {
    pub commit_frequency_ms: u32,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        DelegateConfig {
            commit_frequency_ms: DelegateAccountArgs::default().commit_frequency_ms,
            validator: DelegateAccountArgs::default().validator,
        }
    }
}

//helper to deserialize using bytemuck
pub fn parse_delegate_config(data: &[u8]) -> Result<DelegateConfig, ProgramError> {
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

/// Seeds with bump
#[inline(always)]
pub fn seeds_with_bump<'a>(seeds: &'a [&'a [u8]], bump: &'a [u8]) -> Vec<&'a [u8]> {
    let mut combined: Vec<&'a [u8]> = Vec::with_capacity(seeds.len() + 1);
    combined.extend_from_slice(seeds);
    combined.push(bump);
    combined
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
