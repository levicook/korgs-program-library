use {
    crate::instructions::{
        DeactivateCounterV1Error, DecrementCountV1Error, IncrementCountV1Error,
        InitializeCounterV1Error, SetCountV1Error,
    },
    pinocchio::program_error::ProgramError,
    std::fmt::{Display, Formatter},
};

// Base code offsets for global/program-level errors
// Reserved range: 0x01-0x0f (15 codes) for global errors
const EMPTY_INSTRUCTION_DATA_ERROR_CODE: u32 = 0x01;
const INVALID_INSTRUCTION_DISCRIMINATOR_ERROR_CODE: u32 = 0x02;

// Base code offsets for each instruction's error range
const INITIALIZE_COUNTER_V1_OFFSET: u32 = 0x100; // 256
const DEACTIVATE_COUNTER_V1_OFFSET: u32 = 0x200; // 512
const INCREMENT_COUNT_V1_OFFSET: u32 = 0x300; // 768
const DECREMENT_COUNT_V1_OFFSET: u32 = 0x400; // 1024
const SET_COUNT_V1_OFFSET: u32 = 0x500; // 1280

#[derive(Debug, PartialEq)]
pub enum CounterError {
    ProgramError(ProgramError),
    EmptyInstructionData,
    InvalidInstructionDiscriminator(u8),
    InitializeCounterV1(InitializeCounterV1Error),
    DeactivateCounterV1(DeactivateCounterV1Error),
    IncrementCountV1(IncrementCountV1Error),
    DecrementCountV1(DecrementCountV1Error),
    SetCountV1(SetCountV1Error),
}

pub type CounterResult<T> = Result<T, CounterError>;

impl From<CounterError> for ProgramError {
    fn from(e: CounterError) -> Self {
        match e {
            CounterError::ProgramError(pe) => pe,
            CounterError::EmptyInstructionData => {
                ProgramError::Custom(EMPTY_INSTRUCTION_DATA_ERROR_CODE)
            }
            CounterError::InvalidInstructionDiscriminator(_invalid_descriminator_value) => {
                ProgramError::Custom(INVALID_INSTRUCTION_DISCRIMINATOR_ERROR_CODE)
            }
            CounterError::InitializeCounterV1(e) => ProgramError::Custom(
                INITIALIZE_COUNTER_V1_OFFSET
                    + match e {
                        InitializeCounterV1Error::NotEnoughAccounts { .. } => 0x01,
                        InitializeCounterV1Error::PayerMustBeSigner => 0x02,
                        InitializeCounterV1Error::CounterMustBeWriteable => 0x03,
                        InitializeCounterV1Error::CounterAddressMismatch => 0x04,
                        InitializeCounterV1Error::CounterMustBeEmpty => 0x05,
                        InitializeCounterV1Error::CounterMustHaveZeroLamports => 0x06,
                        InitializeCounterV1Error::CounterMustBeOwnedBySystemProgram => 0x07,
                        InitializeCounterV1Error::SystemProgramAddressMismatch => 0x08,
                        InitializeCounterV1Error::DeserializeError => 0x09,
                        InitializeCounterV1Error::SerializeError => 0x0a,
                        InitializeCounterV1Error::SerializedSizeMismatch { .. } => 0x0b,
                        InitializeCounterV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            CounterError::DeactivateCounterV1(e) => ProgramError::Custom(
                DEACTIVATE_COUNTER_V1_OFFSET
                    + match e {
                        DeactivateCounterV1Error::NotEnoughAccounts { .. } => 0x01,
                        DeactivateCounterV1Error::OwnerMustBeSigner => 0x02,
                        DeactivateCounterV1Error::OwnerMustBeWritable => 0x03,
                        DeactivateCounterV1Error::CounterMustBeWriteable => 0x04,
                        DeactivateCounterV1Error::CounterAddressMismatch => 0x05,
                        DeactivateCounterV1Error::CounterMustBeOwnedByProgram => 0x06,
                        DeactivateCounterV1Error::SystemProgramAddressMismatch => 0x07,
                        DeactivateCounterV1Error::DeserializeError => 0x08,
                        DeactivateCounterV1Error::SerializeError => 0x09,
                        DeactivateCounterV1Error::OwnerMismatch => 0x0a,
                        DeactivateCounterV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            CounterError::IncrementCountV1(e) => ProgramError::Custom(
                INCREMENT_COUNT_V1_OFFSET
                    + match e {
                        IncrementCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        IncrementCountV1Error::OwnerMustBeSigner => 0x02,
                        IncrementCountV1Error::OwnerMustBeWritable => 0x03,
                        IncrementCountV1Error::CounterMustBeWriteable => 0x04,
                        IncrementCountV1Error::CounterAddressMismatch => 0x05,
                        IncrementCountV1Error::CounterMustBeOwnedByProgram => 0x06,
                        IncrementCountV1Error::DeserializeError => 0x07,
                        IncrementCountV1Error::SerializeError => 0x08,
                        IncrementCountV1Error::OwnerMismatch => 0x09,
                        IncrementCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        IncrementCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            CounterError::DecrementCountV1(e) => ProgramError::Custom(
                DECREMENT_COUNT_V1_OFFSET
                    + match e {
                        DecrementCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        DecrementCountV1Error::OwnerMustBeSigner => 0x02,
                        DecrementCountV1Error::OwnerMustBeWritable => 0x03,
                        DecrementCountV1Error::CounterMustBeWriteable => 0x04,
                        DecrementCountV1Error::CounterAddressMismatch => 0x05,
                        DecrementCountV1Error::CounterMustBeOwnedByProgram => 0x06,
                        DecrementCountV1Error::DeserializeError => 0x07,
                        DecrementCountV1Error::SerializeError => 0x08,
                        DecrementCountV1Error::OwnerMismatch => 0x09,
                        DecrementCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        DecrementCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            CounterError::SetCountV1(e) => ProgramError::Custom(
                SET_COUNT_V1_OFFSET
                    + match e {
                        SetCountV1Error::NotEnoughAccounts { .. } => 0x01,
                        SetCountV1Error::OwnerMustBeSigner => 0x02,
                        SetCountV1Error::OwnerMustBeWritable => 0x03,
                        SetCountV1Error::CounterMustBeWriteable => 0x04,
                        SetCountV1Error::CounterAddressMismatch => 0x05,
                        SetCountV1Error::CounterMustBeOwnedByProgram => 0x06,
                        SetCountV1Error::DeserializeError => 0x07,
                        SetCountV1Error::SerializeError => 0x08,
                        SetCountV1Error::OwnerMismatch => 0x09,
                        SetCountV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        SetCountV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
        }
    }
}

impl Display for CounterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Use Debug format for Display to provide detailed error information
        // including variant names and associated data (e.g., expected/observed counts)
        write!(f, "{:?}", self)
    }
}
