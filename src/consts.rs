// NOTE: this should go into a core package that both the sdk + the program can depend on
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

/// The delegation program ID.
pub const DELEGATION_PROGRAM_ID: Pubkey = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

/// The seed of the buffer account PDA.
pub const BUFFER: &[u8] = b"buffer";
