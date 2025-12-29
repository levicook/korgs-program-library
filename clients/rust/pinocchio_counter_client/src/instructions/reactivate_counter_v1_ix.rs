use {
    crate::find_counter_v1_address,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum ReactivateCounterV1IxError {
    #[error("Payer must be a signer")]
    PayerMustBeSigner,

    #[error("Payer must be writable")]
    PayerMustBeWriteable,

    #[error("Counter must be writable")]
    CounterMustBeWriteable,

    #[error("Counter address mismatch: expected {expected:?}, observed {observed:?}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("System program address mismatch: expected {expected:?}, observed {observed:?}")]
    SystemProgramAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct ReactivateCounterV1Ix {
    pub program_id: Pubkey,
    pub payer: AccountMeta,
    pub counter: AccountMeta,
    pub system_program: AccountMeta,
}

impl ReactivateCounterV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, payer: Pubkey) -> Self {
        let counter = find_counter_v1_address(&program_id, &payer);

        Self {
            program_id,
            payer: AccountMeta {
                pubkey: payer,
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
    /// Returns [`ReactivateCounterV1IxError`] if validation fails.
    pub fn validate(&self) -> Result<(), ReactivateCounterV1IxError> {
        if !self.payer.is_signer {
            return Err(ReactivateCounterV1IxError::PayerMustBeSigner);
        }

        if !self.payer.is_writable {
            return Err(ReactivateCounterV1IxError::PayerMustBeWriteable);
        }

        if !self.counter.is_writable {
            return Err(ReactivateCounterV1IxError::CounterMustBeWriteable);
        }

        let expected_counter = find_counter_v1_address(&self.program_id, &self.payer.pubkey);
        let observed_counter = self.counter.pubkey;
        if observed_counter != expected_counter {
            return Err(ReactivateCounterV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: observed_counter,
            });
        }

        let observed_system_program = self.system_program.pubkey;
        let expected_system_program = solana_system_program::id();
        if observed_system_program != expected_system_program {
            return Err(ReactivateCounterV1IxError::SystemProgramAddressMismatch {
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
    /// Returns [`ReactivateCounterV1IxError`] if `validate` is `true` and validation fails.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, ReactivateCounterV1IxError> {
        if validate {
            self.validate()?;
        }

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.payer, self.counter, self.system_program],
            data: vec![InstructionDiscriminator::ReactivateCounterV1.into()],
        })
    }
}

impl TryFrom<ReactivateCounterV1Ix> for Instruction {
    type Error = ReactivateCounterV1IxError;

    fn try_from(value: ReactivateCounterV1Ix) -> Result<Self, Self::Error> {
        value.to_instruction(true)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::find_counter_v1_address};

    #[test]
    fn test_new_derives_counter_pda_correctly() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        let expected_counter = find_counter_v1_address(&program_id, &payer);

        assert_eq!(reactivate_ix.counter.pubkey, expected_counter);
        assert_eq!(reactivate_ix.program_id, program_id);
        assert_eq!(reactivate_ix.payer.pubkey, payer);
    }

    #[test]
    fn test_new_sets_account_metadata_correctly() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);

        // Payer should be signer and writable
        assert!(reactivate_ix.payer.is_signer);
        assert!(reactivate_ix.payer.is_writable);

        // Counter should be writable but not signer
        assert!(!reactivate_ix.counter.is_signer);
        assert!(reactivate_ix.counter.is_writable);

        // System program should be neither signer nor writable
        assert!(!reactivate_ix.system_program.is_signer);
        assert!(!reactivate_ix.system_program.is_writable);
        assert_eq!(
            reactivate_ix.system_program.pubkey,
            solana_system_program::id()
        );
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        assert!(reactivate_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_payer_not_signer() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix.payer.is_signer = false;

        let err = reactivate_ix.validate().unwrap_err();
        match err {
            ReactivateCounterV1IxError::PayerMustBeSigner => {}
            _ => panic!("Expected PayerMustBeSigner, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Payer must be a signer");
    }

    #[test]
    fn test_validate_fails_when_payer_not_writable() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix.payer.is_writable = false;

        let err = reactivate_ix.validate().unwrap_err();
        match err {
            ReactivateCounterV1IxError::PayerMustBeWriteable => {}
            _ => panic!("Expected PayerMustBeWriteable, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Payer must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let expected_counter = find_counter_v1_address(&program_id, &payer);

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        let wrong_counter = Pubkey::new_unique();
        reactivate_ix.counter.pubkey = wrong_counter; // Wrong address

        let err = reactivate_ix.validate().unwrap_err();
        match &err {
            ReactivateCounterV1IxError::CounterAddressMismatch { expected, observed } => {
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
        let payer = Pubkey::new_unique();

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix.counter.is_writable = false;

        let err = reactivate_ix.validate().unwrap_err();
        match err {
            ReactivateCounterV1IxError::CounterMustBeWriteable => {}
            _ => panic!("Expected CounterMustBeWriteable, got {err:?}"),
        }
        assert_eq!(err.to_string(), "Counter must be writable");
    }

    #[test]
    fn test_validate_fails_when_system_program_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let expected_system_program = solana_system_program::id();

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        let wrong_system_program = Pubkey::new_unique();
        reactivate_ix.system_program.pubkey = wrong_system_program; // Wrong address

        let err = reactivate_ix.validate().unwrap_err();
        match &err {
            ReactivateCounterV1IxError::SystemProgramAddressMismatch { expected, observed } => {
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
        let payer = Pubkey::new_unique();
        let expected_counter = find_counter_v1_address(&program_id, &payer);

        let reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        let instruction = reactivate_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(instruction.accounts[2].pubkey, solana_system_program::id());
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::ReactivateCounterV1)]
        );
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        // Valid struct - should work with or without validation
        let reactivate_ix1 = ReactivateCounterV1Ix::new(program_id, payer);
        assert!(reactivate_ix1.to_instruction(true).is_ok());

        let reactivate_ix2 = ReactivateCounterV1Ix::new(program_id, payer);
        assert!(reactivate_ix2.to_instruction(false).is_ok());

        // Invalid struct - should fail with validation, succeed without
        let mut reactivate_ix3 = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix3.payer.is_signer = false;

        assert!(reactivate_ix3.to_instruction(true).is_err());

        let mut reactivate_ix4 = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix4.payer.is_signer = false;
        // This should succeed even though it's invalid
        let instruction = reactivate_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        let instruction = Instruction::try_from(reactivate_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(
            instruction.data,
            vec![u8::from(InstructionDiscriminator::ReactivateCounterV1)]
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut reactivate_ix = ReactivateCounterV1Ix::new(program_id, payer);
        reactivate_ix.payer.is_signer = false;

        let err = Instruction::try_from(reactivate_ix).unwrap_err();
        match err {
            ReactivateCounterV1IxError::PayerMustBeSigner => {}
            _ => panic!("Expected PayerMustBeSigner, got {err:?}"),
        }
    }
}
