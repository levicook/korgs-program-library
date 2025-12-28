use {pinocchio_counter_program::COUNTER_V1_SEED, solana_pubkey::Pubkey};

pub mod instructions;
pub mod transactions;

#[must_use]
pub fn find_counter_v1_address(program_id: &Pubkey, owner: &Pubkey) -> Pubkey {
    find_counter_v1(program_id, owner).0
}

#[must_use]
pub fn find_counter_v1(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[COUNTER_V1_SEED, owner.as_ref()];
    Pubkey::find_program_address(seeds, program_id)
}
