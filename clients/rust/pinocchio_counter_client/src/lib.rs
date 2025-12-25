use pinocchio_counter_program::COUNTER_SEED;
use solana_pubkey::Pubkey;

pub mod instructions;
pub mod transactions;

#[must_use]
pub fn find_counter_address(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[COUNTER_SEED, owner.as_ref()];
    Pubkey::find_program_address(seeds, program_id)
}
