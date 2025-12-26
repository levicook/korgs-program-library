use {
    crate::find_counter_address,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum DeactivateCounterV1IxError {
    #[error("Owner must be a signer")]
    OwnerMustBeSigner,

    #[error("Owner must be writable")]
    OwnerMustBeWritable,

    #[error("Counter must be writable")]
    CounterMustBeWritable,

    #[error("Counter address mismatch: expected {expected:?}, observed {observed:?}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("System program address mismatch: expected {expected:?}, observed {observed:?}")]
    SystemProgramAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct DeactivateCounterV1Ix {
    pub program_id: Pubkey,
    pub owner: AccountMeta,
    pub counter: AccountMeta,
    pub system_program: AccountMeta,
}

impl DeactivateCounterV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, owner: Pubkey) -> Self {
        let (counter, _bump) = find_counter_address(&program_id, &owner);

        Self {
            program_id,
            owner: AccountMeta {
                pubkey: owner,
                is_signer: true,
                is_writable: true,
            },
            counter: AccountMeta {
                pubkey: counter,
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
    /// Returns [`DeactivateCounterV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), DeactivateCounterV1IxError> {
        if !self.owner.is_signer {
            return Err(DeactivateCounterV1IxError::OwnerMustBeSigner);
        }

        if !self.owner.is_writable {
            return Err(DeactivateCounterV1IxError::OwnerMustBeWritable);
        }

        let (expected_counter, _bump) = find_counter_address(&self.program_id, &self.owner.pubkey);
        if self.counter.pubkey != expected_counter {
            return Err(DeactivateCounterV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: self.counter.pubkey,
            });
        }

        if !self.counter.is_writable {
            return Err(DeactivateCounterV1IxError::CounterMustBeWritable);
        }

        if self.system_program.pubkey != solana_system_program::id() {
            return Err(DeactivateCounterV1IxError::SystemProgramAddressMismatch {
                expected: solana_system_program::id(),
                observed: self.system_program.pubkey,
            });
        }

        Ok(())
    }

    /// Converts the instruction builder into a Solana instruction.
    ///
    /// # Errors
    ///
    /// Returns [`DeactivateCounterV1IxError`] if `validate` is `true` and validation fails.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, DeactivateCounterV1IxError> {
        if validate {
            self.validate()?;
        }

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.owner, self.counter, self.system_program],
            data: vec![InstructionDiscriminator::DeactivateCounterV1.into()],
        })
    }
}

impl TryFrom<DeactivateCounterV1Ix> for Instruction {
    type Error = DeactivateCounterV1IxError;

    fn try_from(value: DeactivateCounterV1Ix) -> Result<Self, Self::Error> {
        value.to_instruction(true)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::find_counter_address};

    #[test]
    fn test_new_derives_counter_pda_correctly() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        let (expected_counter, _bump) = find_counter_address(&program_id, &owner);

        assert_eq!(deactivate_ix.counter.pubkey, expected_counter);
        assert_eq!(deactivate_ix.program_id, program_id);
        assert_eq!(deactivate_ix.owner.pubkey, owner);
    }

    #[test]
    fn test_new_sets_account_metadata_correctly() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);

        // Owner should be signer and writable
        assert!(deactivate_ix.owner.is_signer);
        assert!(deactivate_ix.owner.is_writable);

        // Counter should be writable but not signer
        assert!(!deactivate_ix.counter.is_signer);
        assert!(deactivate_ix.counter.is_writable);

        // System program should be neither signer nor writable
        assert!(!deactivate_ix.system_program.is_signer);
        assert!(!deactivate_ix.system_program.is_writable);
        assert_eq!(
            deactivate_ix.system_program.pubkey,
            solana_system_program::id()
        );
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        assert!(deactivate_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_owner_not_signer() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix.owner.is_signer = false;

        let err = deactivate_ix.validate().unwrap_err();
        match err {
            DeactivateCounterV1IxError::OwnerMustBeSigner => {}
            _ => panic!("Expected OwnerMustBeSigner, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Owner must be a signer");
    }

    #[test]
    fn test_validate_fails_when_owner_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix.owner.is_writable = false;

        let err = deactivate_ix.validate().unwrap_err();
        match err {
            DeactivateCounterV1IxError::OwnerMustBeWritable => {}
            _ => panic!("Expected OwnerMustBeWritable, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Owner must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &owner);

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        let wrong_counter = Pubkey::new_unique();
        deactivate_ix.counter.pubkey = wrong_counter; // Wrong address

        let err = deactivate_ix.validate().unwrap_err();
        match &err {
            DeactivateCounterV1IxError::CounterAddressMismatch { expected, observed } => {
                assert_eq!(expected, &expected_counter);
                assert_eq!(observed, &wrong_counter);
            }
            _ => panic!("Expected CounterAddressMismatch, got {err:?}"),
        }
        assert!(
            err.to_string().contains("Counter address mismatch"),
            "Counter address mismatch should be in the error message: {:?}",
            err.to_string()
        );
    }

    #[test]
    fn test_validate_fails_when_counter_not_writable() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix.counter.is_writable = false;

        let err = deactivate_ix.validate().unwrap_err();
        match err {
            DeactivateCounterV1IxError::CounterMustBeWritable => {}
            _ => panic!("Expected CounterMustBeWritable, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Counter must be writable");
    }

    #[test]
    fn test_validate_fails_when_system_program_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let expected_system_program = solana_system_program::id();

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        let wrong_system_program = Pubkey::new_unique();
        deactivate_ix.system_program.pubkey = wrong_system_program; // Wrong address

        let err = deactivate_ix.validate().unwrap_err();
        match &err {
            DeactivateCounterV1IxError::SystemProgramAddressMismatch { expected, observed } => {
                assert_eq!(expected, &expected_system_program);
                assert_eq!(observed, &wrong_system_program);
            }
            _ => panic!("Expected SystemProgramAddressMismatch, got {err:?}"),
        }
        assert!(
            err.to_string().contains("System program address mismatch"),
            "System program address mismatch should be in the error message: {:?}",
            err.to_string()
        );
    }

    #[test]
    fn test_to_instruction_creates_correct_structure() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &owner);

        let deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        let instruction = deactivate_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(instruction.accounts[2].pubkey, solana_system_program::id());
        assert_eq!(
            instruction.data,
            vec![InstructionDiscriminator::DeactivateCounterV1 as u8]
        );
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        // Valid struct - should work with or without validation
        let deactivate_ix1 = DeactivateCounterV1Ix::new(program_id, owner);
        assert!(deactivate_ix1.to_instruction(true).is_ok());

        let deactivate_ix2 = DeactivateCounterV1Ix::new(program_id, owner);
        assert!(deactivate_ix2.to_instruction(false).is_ok());

        // Invalid struct - should fail with validation, succeed without
        let mut deactivate_ix3 = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix3.owner.is_signer = false;

        assert!(deactivate_ix3.to_instruction(true).is_err());

        let mut deactivate_ix4 = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix4.owner.is_signer = false;
        // This should succeed even though it's invalid
        let instruction = deactivate_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        let instruction = Instruction::try_from(deactivate_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(
            instruction.data,
            vec![InstructionDiscriminator::DeactivateCounterV1 as u8]
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let mut deactivate_ix = DeactivateCounterV1Ix::new(program_id, owner);
        deactivate_ix.owner.is_signer = false;

        let err = Instruction::try_from(deactivate_ix).unwrap_err();
        match err {
            DeactivateCounterV1IxError::OwnerMustBeSigner => {}
            _ => panic!("Expected OwnerMustBeSigner, got {err:?}"),
        }
    }
}
