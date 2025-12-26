use {
    pinocchio_counter_client::instructions::InitializeCounterV1Ix,
    solana_hash::Hash,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_message::v0,
    solana_pubkey::Pubkey,
    solana_transaction::{versioned::VersionedTransaction, VersionedMessage},
};

/// Builder for creating malicious InitializeCounterV1 instructions.
///
/// This builder allows you to start from a valid instruction and mutate
/// specific properties to test security boundaries.
#[derive(Debug, Clone)]
pub struct MaliciousInitializeCounterV1Ix {
    program_id: Pubkey,
    payer: AccountMeta,
    counter: AccountMeta,
    system_program: AccountMeta,
    instruction_data: Vec<u8>,
}

impl MaliciousInitializeCounterV1Ix {
    /// Creates a new malicious instruction builder starting from a valid instruction.
    pub fn from_valid(program_id: Pubkey, payer: Pubkey) -> Self {
        let valid = InitializeCounterV1Ix::new(program_id, payer);
        Self {
            program_id,
            payer: valid.payer,
            counter: valid.counter,
            system_program: valid.system_program,
            instruction_data: vec![1u8], // InitializeCounterV1 discriminator
        }
    }

    /// Sets the instruction discriminator to an invalid value.
    pub fn with_invalid_discriminator(mut self, discriminator: u8) -> Self {
        self.instruction_data = vec![discriminator];
        self
    }

    /// Sets empty instruction data.
    pub fn with_empty_data(mut self) -> Self {
        self.instruction_data = vec![];
        self
    }

    /// Sets the counter address to a random address.
    pub fn with_random_counter_address(mut self) -> Self {
        self.counter.pubkey = Pubkey::new_unique();
        self
    }

    /// Makes the payer not a signer.
    pub fn with_payer_not_signer(mut self) -> Self {
        self.payer.is_signer = false;
        self
    }

    /// Makes the counter not writable.
    pub fn with_counter_not_writable(mut self) -> Self {
        self.counter.is_writable = false;
        self
    }

    /// Sets the system program to a random address.
    pub fn with_random_system_program(mut self) -> Self {
        self.system_program.pubkey = Pubkey::new_unique();
        self
    }

    /// Builds the malicious instruction with a custom account list.
    pub fn build_with_accounts(self, accounts: Vec<AccountMeta>) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts,
            data: self.instruction_data,
        }
    }

    /// Builds the malicious instruction.
    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: vec![self.payer, self.counter, self.system_program],
            data: self.instruction_data,
        }
    }
}

/// Builder for creating malicious InitializeCounterV1 transactions.
///
/// This builder allows you to create transactions with malicious instructions
/// or transaction-level attacks.
#[derive(Debug)]
pub struct MaliciousInitializeCounterV1Tx {
    program_id: Pubkey,
    payer_kp: Keypair,
    recent_blockhash: Hash,
    instruction: Instruction,
    signer_kp: Option<Keypair>, // If Some, use this keypair to sign instead of payer
}

impl MaliciousInitializeCounterV1Tx {
    /// Creates a new malicious transaction builder starting from a valid transaction.
    pub fn from_valid(program_id: Pubkey, payer_kp: Keypair, recent_blockhash: Hash) -> Self {
        let valid_ix = InitializeCounterV1Ix::new(program_id, payer_kp.pubkey());
        Self {
            program_id,
            payer_kp,
            recent_blockhash,
            instruction: valid_ix.to_instruction(false).unwrap(),
            signer_kp: None, // Default: sign with payer
        }
    }

    /// Uses a malicious instruction builder to create the instruction.
    pub fn with_malicious_instruction<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MaliciousInitializeCounterV1Ix) -> MaliciousInitializeCounterV1Ix,
    {
        let malicious_ix =
            MaliciousInitializeCounterV1Ix::from_valid(self.program_id, self.payer_kp.pubkey());
        self.instruction = f(malicious_ix).build();
        self
    }

    /// Uses a custom instruction.
    pub fn with_instruction(mut self, instruction: Instruction) -> Self {
        self.instruction = instruction;
        self
    }

    /// Uses a different keypair to sign the transaction (so payer is not a signer).
    /// The signer_kp will be the fee payer and must have funds.
    pub fn with_different_signer(mut self, signer_kp: Keypair) -> Self {
        self.signer_kp = Some(signer_kp);
        self
    }

    /// Builds the malicious transaction.
    pub fn build(self) -> VersionedTransaction {
        // Use signer_kp if provided, otherwise use payer_kp
        let signer = self.signer_kp.as_ref().unwrap_or(&self.payer_kp);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_malicious_ix_builder() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let malicious = MaliciousInitializeCounterV1Ix::from_valid(program_id, payer)
            .with_payer_not_signer()
            .build();

        assert!(!malicious.accounts[0].is_signer);
    }

    #[test]
    fn test_malicious_tx_builder() {
        let program_id = Pubkey::new_unique();
        let payer_kp = Keypair::new();
        let blockhash = Hash::new_unique();

        let malicious = MaliciousInitializeCounterV1Tx::from_valid(program_id, payer_kp, blockhash)
            .with_malicious_instruction(|ix| ix.with_payer_not_signer())
            .build();

        assert!(!malicious.message.static_account_keys().is_empty());
    }
}
