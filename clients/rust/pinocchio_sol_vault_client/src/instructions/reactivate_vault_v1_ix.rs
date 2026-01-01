use {
    crate::find_vault_v1_address,
    pinocchio_sol_vault_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum ReactivateVaultV1IxError {
    #[error("Payer must be a signer")]
    PayerMustBeSigner,

    #[error("Payer must be writable")]
    PayerMustBeWriteable,

    #[error("Vault must be writable")]
    VaultMustBeWriteable,

    #[error("Vault address mismatch: expected {expected:?}, observed {observed:?}")]
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("System program address mismatch: expected {expected:?}, observed {observed:?}")]
    SystemProgramAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct ReactivateVaultV1Ix {
    pub program_id: Pubkey,
    pub payer: AccountMeta,
    pub vault: AccountMeta,
    pub system_program: AccountMeta,
}

impl ReactivateVaultV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, payer: Pubkey) -> Self {
        let vault = find_vault_v1_address(&program_id, &payer);

        Self {
            program_id,
            payer: AccountMeta {
                pubkey: payer,
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
        }
    }

    /// Validates that the account metadata and addresses are correct.
    ///
    /// # Errors
    ///
    /// Returns [`ReactivateVaultV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), ReactivateVaultV1IxError> {
        if !self.payer.is_signer {
            return Err(ReactivateVaultV1IxError::PayerMustBeSigner);
        }

        if !self.payer.is_writable {
            return Err(ReactivateVaultV1IxError::PayerMustBeWriteable);
        }

        if !self.vault.is_writable {
            return Err(ReactivateVaultV1IxError::VaultMustBeWriteable);
        }

        let expected_vault = find_vault_v1_address(&self.program_id, &self.payer.pubkey);
        let observed_vault = self.vault.pubkey;
        if observed_vault != expected_vault {
            return Err(ReactivateVaultV1IxError::VaultAddressMismatch {
                expected: expected_vault,
                observed: observed_vault,
            });
        }

        let observed_system_program = self.system_program.pubkey;
        let expected_system_program = solana_system_program::id();
        if observed_system_program != expected_system_program {
            return Err(ReactivateVaultV1IxError::SystemProgramAddressMismatch {
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
    /// Returns [`ReactivateVaultV1IxError`] if validation fails.
    pub fn build(self) -> Result<Instruction, ReactivateVaultV1IxError> {
        self.validate()?;

        let instruction_data = [u8::from(InstructionDiscriminator::ReactivateVaultV1)];

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.payer, self.vault, self.system_program],
            data: instruction_data.to_vec(),
        })
    }
}
