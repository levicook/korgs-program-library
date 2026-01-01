use {
    crate::find_vault_v1_address,
    pinocchio_sol_vault_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum WithdrawV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWriteable,

    #[error("Vault must be writable")]
    VaultMustBeWriteable,

    #[error("Vault address mismatch: expected {expected:?}, observed {observed:?}")]
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct WithdrawV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub vault: AccountMeta,
    pub amount: u64,
}

impl WithdrawV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, owner: Pubkey, amount: u64) -> Self {
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
            amount,
        }
    }

    /// Validates that the account metadata and addresses are correct.
    ///
    /// # Errors
    ///
    /// Returns [`WithdrawV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), WithdrawV1IxError> {
        if !self.owner.is_signer {
            return Err(WithdrawV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(WithdrawV1IxError::OwnerMustBeWriteable);
        }

        if !self.vault.is_writable {
            return Err(WithdrawV1IxError::VaultMustBeWriteable);
        }

        let expected_vault = find_vault_v1_address(&self.program_id, &self.owner.pubkey);
        let observed_vault = self.vault.pubkey;
        if observed_vault != expected_vault {
            return Err(WithdrawV1IxError::VaultAddressMismatch {
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
    /// Returns [`WithdrawV1IxError`] if validation fails.
    pub fn build(self) -> Result<Instruction, WithdrawV1IxError> {
        self.validate()?;

        let mut instruction_data = vec![u8::from(InstructionDiscriminator::WithdrawV1)];
        instruction_data.extend_from_slice(&self.amount.to_le_bytes());

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.vault],
            data: instruction_data,
        })
    }
}
