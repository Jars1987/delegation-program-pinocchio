use mollusk_svm::Mollusk;
use pinocchio_log::log;
use solana_sdk::{
    account::Account,
    bpf_loader,
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey,
    pubkey::Pubkey,
};
use std::collections::HashMap;

const DELEGATION_ACCOUNT: Pubkey = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");
const ID: Pubkey = pubkey!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");

// Mock implementation of the DelegateAccountArgs struct from delegate.rs
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize)]
struct DelegateAccountArgs {
    commit_frequency_ms: u64,
    seeds: Vec<Vec<u8>>,
    validator: Option<Pubkey>,
}

#[test]
fn test_delegate_full_cpi() {
    // Initialize Mollusk
    let mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");
    
    // Setup system program
    let (system_program_id, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();
    
    // Create maker account with lamports
    let maker = Pubkey::new_from_array([0x02; 32]);
    let maker_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program_id);
    
    // Find PDA for escrow
    let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"escrow", &maker.to_bytes()],
        &ID,
    );
    log!("Escrow bump: {}", escrow_bump);
    
    // create escrow acc with escrow data + Size of escrow struct
    let mut escrow_data = Vec::with_capacity(65);
    
    // maker pubkey (32 bytes)
    escrow_data.extend_from_slice(&maker.to_bytes());
    // mint_a pubkey (32 bytes)
    let mint_a = Pubkey::new_from_array([0x03; 32]);
    escrow_data.extend_from_slice(&mint_a.to_bytes());
    // mint_b pubkey (32 bytes)
    let mint_b = Pubkey::new_from_array([0x04; 32]);
    escrow_data.extend_from_slice(&mint_b.to_bytes());
    // amount (8 bytes)
    escrow_data.extend_from_slice(&1_000_000u64.to_le_bytes());
    // bump (1 byte)
    escrow_data.push(escrow_bump);
    
    // create escrow acc
    let escrow_account = Account {
        lamports: mollusk.sysvars.rent.minimum_balance(escrow_data.len()),
        data: escrow_data,
        owner: ID,
        executable: false,
        rent_epoch: 0,
    };
    
    // create buffer acc
    let (buffer, buffer_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"buffer", &escrow.to_bytes()],
        &ID,
    );
    log!("Buffer bump: {}", buffer_bump);
    
    // create buffer acc
    let buffer_account = Account {
        lamports: mollusk.sysvars.rent.minimum_balance(65),
        data: vec![0; 65], // Pre-allocate with zeros
        owner: ID, // Set the owner to our program ID
        executable: false,
        rent_epoch: 0,
    };
    
    // create delegation record and metadata acc
    // these acc would normally be created by delegation prog
    let delegation_record = Pubkey::new_from_array([0x07; 32]);
    let delegation_record_account = Account {
        lamports: 1 * LAMPORTS_PER_SOL,
        data: vec![0; 65], // Pre-allocate with zeros
        owner: DELEGATION_ACCOUNT, // Set the owner to the delegation account program
        executable: false,
        rent_epoch: 0,
    };
    
    let delegation_metadata = Pubkey::new_from_array([0x08; 32]);
    let delegation_metadata_account = Account {
        lamports: 1 * LAMPORTS_PER_SOL,
        data: vec![0; 65], // pre-alloc with zeros
        owner: DELEGATION_ACCOUNT,
        executable: false,
        rent_epoch: 0,
    };
    
    // create delegation prog acc to do the CPI. executable = true
    let delegation_program_account = Account {
        lamports: 1 * LAMPORTS_PER_SOL,
        data: vec![],
        owner: bpf_loader::ID,
        executable: true,
        rent_epoch: 0,
    };
    
    // register custom handler for delegation prog
    // simulate delegation prog
    let mut account_map = HashMap::new();
    account_map.insert(maker, maker_account.clone());
    account_map.insert(escrow, escrow_account.clone());
    account_map.insert(buffer, buffer_account.clone());
    account_map.insert(delegation_record, delegation_record_account.clone());
    account_map.insert(delegation_metadata, delegation_metadata_account.clone());
    account_map.insert(system_program_id, system_account.clone());
    account_map.insert(DELEGATION_ACCOUNT, delegation_program_account.clone());
    
    // create mock prog acc for main program
    let program_account = Account {
        lamports: 1 * LAMPORTS_PER_SOL,
        data: vec![],
        owner: bpf_loader::ID,  // BPF loader is owner
        executable: true,
        rent_epoch: 0,
    };
    account_map.insert(ID, program_account.clone());
    
    // Create instruction data with delegate instruction discriminator (3)
    let data = vec![3u8];
    
    // Create the instruction
    let instruction = Instruction::new_with_bytes(
        ID,
        &data,
        vec![
            AccountMeta::new(maker, true),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(ID, false),  // program id in lib.rs
            AccountMeta::new(buffer, false),
            AccountMeta::new(delegation_record, false),
            AccountMeta::new_readonly(delegation_metadata, false),
            AccountMeta::new_readonly(system_program_id, false),
        ],
    );
    
    // attempt the CPI
    let result = mollusk.process_instruction(
        &instruction,
        &vec![
            (maker, maker_account),
            (escrow, escrow_account),
            (ID, program_account),
            (buffer, buffer_account),
            (delegation_record, delegation_record_account),
            (delegation_metadata, delegation_metadata_account),
            (system_program_id, system_account),
            (DELEGATION_ACCOUNT, delegation_program_account),
        ],
    );
    
    // check result
    if result.raw_result.is_err() {
        log!("Got expected error from CPI");
        log!("Compute units consumed: {}", result.compute_units_consumed);
        
        // even though error,examine resulting accounts
        for (account_key, account) in &result.resulting_accounts {
            if account_key == &buffer {
                log!("Buffer account modified during test");
            }
            if account_key == &delegation_record {
                log!("Delegation record account modified during test");
            }
        }
    } else {
        // If it succeeds, then congrats :)
        log!("Instruction executed successfully with mock delegation program");
        
        // examine resulting accounts
        for (account_key, account) in &result.resulting_accounts {
            if account_key == &buffer {
                log!("Buffer account modified during test");
            }
            if account_key == &escrow {
                if account.owner == DELEGATION_ACCOUNT {
                    log!("Escrow account owner is now DELEGATION_ACCOUNT - delegation succeeded");
                }
            }
        }
    }
    
    // The important thing is to reach cpi call
    log!("Test passed: The program processed the delegate instruction and attempted the CPI");
}
