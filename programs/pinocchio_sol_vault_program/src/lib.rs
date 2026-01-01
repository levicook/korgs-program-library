#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

mod account_discriminator;
mod error;
mod instructions;
mod instructions_discriminator;
mod state;

use pinocchio::pubkey::{try_find_program_address, Pubkey};
pub use {
    account_discriminator::{AccountDiscriminator, AccountDiscriminatorError},
    error::{InstructionError, InstructionResult},
    instructions::{
        DeactivateVaultV1, DepositV1, InitializeVaultV1, ReactivateVaultV1, WithdrawV1,
        DEACTIVATED_ACCOUNT_SIZE,
    },
    instructions_discriminator::{InstructionDiscriminator, InstructionDiscriminatorError},
    state::VaultV1,
};

pub const VAULT_V1_SEED: &[u8] = b"vault_v1";

/// Finds the program-derived address for a vault account.
///
/// The address is derived using `[VAULT_V1_SEED, owner]` as seeds. This design ensures:
///
/// - **Ownership isolation**: Each owner has a unique, deterministic vault account address
/// - **Security**: Vault accounts cannot be reused or shared across different owners
/// - **Cryptographic binding**: The owner's identity is cryptographically bound to the vault address
///
/// The owner's public key is included in the seed to prevent address collisions and ensure
/// that each user has their own independent vault state.
///
/// Returns the address and bump seed used to derive it.
///
/// # Panics
///
/// Panics if a viable program address bump seed cannot be found. This is
/// statistically very unlikely in practice.
pub fn find_vault_v1(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    try_find_vault_v1(program_id, owner).expect("Unable to find a viable program address bump seed")
}

/// Tries to find the program-derived address for a vault account.
///
/// This is a fallible version of [`find_vault_v1`] that returns `None` instead of panicking
/// if no viable bump seed can be found. See [`find_vault_v1`] for details on the seed derivation.
///
/// Returns the address and bump seed used to derive it, or `None` if a viable
/// program address bump seed cannot be found (statistically very unlikely).
pub fn try_find_vault_v1(program_id: &Pubkey, owner: &Pubkey) -> Option<(Pubkey, u8)> {
    let seeds = &[VAULT_V1_SEED, owner.as_ref()];
    try_find_program_address(seeds, program_id)
}
