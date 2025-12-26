use {
    pinocchio_counter_client::instructions::DecrementCountV1Ix,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_hash::Hash,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_message::v0,
    solana_pubkey::Pubkey,
    solana_transaction::{versioned::VersionedTransaction, VersionedMessage},
};

/// Builder for creating malicious `DecrementCountV1` instructions.
///
/// This builder allows you to start from a valid instruction and mutate
/// specific properties to test security boundaries.
#[derive(Debug, Clone)]
pub struct MaliciousDecrementCountV1Ix {
    program_id: Pubkey,
    owner: AccountMeta,
    counter: AccountMeta,
    instruction_data: Vec<u8>,
}

impl MaliciousDecrementCountV1Ix {
    /// Creates a new malicious instruction builder starting from a valid instruction.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, owner: Pubkey) -> Self {
        let valid = DecrementCountV1Ix::new(program_id, owner);
        Self {
            program_id,
            owner: valid.owner,
            counter: valid.counter,
            instruction_data: vec![InstructionDiscriminator::DecrementCountV1.into()],
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

    /// Sets the counter address to a random address.
    #[must_use]
    pub fn with_random_counter_address(mut self) -> Self {
        self.counter.pubkey = Pubkey::new_unique();
        self
    }

    /// Sets the counter address to a specific address.
    #[must_use]
    pub fn with_counter_address(mut self, address: Pubkey) -> Self {
        self.counter.pubkey = address;
        self
    }

    /// Makes the owner not a signer.
    #[must_use]
    pub fn with_owner_not_signer(mut self) -> Self {
        self.owner.is_signer = false;
        self
    }

    /// Makes the counter not writable.
    #[must_use]
    pub fn with_counter_not_writable(mut self) -> Self {
        self.counter.is_writable = false;
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
            accounts: vec![self.owner, self.counter],
            data: self.instruction_data,
        }
    }
}

/// Builder for creating malicious `DecrementCountV1` transactions.
///
/// This builder allows you to create transactions with malicious instructions
/// or transaction-level attacks.
#[derive(Debug)]
pub struct MaliciousDecrementCountV1Tx {
    program_id: Pubkey,
    owner_kp: Keypair,
    recent_blockhash: Hash,
    instruction: Instruction,
    signer_kp: Option<Keypair>, // If Some, use this keypair to sign instead of owner
}

impl MaliciousDecrementCountV1Tx {
    /// Creates a new malicious transaction builder starting from a valid transaction.
    ///
    /// # Panics
    ///
    /// Panics if building the instruction fails.
    #[must_use]
    pub fn from_valid(program_id: Pubkey, owner_kp: Keypair, recent_blockhash: Hash) -> Self {
        let valid_ix = DecrementCountV1Ix::new(program_id, owner_kp.pubkey());
        Self {
            program_id,
            owner_kp,
            recent_blockhash,
            instruction: valid_ix.to_instruction(false).unwrap(),
            signer_kp: None, // Default: sign with owner
        }
    }

    /// Uses a malicious instruction builder to create the instruction.
    #[must_use]
    pub fn with_malicious_instruction<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MaliciousDecrementCountV1Ix) -> MaliciousDecrementCountV1Ix,
    {
        let malicious_ix =
            MaliciousDecrementCountV1Ix::from_valid(self.program_id, self.owner_kp.pubkey());
        self.instruction = f(malicious_ix).build();
        self
    }

    /// Uses a custom instruction.
    #[must_use]
    pub fn with_instruction(mut self, instruction: Instruction) -> Self {
        self.instruction = instruction;
        self
    }

    /// Uses a different keypair to sign the transaction (so owner is not a signer).
    /// The `signer_kp` will be the fee payer and must have funds.
    #[must_use]
    pub fn with_different_signer(mut self, signer_kp: Keypair) -> Self {
        self.signer_kp = Some(signer_kp);
        self
    }

    /// Builds the malicious transaction.
    ///
    /// # Panics
    ///
    /// Panics if message compilation or transaction creation fails.
    #[must_use]
    pub fn build(self) -> VersionedTransaction {
        // Use signer_kp if provided, otherwise use owner_kp
        let signer = self.signer_kp.as_ref().unwrap_or(&self.owner_kp);
        let fee_payer_pk = signer.pubkey();

        let message = VersionedMessage::V0(
            v0::Message::try_compile(
                &fee_payer_pk,
                &[self.instruction],
                &[],
                self.recent_blockhash,
            )
            .expect("Failed to compile message"),
        );

        VersionedTransaction::try_new(message, &[signer]).expect("Failed to create transaction")
    }
}
