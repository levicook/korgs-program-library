use {
    crate::find_vault_v1_address,
    pinocchio_sol_vault_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum DepositV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWriteable,

    #[error("Vault must be writable")]
    VaultMustBeWriteable,

    #[error("Vault address mismatch: expected {expected:?}, observed {observed:?}")]
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("System program address mismatch: expected {expected:?}, observed {observed:?}")]
    SystemProgramAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct DepositV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub vault: AccountMeta,
    pub system_program: AccountMeta,
    pub amount: u64,
}

impl DepositV1Ix {
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
            system_program: AccountMeta {
                pubkey: solana_system_program::id(),
                is_signer: false,
                is_writable: false,
            },
            amount,
        }
    }

    /// Validates that the account metadata and addresses are correct.
    ///
    /// # Errors
    ///
    /// Returns [`DepositV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), DepositV1IxError> {
        if !self.owner.is_signer {
            return Err(DepositV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(DepositV1IxError::OwnerMustBeWriteable);
        }

        if !self.vault.is_writable {
            return Err(DepositV1IxError::VaultMustBeWriteable);
        }

        let expected_vault = find_vault_v1_address(&self.program_id, &self.owner.pubkey);
        let observed_vault = self.vault.pubkey;
        if observed_vault != expected_vault {
            return Err(DepositV1IxError::VaultAddressMismatch {
                expected: expected_vault,
                observed: observed_vault,
            });
        }

        let observed_system_program = self.system_program.pubkey;
        let expected_system_program = solana_system_program::id();
        if observed_system_program != expected_system_program {
            return Err(DepositV1IxError::SystemProgramAddressMismatch {
                expected: expected_system_program,
                observed: observed_system_program,
            });
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`DepositV1IxError`] if validation fails.
    pub fn build(self) -> Result<Instruction, DepositV1IxError> {
        self.validate()?;

        let mut instruction_data = vec![u8::from(InstructionDiscriminator::DepositV1)];
        instruction_data.extend_from_slice(&self.amount.to_le_bytes());

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.vault, self.system_program],
            data: instruction_data,
        })
    }
}
