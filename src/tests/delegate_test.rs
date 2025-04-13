#[cfg(test)]
mod delegate_tests {
    use crate::instruction::DELEGATION_ACCOUNT;
    use mollusk_svm::{program, result::Check, Mollusk};
    use pinocchio_log::log;
    use solana_sdk::{
        account::{Account, AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_option::COption,
        program_pack::Pack,
        pubkey,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar
    };
    use spl_token::state::AccountState;

    const ID: Pubkey = pubkey!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");

    #[test]
    fn test_delegate() {
        // Initialize Mollusk test environment
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        // Setup system program
        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        // Add the SPL token program
        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        // Create maker account with some lamports
        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Find PDA for escrow
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes()],
            &ID,
        );
        log!("Escrow bump: {}", escrow_bump);

        // Create escrow account with Escrow data
        let mut escrow_account = Account::new(
            mollusk.sysvars.rent.minimum_balance(65),  // Assuming Escrow::LEN is around 65 bytes
            65,  // Size of Escrow struct
            &ID,
        );

        // Create mints for token A and B
        let mint_a = Pubkey::new_from_array([0x03; 32]);
        let mut mint_a_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_a_account.data_as_mut_slice(),
        )
        .unwrap();

        let mint_b = Pubkey::new_from_array([0x04; 32]);
        let mut mint_b_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_b_account.data_as_mut_slice(),
        )
        .unwrap();

        // Initialize escrow data
        // This simulates that an escrow has already been created
        let escrow_data = [
            // maker pubkey (32 bytes)
            maker.to_bytes().to_vec(),
            // mint_a pubkey (32 bytes)
            mint_a.to_bytes().to_vec(),
            // mint_b pubkey (32 bytes)
            mint_b.to_bytes().to_vec(),
            // amount (8 bytes)
            1_000_000u64.to_le_bytes().to_vec(),
            // bump (1 byte)
            vec![escrow_bump],
        ]
        .concat();
        
        escrow_account.data_as_mut_slice()[..escrow_data.len()].copy_from_slice(&escrow_data);

        // Create buffer account (will be created during the instruction)
        let (buffer, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"buffer"), &escrow.to_bytes()],
            &ID,
        );
        let buffer_account = Account::new(0, 0, &system_program);

        // Create magic account (this is the program account)
        let magic_account = Account::new(0, 0, &ID);

        // Create delegation record and metadata accounts
        let delegation_record = Pubkey::new_from_array([0x07; 32]);
        let delegation_record_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        let delegation_metadata = Pubkey::new_from_array([0x08; 32]);
        let delegation_metadata_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Create instruction data with delegate instruction discriminator (3)
        let data = vec![3u8];

        // Create the instruction
        let instruction = Instruction::new_with_bytes(
            ID,
            &data,
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new(escrow, false),
                AccountMeta::new_readonly(ID, false),  // magic account
                AccountMeta::new(buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new_readonly(delegation_metadata, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        // Process the instruction
        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account),
                (escrow, escrow_account),
                (ID, magic_account),
                (buffer, buffer_account),
                (delegation_record, delegation_record_account),
                (delegation_metadata, delegation_metadata_account),
                (system_program, system_account),
            ],
            &[Check::success()],
        );

        // Verify the results
        let accounts = mollusk.get_accounts();

        // 1. Check that the escrow account is now owned by DELEGATION_ACCOUNT
        let escrow_account = accounts.get(&escrow).unwrap();
        assert_eq!(escrow_account.owner(), &DELEGATION_ACCOUNT, "Escrow account should be owned by DELEGATION_ACCOUNT");

        // 2. Check that the buffer account has the escrow data
        let buffer_account = accounts.get(&buffer).unwrap();
        assert_eq!(buffer_account.data().len(), 65, "Buffer account should have the escrow data");

        // 3. Check that the escrow account has been properly delegated
        // Note: Since we can't actually verify the delegation logic which happens in the DELEGATION_ACCOUNT program,
        // we're just checking that our program successfully called it and the accounts are set up correctly
    }
}
