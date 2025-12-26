#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

mod error;
mod instructions;
mod state;

use pinocchio::pubkey::{find_program_address, Pubkey};
pub use {
    error::{CounterError, CounterResult},
    instructions::{
        DeactivateCounterV1, DecrementCountV1, IncrementCountV1, InitializeCounterV1,
        InstructionDiscriminator, SetCountV1, SetCountV1Args,
    },
    state::{AccountDiscriminator, CounterV1, DEACTIVATED_ACCOUNT_SIZE},
};

pub const COUNTER_SEED: &[u8] = b"counter";

pub(crate) fn find_counter_address(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[COUNTER_SEED, owner.as_ref()];
    find_program_address(seeds, program_id)
}
