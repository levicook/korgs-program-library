#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

mod error;
mod instructions;
mod state;

pub use error::{CounterError, CounterResult};
pub use instructions::{CreateCounterV1, InstructionDiscriminator};
use pinocchio::pubkey::{find_program_address, Pubkey};
pub use state::{AccountDiscriminator, CounterV1};

pub(crate) const COUNTER_SEED: &[u8] = b"counter";

pub(crate) fn find_counter_address(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[COUNTER_SEED, owner.as_ref()];
    find_program_address(seeds, program_id)
}
