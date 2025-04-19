//#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

mod consts;
mod error;
mod instruction;
mod tests;
mod types;
mod utils;

pinocchio_pubkey::declare_id!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");
