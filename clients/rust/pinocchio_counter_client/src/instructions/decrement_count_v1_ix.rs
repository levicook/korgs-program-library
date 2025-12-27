use {
    crate::find_counter_address,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum DecrementCountV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWriteable,

    #[error("Counter account must be writable")]
    CounterMustBeWriteable,

    #[error("Counter address mismatch. Expected: {expected}, Observed: {observed}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
}

/// Instruction builder for `DecrementCountV1`.
///
/// This struct facilitates the creation of a Solana `Instruction` for decrementing
/// a counter's count. It includes methods for setting account metadata and
/// validating the instruction's integrity.
#[derive(Debug, Clone)]
pub struct DecrementCountV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub counter: AccountMeta,
}

impl DecrementCountV1Ix {
    /// Creates a new instruction builder for `DecrementCountV1`.
    ///
    /// # Arguments
    ///
    /// * `program_id` - The ID of the Pinocchio counter program.
    /// * `owner` - The public key of the counter's owner.
    ///
    /// # Returns
    ///
    /// A new `DecrementCountV1Ix` instance with default account metadata.
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
    /// Returns [`DecrementCountV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), DecrementCountV1IxError> {
        if !self.owner.is_signer {
            return Err(DecrementCountV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(DecrementCountV1IxError::OwnerMustBeWriteable);
        }

        let (expected_counter, _bump) = find_counter_address(&self.program_id, &self.owner.pubkey);
        if self.counter.pubkey != expected_counter {
            return Err(DecrementCountV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: self.counter.pubkey,
            });
        }

        if !self.counter.is_writable {
            return Err(DecrementCountV1IxError::CounterMustBeWriteable);
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`DecrementCountV1IxError`] if `validate` is `true` and validation fails.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, DecrementCountV1IxError> {
        if validate {
            self.validate()?;
        }

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.counter],
            data: vec![InstructionDiscriminator::DecrementCountV1.into()],
        })
    }
}

impl TryFrom<DecrementCountV1Ix> for Instruction {
    type Error = DecrementCountV1IxError;

    fn try_from(value: DecrementCountV1Ix) -> Result<Self, Self::Error> {
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

        let decrement_ix = DecrementCountV1Ix::new(program_id, owner);

        assert_eq!(decrement_ix.counter.pubkey, expected_counter);
        assert_eq!(decrement_ix.program_id, program_id);
        assert_eq!(decrement_ix.owner.pubkey, owner);
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let decrement_ix = DecrementCountV1Ix::new(program_id, owner);

        assert!(decrement_ix.owner.is_signer);
        assert!(decrement_ix.owner.is_writable);
        assert!(!decrement_ix.counter.is_signer);
        assert!(decrement_ix.counter.is_writable);

        assert!(decrement_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_owner_not_signer() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix.owner.is_signer = false;

        let err = decrement_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be a signer");
    }

    #[test]
    fn test_validate_fails_when_owner_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix.owner.is_writable = false;

        let err = decrement_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let wrong_counter = Pubkey::new_unique();

        let mut decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix.counter.pubkey = wrong_counter;

        let err = decrement_ix.validate().unwrap_err();
        assert!(err.to_string().contains("Counter address mismatch"));
    }

    #[test]
    fn test_validate_fails_when_counter_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix.counter.is_writable = false;

        let err = decrement_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Counter account must be writable");
    }

    #[test]
    fn test_to_instruction_creates_correct_structure() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &owner);

        let decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        let instruction = decrement_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::DecrementCountV1)]
        );
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let decrement_ix1 = DecrementCountV1Ix::new(program_id, owner);
        assert!(decrement_ix1.to_instruction(true).is_ok());

        let decrement_ix2 = DecrementCountV1Ix::new(program_id, owner);
        assert!(decrement_ix2.to_instruction(false).is_ok());

        let mut decrement_ix3 = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix3.owner.is_signer = false;
        assert!(decrement_ix3.to_instruction(true).is_err());

        let mut decrement_ix4 = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix4.owner.is_signer = false;
        let instruction = decrement_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert!(!instruction.accounts[0].is_signer);
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        let instruction = Instruction::try_from(decrement_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::DecrementCountV1)]
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut decrement_ix = DecrementCountV1Ix::new(program_id, owner);
        decrement_ix.owner.is_signer = false;

        let err = Instruction::try_from(decrement_ix).unwrap_err();
        match err {
            DecrementCountV1IxError::OwnerMustBeSigner => {}
            _ => panic!("Expected OwnerMustBeSigner, got {err:?}"),
        }
    }
}
