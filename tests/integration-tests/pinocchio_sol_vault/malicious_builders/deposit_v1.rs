use {
    pinocchio_sol_vault_client::instructions::DepositV1Ix,
    solana_hash::Hash,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_message::v0,
    solana_pubkey::Pubkey,
    solana_transaction::{versioned::VersionedTransaction, VersionedMessage},
};

/// Builder for creating malicious `DepositV1` instructions.
#[derive(Debug, Clone)]
pub struct MaliciousDepositV1Ix {
    program_id: Pubkey,
    owner: AccountMeta,
    vault: AccountMeta,
    system_program: AccountMeta,
    instruction_data: Vec<u8>,
}

impl MaliciousDepositV1Ix {
    /// Creates a new malicious instruction builder starting from a valid instruction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, owner: Pubkey, amount: u64) -> Self {
        let valid = DepositV1Ix::new(program_id, owner, amount);
        let mut instruction_data = vec![2u8]; // DepositV1 discriminator
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        Self {
            program_id,
            owner: valid.owner,
            vault: valid.vault,
            system_program: valid.system_program,
            instruction_data,
        }
    }

    /// Sets the instruction discriminator to an invalid value.
    #[must_use]
    pub fn with_invalid_discriminator(mut self, discriminator: u8) -> Self {
        let amount_bytes: Vec<u8> = self.instruction_data[1..].to_vec();
        self.instruction_data = vec![discriminator];
        self.instruction_data.extend_from_slice(&amount_bytes);
        self
    }

    /// Sets invalid instruction data (too short).
    #[must_use]
    pub fn with_invalid_data(mut self) -> Self {
        self.instruction_data = vec![2u8, 0u8, 1u8, 2u8]; // Only 4 bytes instead of 9
        self
    }

    /// Sets the vault address to a random address.
    #[must_use]
    pub fn with_random_vault_address(mut self) -> Self {
        self.vault.pubkey = Pubkey::new_unique();
        self
    }

    /// Sets the vault address to a specific address.
    #[must_use]
    pub fn with_vault_address(mut self, vault_address: Pubkey) -> Self {
        self.vault.pubkey = vault_address;
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
            accounts: vec![self.owner, self.vault, self.system_program],
            data: self.instruction_data,
        }
    }
}

/// Builder for creating malicious `DepositV1` transactions.
#[derive(Debug)]
pub struct MaliciousDepositV1Tx {
    program_id: Pubkey,
    owner_kp: Keypair,
    recent_blockhash: Hash,
    instruction: Instruction,
    signer: Option<Keypair>,
}

impl MaliciousDepositV1Tx {
    /// Creates a new malicious transaction builder starting from a valid transaction.
    #[must_use]
    pub fn from_valid(
        program_id: Pubkey,
        owner_kp: Keypair,
        amount: u64,
        recent_blockhash: Hash,
    ) -> Self {
        let valid_ix = DepositV1Ix::new(program_id, owner_kp.pubkey(), amount)
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
        F: FnOnce(MaliciousDepositV1Ix) -> MaliciousDepositV1Ix,
    {
        // Extract amount from existing instruction data if available
        let amount = if self.instruction.data.len() >= 9 {
            u64::from_le_bytes(self.instruction.data[1..9].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        let malicious_ix =
            MaliciousDepositV1Ix::from_valid(self.program_id, self.owner_kp.pubkey(), amount);
        self.instruction = f(malicious_ix).build();
        self
    }

    /// Sets a custom instruction.
    #[must_use]
    pub fn with_instruction(mut self, instruction: Instruction) -> Self {
        self.instruction = instruction;
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
