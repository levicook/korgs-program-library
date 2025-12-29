use {
    crate::{
        instructions::{
            DeactivateCounterV1Error, DecrementCountV1Error, IncrementCountV1Error,
            InitializeCounterV1Error, ReactivateCounterV1Error, SetCountV1Error,
        },
        InstructionDiscriminatorError,
    },
    pinocchio::program_error::ProgramError,
};

// Code offsets for each error type
const INSTRUCTION_DISCRIMINATOR_ERROR_OFFSET: u32 = 0x000; // 0
const INITIALIZE_COUNTER_V1_OFFSET: u32 = 0x100; // 256
const DEACTIVATE_COUNTER_V1_OFFSET: u32 = 0x200; // 512
const INCREMENT_COUNT_V1_OFFSET: u32 = 0x300; // 768
const DECREMENT_COUNT_V1_OFFSET: u32 = 0x400; // 1024
const SET_COUNT_V1_OFFSET: u32 = 0x500; // 1280
const REACTIVATE_COUNTER_V1_OFFSET: u32 = 0x600; // 1536

#[derive(Debug)]
pub enum InstructionError {
    ProgramError(ProgramError),
    InitializeCounterV1(InitializeCounterV1Error),
    DeactivateCounterV1(DeactivateCounterV1Error),
    IncrementCountV1(IncrementCountV1Error),
    DecrementCountV1(DecrementCountV1Error),
    SetCountV1(SetCountV1Error),
    ReactivateCounterV1(ReactivateCounterV1Error),
}

pub type InstructionResult<T> = Result<T, InstructionError>;

impl From<InstructionError> for ProgramError {
    fn from(e: InstructionError) -> Self {
        match e {
            InstructionError::ProgramError(pe) => pe,
            InstructionError::InitializeCounterV1(e) => ProgramError::Custom(
                INITIALIZE_COUNTER_V1_OFFSET
                    + match e {
                        InitializeCounterV1Error::NotEnoughAccounts { .. } => 0x01,
                        InitializeCounterV1Error::PayerMustBeSigner => 0x02,
                        InitializeCounterV1Error::CounterMustBeWriteable => 0x03,
                        InitializeCounterV1Error::CounterAddressMismatch { .. } => 0x04,
                        InitializeCounterV1Error::CounterMustBeEmpty => 0x05,
                        InitializeCounterV1Error::CounterMustHaveZeroLamports => 0x06,
                        InitializeCounterV1Error::CounterMustBeOwnedBySystemProgram => 0x07,
                        InitializeCounterV1Error::SystemProgramAddressMismatch => 0x08,
                        InitializeCounterV1Error::DeserializeError(_) => 0x09,
                        InitializeCounterV1Error::SerializeError(_) => 0x0a,
                        InitializeCounterV1Error::SerializedSizeMismatch { .. } => 0x0b,
                        InitializeCounterV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::DeactivateCounterV1(e) => ProgramError::Custom(
                DEACTIVATE_COUNTER_V1_OFFSET
                    + match e {
                        DeactivateCounterV1Error::NotEnoughAccounts { .. } => 0x01,
                        DeactivateCounterV1Error::OwnerMustBeSigner => 0x02,
                        DeactivateCounterV1Error::OwnerMustBeWriteable => 0x03,
                        DeactivateCounterV1Error::CounterMustBeWriteable => 0x04,
                        DeactivateCounterV1Error::CounterAddressMismatch { .. } => 0x05,
                        // 0x06 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
                        // 0x07 reserved to maintain existing error code mappings
                        DeactivateCounterV1Error::DeserializeError(_) => 0x08,
                        // 0x09 reserved to maintain existing error code mappings
                        // 0x0a reserved (retired: OwnerMismatch - redundant with address validation)
                        DeactivateCounterV1Error::AccountDiscriminatorError(_) => 0x0b,
                        DeactivateCounterV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::IncrementCountV1(e) => ProgramError::Custom(
                INCREMENT_COUNT_V1_OFFSET
                    + match e {
                        IncrementCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        IncrementCountV1Error::OwnerMustBeSigner => 0x02,
                        // 0x03 reserved to maintain existing error code mappings
                        IncrementCountV1Error::CounterMustBeWriteable => 0x04,
                        IncrementCountV1Error::CounterAddressMismatch { .. } => 0x05,
                        // 0x06 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
                        IncrementCountV1Error::DeserializeError(_) => 0x07,
                        IncrementCountV1Error::SerializeError(_) => 0x08,
                        // 0x09 reserved (retired: OwnerMismatch - redundant with address validation)
                        IncrementCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        IncrementCountV1Error::AccountDiscriminatorError(_) => 0x0b,
                        IncrementCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::DecrementCountV1(e) => ProgramError::Custom(
                DECREMENT_COUNT_V1_OFFSET
                    + match e {
                        DecrementCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        DecrementCountV1Error::OwnerMustBeSigner => 0x02,
                        // 0x03 reserved to maintain existing error code mappings
                        DecrementCountV1Error::CounterMustBeWriteable => 0x04,
                        DecrementCountV1Error::CounterAddressMismatch { .. } => 0x05,
                        // 0x06 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
                        DecrementCountV1Error::DeserializeError(_) => 0x07,
                        DecrementCountV1Error::SerializeError(_) => 0x08,
                        // 0x09 reserved (retired: OwnerMismatch - redundant with address validation)
                        DecrementCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        DecrementCountV1Error::AccountDiscriminatorError(_) => 0x0b,
                        DecrementCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::SetCountV1(e) => ProgramError::Custom(
                SET_COUNT_V1_OFFSET
                    + match e {
                        SetCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        SetCountV1Error::OwnerMustBeSigner => 0x02,
                        // 0x03 reserved to maintain existing error code mappings
                        SetCountV1Error::CounterMustBeWriteable => 0x04,
                        SetCountV1Error::CounterAddressMismatch { .. } => 0x05,
                        // 0x06 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
                        SetCountV1Error::DeserializeError(_) => 0x07,
                        SetCountV1Error::SerializeError(_) => 0x08,
                        // 0x09 reserved (retired: OwnerMismatch - redundant with address validation)
                        SetCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        SetCountV1Error::AccountDiscriminatorError(_) => 0x0b,
                        SetCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::ReactivateCounterV1(e) => ProgramError::Custom(
                REACTIVATE_COUNTER_V1_OFFSET
                    + match e {
                        ReactivateCounterV1Error::NotEnoughAccounts { .. } => 0x01,
                        ReactivateCounterV1Error::PayerMustBeSigner => 0x02,
                        ReactivateCounterV1Error::CounterMustBeWriteable => 0x03,
                        ReactivateCounterV1Error::CounterAddressMismatch { .. } => 0x04,
                        // 0x05 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
                        ReactivateCounterV1Error::SystemProgramAddressMismatch => 0x06,
                        ReactivateCounterV1Error::DeserializeError(_) => 0x07,
                        ReactivateCounterV1Error::SerializeError(_) => 0x08,
                        // 0x09 reserved to maintain existing error code mappings
                        ReactivateCounterV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        ReactivateCounterV1Error::AccountDiscriminatorError(_) => 0x0b,
                        ReactivateCounterV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
        }
    }
}

impl From<InstructionDiscriminatorError> for ProgramError {
    fn from(e: InstructionDiscriminatorError) -> Self {
        ProgramError::Custom(
            INSTRUCTION_DISCRIMINATOR_ERROR_OFFSET
                + match e {
                    InstructionDiscriminatorError::Missing => 0x01,
                    InstructionDiscriminatorError::Invalid(_) => 0x02,
                },
        )
    }
}

impl From<DeactivateCounterV1Error> for InstructionError {
    fn from(err: DeactivateCounterV1Error) -> Self {
        match err {
            DeactivateCounterV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::DeactivateCounterV1(err),
        }
    }
}

impl From<DecrementCountV1Error> for InstructionError {
    fn from(err: DecrementCountV1Error) -> Self {
        match err {
            DecrementCountV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::DecrementCountV1(err),
        }
    }
}

impl From<IncrementCountV1Error> for InstructionError {
    fn from(err: IncrementCountV1Error) -> Self {
        match err {
            IncrementCountV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::IncrementCountV1(err),
        }
    }
}

impl From<InitializeCounterV1Error> for InstructionError {
    fn from(err: InitializeCounterV1Error) -> Self {
        match err {
            InitializeCounterV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::InitializeCounterV1(err),
        }
    }
}

impl From<SetCountV1Error> for InstructionError {
    fn from(err: SetCountV1Error) -> Self {
        match err {
            SetCountV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::SetCountV1(err),
        }
    }
}

impl From<ReactivateCounterV1Error> for InstructionError {
    fn from(err: ReactivateCounterV1Error) -> Self {
        match err {
            ReactivateCounterV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::ReactivateCounterV1(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{AccountDiscriminator, AccountDiscriminatorError},
        wincode::{ReadError, WriteError},
    };

    #[test]
    fn test_instruction_discriminator_error_codes() {
        let test_cases = [
            (0x001, InstructionDiscriminatorError::Missing),
            (0x002, InstructionDiscriminatorError::Invalid(u8::MAX)),
        ];

        for (expected_code, error) in test_cases {
            let program_error: ProgramError = error.into();
            assert!(matches!(program_error, ProgramError::Custom(code) if code == expected_code));
        }
    }

    #[test]
    fn test_instruction_error_codes() {
        let test_cases = [
            // ==============================================================================
            // InitializeCounterV1 (0x100 range)
            // ==============================================================================
            // 0x100 reserved
            (
                0x101,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::NotEnoughAccounts {
                        expected: 3,
                        observed: 2,
                    },
                ),
            ),
            (
                0x102,
                InstructionError::InitializeCounterV1(InitializeCounterV1Error::PayerMustBeSigner),
            ),
            (
                0x103,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::CounterMustBeWriteable,
                ),
            ),
            (
                0x104,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::CounterAddressMismatch {
                        expected: Default::default(),
                        observed: Default::default(),
                    },
                ),
            ),
            (
                0x105,
                InstructionError::InitializeCounterV1(InitializeCounterV1Error::CounterMustBeEmpty),
            ),
            (
                0x106,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::CounterMustHaveZeroLamports,
                ),
            ),
            (
                0x107,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::CounterMustBeOwnedBySystemProgram,
                ),
            ),
            (
                0x108,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::SystemProgramAddressMismatch,
                ),
            ),
            (
                0x109,
                InstructionError::InitializeCounterV1(InitializeCounterV1Error::DeserializeError(
                    ReadError::Custom("test"),
                )),
            ),
            (
                0x10a,
                InstructionError::InitializeCounterV1(InitializeCounterV1Error::SerializeError(
                    WriteError::Custom("test"),
                )),
            ),
            (
                0x10b,
                InstructionError::InitializeCounterV1(
                    InitializeCounterV1Error::SerializedSizeMismatch {
                        expected: 100,
                        observed: 50,
                    },
                ),
            ),
            // ==============================================================================
            // DeactivateCounterV1 (0x200 range)
            // ==============================================================================
            // 0x200 reserved
            (
                0x201,
                InstructionError::DeactivateCounterV1(
                    DeactivateCounterV1Error::NotEnoughAccounts {
                        expected: 2,
                        observed: 1,
                    },
                ),
            ),
            (
                0x202,
                InstructionError::DeactivateCounterV1(DeactivateCounterV1Error::OwnerMustBeSigner),
            ),
            (
                0x203,
                InstructionError::DeactivateCounterV1(
                    DeactivateCounterV1Error::OwnerMustBeWriteable,
                ),
            ),
            (
                0x204,
                InstructionError::DeactivateCounterV1(
                    DeactivateCounterV1Error::CounterMustBeWriteable,
                ),
            ),
            (
                0x205,
                InstructionError::DeactivateCounterV1(
                    DeactivateCounterV1Error::CounterAddressMismatch {
                        expected: Default::default(),
                        observed: Default::default(),
                    },
                ),
            ),
            // 0x206 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
            // 0x207 reserved (retired)
            (
                0x208,
                InstructionError::DeactivateCounterV1(DeactivateCounterV1Error::DeserializeError(
                    ReadError::Custom("test"),
                )),
            ),
            // 0x209 reserved (retired)
            // 0x20a reserved (retired: OwnerMismatch - redundant with address validation)
            (
                0x20b,
                InstructionError::DeactivateCounterV1(
                    DeactivateCounterV1Error::AccountDiscriminatorError(
                        AccountDiscriminatorError::Missing,
                    ),
                ),
            ),
            // ==============================================================================
            // IncrementCountV1 (0x300 range)
            // ==============================================================================
            // 0x300 reserved
            (
                0x301,
                InstructionError::IncrementCountV1(IncrementCountV1Error::NotEnoughAccounts {
                    expected: 2,
                    observed: 1,
                }),
            ),
            (
                0x302,
                InstructionError::IncrementCountV1(IncrementCountV1Error::OwnerMustBeSigner),
            ),
            // 0x303 reserved (retired)
            (
                0x304,
                InstructionError::IncrementCountV1(IncrementCountV1Error::CounterMustBeWriteable),
            ),
            (
                0x305,
                InstructionError::IncrementCountV1(IncrementCountV1Error::CounterAddressMismatch {
                    expected: Default::default(),
                    observed: Default::default(),
                }),
            ),
            // 0x306 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
            (
                0x307,
                InstructionError::IncrementCountV1(IncrementCountV1Error::DeserializeError(
                    ReadError::Custom("test"),
                )),
            ),
            (
                0x308,
                InstructionError::IncrementCountV1(IncrementCountV1Error::SerializeError(
                    WriteError::Custom("test"),
                )),
            ),
            // 0x309 reserved (retired: OwnerMismatch - redundant with address validation)
            (
                0x30a,
                InstructionError::IncrementCountV1(IncrementCountV1Error::SerializedSizeMismatch {
                    expected: 100,
                    observed: 50,
                }),
            ),
            (
                0x30b,
                InstructionError::IncrementCountV1(
                    IncrementCountV1Error::AccountDiscriminatorError(
                        AccountDiscriminatorError::DiscriminatorMismatch {
                            expected: AccountDiscriminator::CounterV1Account,
                            observed: AccountDiscriminator::DeactivatedAccount,
                        },
                    ),
                ),
            ),
            // ==============================================================================
            // DecrementCountV1 (0x400 range)
            // ==============================================================================
            // 0x400 reserved
            (
                0x401,
                InstructionError::DecrementCountV1(DecrementCountV1Error::NotEnoughAccounts {
                    expected: 2,
                    observed: 1,
                }),
            ),
            (
                0x402,
                InstructionError::DecrementCountV1(DecrementCountV1Error::OwnerMustBeSigner),
            ),
            // 0x403 reserved (retired)
            (
                0x404,
                InstructionError::DecrementCountV1(DecrementCountV1Error::CounterMustBeWriteable),
            ),
            (
                0x405,
                InstructionError::DecrementCountV1(DecrementCountV1Error::CounterAddressMismatch {
                    expected: Default::default(),
                    observed: Default::default(),
                }),
            ),
            // 0x406 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
            (
                0x407,
                InstructionError::DecrementCountV1(DecrementCountV1Error::DeserializeError(
                    ReadError::Custom("test"),
                )),
            ),
            (
                0x408,
                InstructionError::DecrementCountV1(DecrementCountV1Error::SerializeError(
                    WriteError::Custom("test"),
                )),
            ),
            // 0x409 reserved (retired: OwnerMismatch - redundant with address validation)
            (
                0x40a,
                InstructionError::DecrementCountV1(DecrementCountV1Error::SerializedSizeMismatch {
                    expected: 100,
                    observed: 50,
                }),
            ),
            (
                0x40b,
                InstructionError::DecrementCountV1(
                    DecrementCountV1Error::AccountDiscriminatorError(
                        AccountDiscriminatorError::Invalid(0),
                    ),
                ),
            ),
            // ==============================================================================
            // SetCountV1 (0x500 range)
            // ==============================================================================
            // 0x500 reserved
            (
                0x501,
                InstructionError::SetCountV1(SetCountV1Error::NotEnoughAccounts {
                    expected: 2,
                    observed: 1,
                }),
            ),
            (
                0x502,
                InstructionError::SetCountV1(SetCountV1Error::OwnerMustBeSigner),
            ),
            // 0x503 reserved (retired)
            (
                0x504,
                InstructionError::SetCountV1(SetCountV1Error::CounterMustBeWriteable),
            ),
            (
                0x505,
                InstructionError::SetCountV1(SetCountV1Error::CounterAddressMismatch {
                    expected: Default::default(),
                    observed: Default::default(),
                }),
            ),
            // 0x506 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
            (
                0x507,
                InstructionError::SetCountV1(SetCountV1Error::DeserializeError(ReadError::Custom(
                    "test",
                ))),
            ),
            (
                0x508,
                InstructionError::SetCountV1(SetCountV1Error::SerializeError(WriteError::Custom(
                    "test",
                ))),
            ),
            // 0x509 reserved (retired: OwnerMismatch - redundant with address validation)
            (
                0x50a,
                InstructionError::SetCountV1(SetCountV1Error::SerializedSizeMismatch {
                    expected: 100,
                    observed: 50,
                }),
            ),
            (
                0x50b,
                InstructionError::SetCountV1(SetCountV1Error::AccountDiscriminatorError(
                    AccountDiscriminatorError::Missing,
                )),
            ),
            // ==============================================================================
            // ReactivateCounterV1 (0x600 range)
            // ==============================================================================
            // 0x600 reserved
            (
                0x601,
                InstructionError::ReactivateCounterV1(
                    ReactivateCounterV1Error::NotEnoughAccounts {
                        expected: 3,
                        observed: 2,
                    },
                ),
            ),
            (
                0x602,
                InstructionError::ReactivateCounterV1(ReactivateCounterV1Error::PayerMustBeSigner),
            ),
            // 0x603 reserved (retired: CounterMustBeWriteable - runtime enforces)
            (
                0x604,
                InstructionError::ReactivateCounterV1(
                    ReactivateCounterV1Error::CounterAddressMismatch {
                        expected: Default::default(),
                        observed: Default::default(),
                    },
                ),
            ),
            // 0x605 reserved (retired: CounterMustBeOwnedByProgram - redundant with address validation)
            (
                0x606,
                InstructionError::ReactivateCounterV1(
                    ReactivateCounterV1Error::SystemProgramAddressMismatch,
                ),
            ),
            (
                0x607,
                InstructionError::ReactivateCounterV1(ReactivateCounterV1Error::DeserializeError(
                    ReadError::Custom("test"),
                )),
            ),
            (
                0x608,
                InstructionError::ReactivateCounterV1(ReactivateCounterV1Error::SerializeError(
                    WriteError::Custom("test"),
                )),
            ),
            // 0x609 reserved to maintain existing error code mappings
            (
                0x60a,
                InstructionError::ReactivateCounterV1(
                    ReactivateCounterV1Error::SerializedSizeMismatch {
                        expected: 100,
                        observed: 50,
                    },
                ),
            ),
            (
                0x60b,
                InstructionError::ReactivateCounterV1(
                    ReactivateCounterV1Error::AccountDiscriminatorError(
                        AccountDiscriminatorError::Missing,
                    ),
                ),
            ),
        ];

        for (expected_code, error) in test_cases {
            let program_error: ProgramError = error.into();
            assert!(
                matches!(program_error, ProgramError::Custom(code) if code == expected_code),
                "Expected error code {expected_code:#x}, got {program_error:?}"
            );
        }
    }
}
