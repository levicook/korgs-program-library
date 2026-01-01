use {pinocchio_sol_vault_program::VAULT_V1_SEED, solana_pubkey::Pubkey};

pub mod instructions;
pub mod transactions;

#[must_use]
pub fn find_vault_v1_address(program_id: &Pubkey, owner: &Pubkey) -> Pubkey {
    find_vault_v1(program_id, owner).0
}

#[must_use]
pub fn find_vault_v1(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[VAULT_V1_SEED, owner.as_ref()];
    Pubkey::find_program_address(seeds, program_id)
}
