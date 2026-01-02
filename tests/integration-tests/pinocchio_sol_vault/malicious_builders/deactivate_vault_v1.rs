use {
    pinocchio_sol_vault_client::instructions::DeactivateVaultV1Ix,
    solana_hash::Hash,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_message::v0,
    solana_pubkey::Pubkey,
    solana_transaction::{versioned::VersionedTransaction, VersionedMessage},
};

/// Builder for creating malicious `DeactivateVaultV1` instructions.
#[derive(Debug, Clone)]
pub struct MaliciousDeactivateVaultV1Ix {
    program_id: Pubkey,
    owner: AccountMeta,
    vault: AccountMeta,
    instruction_data: Vec<u8>,
}

impl MaliciousDeactivateVaultV1Ix {
    /// Creates a new malicious instruction builder starting from a valid instruction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, owner: Pubkey) -> Self {
        let valid = DeactivateVaultV1Ix::new(program_id, owner);
        Self {
            program_id,
            owner: valid.owner,
            vault: valid.vault,
            instruction_data: vec![4u8], // DeactivateVaultV1 discriminator
        }
    }

    /// Sets the instruction discriminator to an invalid value.
    #[must_use]
    pub fn with_invalid_discriminator(mut self, discriminator: u8) -> Self {
        self.instruction_data = vec![discriminator];
        self
    }

    /// Sets the vault address to a random address.
    #[must_use]
    pub fn with_random_vault_address(mut self) -> Self {
        self.vault.pubkey = Pubkey::new_unique();
        self
    }

    /// Makes the owner not a signer.
    #[must_use]
    pub fn with_owner_not_signer(mut self) -> Self {
        self.owner.is_signer = false;
        self
    }

    /// Makes the vault not writable.
    #[must_use]
    pub fn with_vault_not_writable(mut self) -> Self {
        self.vault.is_writable = false;
        self
    }

    /// Builds the malicious instruction.
    #[must_use]
    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.vault],
            data: self.instruction_data,
        }
    }
}

/// Builder for creating malicious `DeactivateVaultV1` transactions.
#[derive(Debug)]
pub struct MaliciousDeactivateVaultV1Tx {
    program_id: Pubkey,
    owner_kp: Keypair,
    recent_blockhash: Hash,
    instruction: Instruction,
    signer: Option<Keypair>,
}

impl MaliciousDeactivateVaultV1Tx {
    /// Creates a new malicious transaction builder starting from a valid transaction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, owner_kp: Keypair, recent_blockhash: Hash) -> Self {
        let valid_ix = DeactivateVaultV1Ix::new(program_id, owner_kp.pubkey())
            .build()
            .unwrap();
        Self {
            program_id,
            owner_kp,
            recent_blockhash,
            instruction: valid_ix,
            signer: None,
        }
    }

    /// Sets a malicious instruction.
    #[must_use]
    pub fn with_malicious_instruction<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MaliciousDeactivateVaultV1Ix) -> MaliciousDeactivateVaultV1Ix,
    {
        let malicious_ix =
            MaliciousDeactivateVaultV1Ix::from_valid(self.program_id, self.owner_kp.pubkey());
        self.instruction = f(malicious_ix).build();
        self
    }

    /// Sets a different signer for the transaction.
    #[must_use]
    pub fn with_different_signer(mut self, signer: Keypair) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Builds the malicious transaction.
    ///
    /// # Panics
    ///
    /// Panics if transaction compilation or signing fails.
    pub fn build(self) -> VersionedTransaction {
        let signer = self.signer.as_ref().unwrap_or(&self.owner_kp);
        let owner_pk = signer.pubkey();

        let message = VersionedMessage::V0(
            v0::Message::try_compile(&owner_pk, &[self.instruction], &[], self.recent_blockhash)
                .expect("Failed to compile message"),
        );

        VersionedTransaction::try_new(message, &[signer.insecure_clone()])
            .expect("Failed to create transaction")
    }
}
