use {
    crate::find_counter_address,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum IncrementCountV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWritable,

    #[error("Counter account must be writable")]
    CounterMustBeWritable,

    #[error("Counter address mismatch. Expected: {expected}, Observed: {observed}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
}

/// Instruction builder for `IncrementCountV1`.
///
/// This struct facilitates the creation of a Solana `Instruction` for incrementing
/// a counter's count. It includes methods for setting account metadata and
/// validating the instruction's integrity.
#[derive(Debug, Clone)]
pub struct IncrementCountV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub counter: AccountMeta,
}

impl IncrementCountV1Ix {
    /// Creates a new instruction builder for `IncrementCountV1`.
    ///
    /// # Arguments
    ///
    /// * `program_id` - The ID of the Pinocchio counter program.
    /// * `owner` - The public key of the counter's owner.
    ///
    /// # Returns
    ///
    /// A new `IncrementCountV1Ix` instance with default account metadata.
    #[must_use]
    pub fn new(program_id: Pubkey, owner: Pubkey) -> Self {
        let (counter_address, _bump) = find_counter_address(&program_id, &owner);
        Self {
            program_id,
            owner: AccountMeta {
                pubkey: owner,
                is_signer: true,
                is_writable: true,
            },
            counter: AccountMeta {
                pubkey: counter_address,
                is_signer: false,
                is_writable: true,
            },
        }
    }

    /// Sets the owner account metadata.
    #[must_use]
    pub fn with_owner(mut self, owner: AccountMeta) -> Self {
        self.owner = owner;
        self
    }

    /// Sets the counter account metadata.
    #[must_use]
    pub fn with_counter(mut self, counter: AccountMeta) -> Self {
        self.counter = counter;
        self
    }

    /// Validates the instruction's account metadata.
    ///
    /// # Errors
    ///
    /// Returns [`IncrementCountV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), IncrementCountV1IxError> {
        if !self.owner.is_signer {
            return Err(IncrementCountV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(IncrementCountV1IxError::OwnerMustBeWritable);
        }

        let (expected_counter, _bump) = find_counter_address(&self.program_id, &self.owner.pubkey);
        if self.counter.pubkey != expected_counter {
            return Err(IncrementCountV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: self.counter.pubkey,
            });
        }

        if !self.counter.is_writable {
            return Err(IncrementCountV1IxError::CounterMustBeWritable);
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`IncrementCountV1IxError`] if `validate` is `true` and validation fails.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, IncrementCountV1IxError> {
        if validate {
            self.validate()?;
        }

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.counter],
            data: vec![InstructionDiscriminator::IncrementCountV1.into()],
        })
    }
}

impl TryFrom<IncrementCountV1Ix> for Instruction {
    type Error = IncrementCountV1IxError;

    fn try_from(value: IncrementCountV1Ix) -> Result<Self, Self::Error> {
        value.to_instruction(true)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::find_counter_address};

    #[test]
    fn test_new_creates_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let (expected_counter, _bump) = find_counter_address(&program_id, &owner);

        let increment_ix = IncrementCountV1Ix::new(program_id, owner);

        assert_eq!(increment_ix.counter.pubkey, expected_counter);
        assert_eq!(increment_ix.program_id, program_id);
        assert_eq!(increment_ix.owner.pubkey, owner);
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let increment_ix = IncrementCountV1Ix::new(program_id, owner);

        assert!(increment_ix.owner.is_signer);
        assert!(increment_ix.owner.is_writable);
        assert!(!increment_ix.counter.is_signer);
        assert!(increment_ix.counter.is_writable);

        assert!(increment_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_owner_not_signer() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut increment_ix = IncrementCountV1Ix::new(program_id, owner);
        increment_ix.owner.is_signer = false;

        let err = increment_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be a signer");
    }

    #[test]
    fn test_validate_fails_when_owner_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut increment_ix = IncrementCountV1Ix::new(program_id, owner);
        increment_ix.owner.is_writable = false;

        let err = increment_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let wrong_counter = Pubkey::new_unique();

        let mut increment_ix = IncrementCountV1Ix::new(program_id, owner);
        increment_ix.counter.pubkey = wrong_counter;

        let err = increment_ix.validate().unwrap_err();
        assert!(err.to_string().contains("Counter address mismatch"));
    }

    #[test]
    fn test_validate_fails_when_counter_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut increment_ix = IncrementCountV1Ix::new(program_id, owner);
        increment_ix.counter.is_writable = false;

        let err = increment_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Counter account must be writable");
    }

    #[test]
    fn test_to_instruction_creates_correct_structure() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &owner);

        let increment_ix = IncrementCountV1Ix::new(program_id, owner);
        let instruction = increment_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::IncrementCountV1)]
        );
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let increment_ix1 = IncrementCountV1Ix::new(program_id, owner);
        assert!(increment_ix1.to_instruction(true).is_ok());

        let increment_ix2 = IncrementCountV1Ix::new(program_id, owner);
        assert!(increment_ix2.to_instruction(false).is_ok());

        let mut increment_ix3 = IncrementCountV1Ix::new(program_id, owner);
        increment_ix3.owner.is_signer = false;
        assert!(increment_ix3.to_instruction(true).is_err());

        let mut increment_ix4 = IncrementCountV1Ix::new(program_id, owner);
        increment_ix4.owner.is_signer = false;
        let instruction = increment_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert!(!instruction.accounts[0].is_signer);
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let increment_ix = IncrementCountV1Ix::new(program_id, owner);
        let instruction = Instruction::try_from(increment_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::IncrementCountV1)]
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut increment_ix = IncrementCountV1Ix::new(program_id, owner);
        increment_ix.owner.is_signer = false;

        let err = Instruction::try_from(increment_ix).unwrap_err();
        match err {
            IncrementCountV1IxError::OwnerMustBeSigner => {}
            _ => panic!("Expected OwnerMustBeSigner, got {err:?}"),
        }
    }
}
