use {
    pinocchio_sol_vault_client::instructions::InitializeVaultV1Ix,
    solana_hash::Hash,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_message::v0,
    solana_pubkey::Pubkey,
    solana_transaction::{versioned::VersionedTransaction, VersionedMessage},
};

/// Builder for creating malicious `InitializeVaultV1` instructions.
///
/// This builder allows you to start from a valid instruction and mutate
/// specific properties to test security boundaries.
#[derive(Debug, Clone)]
pub struct MaliciousInitializeVaultV1Ix {
    program_id: Pubkey,
    payer: AccountMeta,
    vault: AccountMeta,
    system_program: AccountMeta,
    instruction_data: Vec<u8>,
}

impl MaliciousInitializeVaultV1Ix {
    /// Creates a new malicious instruction builder starting from a valid instruction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, payer: Pubkey) -> Self {
        let valid = InitializeVaultV1Ix::new(program_id, payer);
        Self {
            program_id,
            payer: valid.payer,
            vault: valid.vault,
            system_program: valid.system_program,
            instruction_data: vec![1u8], // InitializeVaultV1 discriminator
        }
    }

    /// Sets the instruction discriminator to an invalid value.
    #[must_use]
    pub fn with_invalid_discriminator(mut self, discriminator: u8) -> Self {
        self.instruction_data = vec![discriminator];
        self
    }

    /// Sets empty instruction data.
    #[must_use]
    pub fn with_empty_data(mut self) -> Self {
        self.instruction_data = vec![];
        self
    }

    /// Sets the vault address to a random address.
    #[must_use]
    pub fn with_random_vault_address(mut self) -> Self {
        self.vault.pubkey = Pubkey::new_unique();
        self
    }

    /// Makes the payer not a signer.
    #[must_use]
    pub fn with_payer_not_signer(mut self) -> Self {
        self.payer.is_signer = false;
        self
    }

    /// Makes the vault not writable.
    #[must_use]
    pub fn with_vault_not_writable(mut self) -> Self {
        self.vault.is_writable = false;
        self
    }

    /// Sets the system program to a random address.
    #[must_use]
    pub fn with_random_system_program(mut self) -> Self {
        self.system_program.pubkey = Pubkey::new_unique();
        self
    }

    /// Builds the malicious instruction with a custom account list.
    #[must_use]
    pub fn build_with_accounts(self, accounts: Vec<AccountMeta>) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts,
            data: self.instruction_data,
        }
    }

    /// Builds the malicious instruction.
    #[must_use]
    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: vec![self.payer, self.vault, self.system_program],
            data: self.instruction_data,
        }
    }
}

/// Builder for creating malicious `InitializeVaultV1` transactions.
#[derive(Debug)]
pub struct MaliciousInitializeVaultV1Tx {
    program_id: Pubkey,
    payer_kp: Keypair,
    recent_blockhash: Hash,
    instruction: Instruction,
    signer: Option<Keypair>,
}

impl MaliciousInitializeVaultV1Tx {
    /// Creates a new malicious transaction builder starting from a valid transaction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, payer_kp: Keypair, recent_blockhash: Hash) -> Self {
        let valid_ix = InitializeVaultV1Ix::new(program_id, payer_kp.pubkey())
            .build()
            .unwrap();
        Self {
            program_id,
            payer_kp,
            recent_blockhash,
            instruction: valid_ix,
            signer: None,
        }
    }

    /// Sets a malicious instruction.
    #[must_use]
    pub fn with_malicious_instruction<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MaliciousInitializeVaultV1Ix) -> MaliciousInitializeVaultV1Ix,
    {
        let malicious_ix =
            MaliciousInitializeVaultV1Ix::from_valid(self.program_id, self.payer_kp.pubkey());
        self.instruction = f(malicious_ix).build();
        self
    }

    /// Sets a custom instruction.
    #[must_use]
    pub fn with_instruction(mut self, instruction: Instruction) -> Self {
        self.instruction = instruction;
        self
    }

    /// Sets a different signer for the transaction (useful for testing payer_not_signer).
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
        let signer = self.signer.as_ref().unwrap_or(&self.payer_kp);
        let payer_pk = signer.pubkey();

        let message = VersionedMessage::V0(
            v0::Message::try_compile(&payer_pk, &[self.instruction], &[], self.recent_blockhash)
                .expect("Failed to compile message"),
        );

        VersionedTransaction::try_new(message, &[signer.insecure_clone()])
            .expect("Failed to create transaction")
    }
}
