use {
    crate::find_counter_address,
    pinocchio_counter_program::InstructionDiscriminator,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

#[derive(Debug, thiserror::Error)]
pub enum InitializeCounterV1IxError {
    #[error("Payer must be a signer")]
    PayerMustBeSigner,

    #[error("Payer must be writable")]
    PayerMustBeWritable,

    #[error("Counter must be writable")]
    CounterMustBeWritable,

    #[error("Counter address mismatch: expected {expected:?}, observed {observed:?}")]
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },

    #[error("System program address mismatch: expected {expected:?}, observed {observed:?}")]
    SystemProgramAddressMismatch { expected: Pubkey, observed: Pubkey },
}

pub struct InitializeCounterV1Ix {
    pub program_id: Pubkey,
    pub payer: AccountMeta,
    pub counter: AccountMeta,
    pub system_program: AccountMeta,
}

impl InitializeCounterV1Ix {
    #[must_use]
    pub fn new(program_id: Pubkey, payer: Pubkey) -> Self {
        let (counter, _bump) = find_counter_address(&program_id, &payer);

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
    /// Returns [`InitializeCounterV1IxError`] if:
    /// - Payer is not marked as a signer
    /// - Payer is not marked as writable
    /// - Counter address doesn't match the derived PDA address
    /// - Counter is not marked as writable
    /// - System program address is incorrect
    pub fn validate(&self) -> Result<(), InitializeCounterV1IxError> {
        if !self.payer.is_signer {
            return Err(InitializeCounterV1IxError::PayerMustBeSigner);
        }

        if !self.payer.is_writable {
            return Err(InitializeCounterV1IxError::PayerMustBeWritable);
        }

        let (expected_counter, _bump) = find_counter_address(&self.program_id, &self.payer.pubkey);
        if self.counter.pubkey != expected_counter {
            return Err(InitializeCounterV1IxError::CounterAddressMismatch {
                expected: expected_counter,
                observed: self.counter.pubkey,
            });
        }

        if !self.counter.is_writable {
            return Err(InitializeCounterV1IxError::CounterMustBeWritable);
        }

        if self.system_program.pubkey != solana_system_program::id() {
            return Err(InitializeCounterV1IxError::SystemProgramAddressMismatch {
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
    /// Returns [`InitializeCounterV1IxError`] if `validate` is `true` and validation fails.
    /// See [`validate`](Self::validate) for error conditions.
    pub fn to_instruction(self, validate: bool) -> Result<Instruction, InitializeCounterV1IxError> {
        if validate {
            self.validate()?;
        }

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![self.payer, self.counter, self.system_program],
            data: vec![InstructionDiscriminator::InitializeCounterV1.into()],
        })
    }
}

impl TryFrom<InitializeCounterV1Ix> for Instruction {
    type Error = InitializeCounterV1IxError;

    fn try_from(value: InitializeCounterV1Ix) -> Result<Self, Self::Error> {
        value.to_instruction(true)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::find_counter_address};

    #[test]
    fn test_new_derives_counter_pda_correctly() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let init_ix = InitializeCounterV1Ix::new(program_id, payer);
        let (expected_counter, _bump) = find_counter_address(&program_id, &payer);

        assert_eq!(init_ix.counter.pubkey, expected_counter);
        assert_eq!(init_ix.program_id, program_id);
        assert_eq!(init_ix.payer.pubkey, payer);
    }

    #[test]
    fn test_new_sets_account_metadata_correctly() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let init_ix = InitializeCounterV1Ix::new(program_id, payer);

        // Payer should be signer and writable
        assert!(init_ix.payer.is_signer);
        assert!(init_ix.payer.is_writable);

        // Counter should be writable but not signer
        assert!(!init_ix.counter.is_signer);
        assert!(init_ix.counter.is_writable);

        // System program should be neither signer nor writable
        assert!(!init_ix.system_program.is_signer);
        assert!(!init_ix.system_program.is_writable);
        assert_eq!(init_ix.system_program.pubkey, solana_system_program::id());
    }

    #[test]
    fn test_validate_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let init_ix = InitializeCounterV1Ix::new(program_id, payer);
        assert!(init_ix.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_when_payer_not_signer() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        init_ix.payer.is_signer = false;

        let err = init_ix.validate().unwrap_err();
        match err {
            InitializeCounterV1IxError::PayerMustBeSigner => {}
            _ => panic!("Expected PayerMustBeSigner, got {:?}", err),
        }
        assert_eq!(err.to_string(), "Payer must be a signer");
    }

    #[test]
    fn test_validate_fails_when_payer_not_writable() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        init_ix.payer.is_writable = false;

        let err = init_ix.validate().unwrap_err();
        match err {
            InitializeCounterV1IxError::PayerMustBeWritable => {}
            _ => panic!("Expected PayerMustBeWritable, got {:?}", err),
        }
        assert_eq!(err.to_string(), "Payer must be writable");
    }

    #[test]
    fn test_validate_fails_when_counter_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &payer);

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        let wrong_counter = Pubkey::new_unique();
        init_ix.counter.pubkey = wrong_counter; // Wrong address

        let err = init_ix.validate().unwrap_err();
        match &err {
            InitializeCounterV1IxError::CounterAddressMismatch { expected, observed } => {
                assert_eq!(expected, &expected_counter);
                assert_eq!(observed, &wrong_counter);
            }
            _ => panic!("Expected CounterAddressMismatch, got {:?}", err),
        }
        assert!(
            err.to_string().contains("Counter address mismatch"),
            "Counter address mismatch should be in the error message: {:?}",
            err.to_string()
        );

        assert!(
            err.to_string().contains(&expected_counter.to_string()),
            "expected counter should be in the error message: {:?}",
            err.to_string()
        );

        assert!(
            err.to_string().contains(&wrong_counter.to_string()),
            "wrong counter should be in the error message: {:?}",
            err.to_string()
        );
    }

    #[test]
    fn test_validate_fails_when_counter_not_writable() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        init_ix.counter.is_writable = false;

        let err = init_ix.validate().unwrap_err();
        match err {
            InitializeCounterV1IxError::CounterMustBeWritable => {}
            _ => panic!("Expected CounterMustBeWritable, got {:?}", err),
        }
        assert_eq!(err.to_string(), "Counter must be writable");
    }

    #[test]
    fn test_validate_fails_when_system_program_address_mismatch() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let expected_system_program = solana_system_program::id();

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        let wrong_system_program = Pubkey::new_unique();
        init_ix.system_program.pubkey = wrong_system_program; // Wrong address

        let err = init_ix.validate().unwrap_err();
        match &err {
            InitializeCounterV1IxError::SystemProgramAddressMismatch { expected, observed } => {
                assert_eq!(expected, &expected_system_program);
                assert_eq!(observed, &wrong_system_program);
            }
            _ => panic!("Expected SystemProgramAddressMismatch, got {:?}", err),
        }
        assert!(
            err.to_string().contains("System program address mismatch"),
            "System program address mismatch should be in the error message: {:?}",
            err.to_string()
        );
        assert!(
            err.to_string()
                .contains(&expected_system_program.to_string()),
            "expected system program should be in the error message: {:?}",
            err.to_string()
        );
        assert!(
            err.to_string().contains(&wrong_system_program.to_string()),
            "wrong system program should be in the error message: {:?}",
            err.to_string()
        );
    }

    #[test]
    fn test_to_instruction_creates_correct_structure() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let (expected_counter, _) = find_counter_address(&program_id, &payer);

        let init_ix = InitializeCounterV1Ix::new(program_id, payer);
        let instruction = init_ix.to_instruction(true).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, expected_counter);
        assert_eq!(instruction.accounts[2].pubkey, solana_system_program::id());
        assert_eq!(
            instruction.data,
            vec![InstructionDiscriminator::InitializeCounterV1 as u8]
        );
    }

    #[test]
    fn test_to_instruction_respects_validate_flag() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        // Valid struct - should work with or without validation
        let init_ix1 = InitializeCounterV1Ix::new(program_id, payer);
        assert!(init_ix1.to_instruction(true).is_ok());

        let init_ix2 = InitializeCounterV1Ix::new(program_id, payer);
        assert!(init_ix2.to_instruction(false).is_ok());

        // Invalid struct - should fail with validation, succeed without
        let mut init_ix3 = InitializeCounterV1Ix::new(program_id, payer);
        init_ix3.payer.is_signer = false;
        assert!(init_ix3.to_instruction(true).is_err());

        let mut init_ix4 = InitializeCounterV1Ix::new(program_id, payer);
        init_ix4.payer.is_signer = false;

        // This should succeed even though it's invalid
        let instruction = init_ix4.to_instruction(false).unwrap();
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(
            instruction.accounts[1].pubkey,
            find_counter_address(&program_id, &payer).0
        );
        assert_eq!(instruction.accounts[2].pubkey, solana_system_program::id());
        assert_eq!(
            instruction.data,
            vec![InstructionDiscriminator::InitializeCounterV1 as u8]
        );
    }

    #[test]
    fn test_try_from_succeeds_for_valid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let init_ix = InitializeCounterV1Ix::new(program_id, payer);
        let instruction = Instruction::try_from(init_ix).unwrap();

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(
            instruction.data,
            vec![InstructionDiscriminator::InitializeCounterV1 as u8]
        );
    }

    #[test]
    fn test_try_from_fails_for_invalid_struct() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let mut init_ix = InitializeCounterV1Ix::new(program_id, payer);
        init_ix.payer.is_signer = false;

        let err = Instruction::try_from(init_ix).unwrap_err();
        match err {
            InitializeCounterV1IxError::PayerMustBeSigner => {}
            _ => panic!("Expected PayerMustBeSigner, got {:?}", err),
        }
    }
}
