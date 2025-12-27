#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

mod error;
mod instructions;
mod state;

use pinocchio::pubkey::{try_find_program_address, Pubkey};
pub use {
    error::{CounterError, CounterResult},
    instructions::{
        DeactivateCounterV1, DecrementCountV1, IncrementCountV1, InitializeCounterV1,
        InstructionDiscriminator, SetCountV1, SetCountV1Args,
    },
    state::{AccountDiscriminator, CounterV1, DEACTIVATED_ACCOUNT_SIZE},
};

pub const COUNTER_SEED: &[u8] = b"counter";

/// Finds the program-derived address for a counter account.
///
/// Returns the address and bump seed used to derive it.
///
/// # Panics
///
/// Panics if a viable program address bump seed cannot be found. This is
/// statistically very unlikely in practice.
pub fn find_counter_address(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    try_find_counter_address(program_id, owner)
        .expect("Unable to find a viable program address bump seed")
}

/// Tries to find the program-derived address for a counter account.
///
/// Returns the address and bump seed used to derive it, or `None` if a viable
/// program address bump seed cannot be found (statistically very unlikely).
pub fn try_find_counter_address(program_id: &Pubkey, owner: &Pubkey) -> Option<(Pubkey, u8)> {
    let seeds = &[COUNTER_SEED, owner.as_ref()];
    try_find_program_address(seeds, program_id)
}
