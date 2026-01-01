use {
    crate::find_vault_v1_address,
    pinocchio_sol_vault_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum DeactivateVaultV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWriteable,

    #[error("Vault must be writable")]
    VaultMustBeWriteable,

    #[error("Vault address mismatch: expected {expected:?}, observed {observed:?}")]
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct DeactivateVaultV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub vault: AccountMeta,
}

impl DeactivateVaultV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, owner: Pubkey) -> Self {
        let vault = find_vault_v1_address(&program_id, &owner);

        Self {
            program_id,
            owner: AccountMeta {
                pubkey: owner,
                is_signer: true,
                is_writable: true,
            },
            vault: AccountMeta {
                pubkey: vault,
                is_signer: false,
                is_writable: true,
            },
        }
    }

    /// Validates that the account metadata and addresses are correct.
    ///
    /// # Errors
    ///
    /// Returns [`DeactivateVaultV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), DeactivateVaultV1IxError> {
        if !self.owner.is_signer {
            return Err(DeactivateVaultV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(DeactivateVaultV1IxError::OwnerMustBeWriteable);
        }

        if !self.vault.is_writable {
            return Err(DeactivateVaultV1IxError::VaultMustBeWriteable);
        }

        let expected_vault = find_vault_v1_address(&self.program_id, &self.owner.pubkey);
        let observed_vault = self.vault.pubkey;
        if observed_vault != expected_vault {
            return Err(DeactivateVaultV1IxError::VaultAddressMismatch {
                expected: expected_vault,
                observed: observed_vault,
            });
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`DeactivateVaultV1IxError`] if validation fails.
    pub fn build(self) -> Result<Instruction, DeactivateVaultV1IxError> {
        self.validate()?;

        let instruction_data = [u8::from(InstructionDiscriminator::DeactivateVaultV1)];

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.vault],
            data: instruction_data.to_vec(),
        })
    }
}
