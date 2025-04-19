#[cfg(test)]
mod tests {

    #![no_std]
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;
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
        sysvar::Sysvar,
    };
    use spl_token::state::AccountState;

    const ID: Pubkey = pubkey!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");
}
