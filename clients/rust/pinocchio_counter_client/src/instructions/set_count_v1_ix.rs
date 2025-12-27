use {
    crate::find_counter_address,
    pinocchio_counter_program::{InstructionDiscriminator, SetCountV1Args},
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    wincode::serialize,
};

#[derive(Debug, thiserror::Error)]
pub enum SetCountV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWriteable,

    #[error("Counter account must be writable")]
    CounterMustBeWriteable,

    #[error("Counter address mismatch. Expected: {expected}, Observed: {observed}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("Failed to serialize instruction arguments")]
    SerializationError,
}

/// Instruction builder for `SetCountV1`.
///
/// This struct facilitates the creation of a Solana `Instruction` for setting
/// a counter's count to a specific value. It includes methods for setting account
/// metadata and validating the instruction's integrity.
#[derive(Debug, Clone)]
pub struct SetCountV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub counter: AccountMeta,
    pub count: u64,
}

impl SetCountV1Ix {
    /// Creates a new instruction builder for `SetCountV1`.
    ///
    /// # Arguments
    ///
    /// * `program_id` - The ID of the Pinocchio counter program.
    /// * `owner` - The public key of the counter's owner.
    /// * `count` - The count value to set.
    ///
    /// # Returns
    ///
    /// A new `SetCountV1Ix` instance with default account metadata.
    #[must_use]
    pub fn new(program_id: Pubkey, owner: Pubkey, count: u64) -> Self {
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
            count,
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

    /// Sets the count value.
    #[must_use]
    pub fn with_count(mut self, count: u64) -> Self {
        self.count = count;
        self
    }

    /// Validates the instruction's account metadata.
    ///
    /// # Errors
    ///
    /// Returns [`SetCountV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), SetCountV1IxError> {
        if !self.owner.is_signer {
            return Err(SetCountV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(SetCountV1IxError::OwnerMustBeWriteable);
        }

        let (expected_counter, _bump) = find_counter_address(&self.program_id, &self.owner.pubkey);
        if self.counter.pubkey != expected_counter {
            return Err(SetCountV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: self.counter.pubkey,
            });
        }

        if !self.counter.is_writable {
            return Err(SetCountV1IxError::CounterMustBeWriteable);
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`SetCountV1IxError`] if `validate` is `true` and validation fails, or if
    /// serialization of instruction arguments fails.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, SetCountV1IxError> {
        if validate {
            self.validate()?;
        }

        let args = SetCountV1Args { count: self.count };
        let args_data = serialize(&args).map_err(|_| SetCountV1IxError::SerializationError)?;

        let mut instruction_data = vec![InstructionDiscriminator::SetCountV1.into()];
        instruction_data.extend_from_slice(&args_data);

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.counter],
            data: instruction_data,
        })
    }
}

impl TryFrom<SetCountV1Ix> for Instruction {
    type Error = SetCountV1IxError;

    fn try_from(value: SetCountV1Ix) -> Result<Self, Self::Error> {
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
        let count = 42u64;
        let (expected_counter, _bump) = find_counter_address(&program_id, &owner);

        let set_ix = SetCountV1Ix::new(program_id, owner, count);

        assert_eq!(set_ix.counter.pubkey, expected_counter);
        assert_eq!(set_ix.program_id, program_id);
        assert_eq!(set_ix.owner.pubkey, owner);
        assert_eq!(set_ix.count, count);
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let set_ix = SetCountV1Ix::new(program_id, owner, 100);

        assert!(set_ix.owner.is_signer);
        assert!(set_ix.owner.is_writable);
        assert!(!set_ix.counter.is_signer);
        assert!(set_ix.counter.is_writable);

        assert!(set_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_owner_not_signer() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut set_ix = SetCountV1Ix::new(program_id, owner, 50);
        set_ix.owner.is_signer = false;

        let err = set_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be a signer");
    }

    #[test]
    fn test_validate_fails_when_owner_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut set_ix = SetCountV1Ix::new(program_id, owner, 50);
        set_ix.owner.is_writable = false;

        let err = set_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Owner must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let wrong_counter = Pubkey::new_unique();

        let mut set_ix = SetCountV1Ix::new(program_id, owner, 50);
        set_ix.counter.pubkey = wrong_counter;

        let err = set_ix.validate().unwrap_err();
        assert!(err.to_string().contains("Counter address mismatch"));
    }

    #[test]
    fn test_validate_fails_when_counter_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut set_ix = SetCountV1Ix::new(program_id, owner, 50);
        set_ix.counter.is_writable = false;

        let err = set_ix.validate().unwrap_err();
        assert_eq!(err.to_string(), "Counter account must be writable");
    }

    #[test]
    fn test_to_instruction_creates_correct_structure() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let count = 123u64;
        let (expected_counter, _) = find_counter_address(&program_id, &owner);

        let set_ix = SetCountV1Ix::new(program_id, owner, count);
        let instruction = set_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(
            instruction.data[0],
            u8::from(InstructionDiscriminator::SetCountV1)
        );

        // Verify the count is serialized correctly
        let args: SetCountV1Args = wincode::deserialize(&instruction.data[1..]).unwrap();
        assert_eq!(args.count, count);
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let set_ix1 = SetCountV1Ix::new(program_id, owner, 10);
        assert!(set_ix1.to_instruction(true).is_ok());

        let set_ix2 = SetCountV1Ix::new(program_id, owner, 20);
        assert!(set_ix2.to_instruction(false).is_ok());

        let mut set_ix3 = SetCountV1Ix::new(program_id, owner, 30);
        set_ix3.owner.is_signer = false;
        assert!(set_ix3.to_instruction(true).is_err());

        let mut set_ix4 = SetCountV1Ix::new(program_id, owner, 40);
        set_ix4.owner.is_signer = false;
        let instruction = set_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert!(!instruction.accounts[0].is_signer);
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let set_ix = SetCountV1Ix::new(program_id, owner, 99);
        let instruction = Instruction::try_from(set_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.data[0],
            u8::from(InstructionDiscriminator::SetCountV1)
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut set_ix = SetCountV1Ix::new(program_id, owner, 50);
        set_ix.owner.is_signer = false;

        let err = Instruction::try_from(set_ix).unwrap_err();
        match err {
            SetCountV1IxError::OwnerMustBeSigner => {}
            _ => panic!("Expected OwnerMustBeSigner, got {err:?}"),
        }
    }
}
