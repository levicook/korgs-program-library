use std::fmt::{Display, Formatter};

use pinocchio::program_error::{ProgramError, ToStr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CounterError {
    CounterAddressMismatch,
    CounterMustBeEmpty,
    CounterMustBeOwnedBySystemProgram,
    CounterMustBeWriteable,
    CounterMustHaveZeroLamports,
    InvalidInstructionDiscriminator(u8),
    NotEnoughAccounts { expected: usize, observed: usize },
    PayerMustBeSigner,
    SerializeError,
    SerializedSizeMismatch { expected: usize, observed: usize },
    SystemProgramAddressMismatch,
}

pub type CounterResult<T> = Result<T, CounterError>;

impl CounterError {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CounterAddressMismatch => "Counter address must match",
            Self::CounterMustBeEmpty => "Counter must be empty",
            Self::CounterMustBeOwnedBySystemProgram => "Counter must be owned by system program",
            Self::CounterMustBeWriteable => "Counter must be writable",
            Self::CounterMustHaveZeroLamports => "Counter must have zero lamports",
            Self::InvalidInstructionDiscriminator(_) => "Invalid instruction discriminator",
            Self::NotEnoughAccounts { .. } => "Not enough accounts",
            Self::PayerMustBeSigner => "Payer must be a signer",
            Self::SerializeError => "Serialization error",
            Self::SerializedSizeMismatch { .. } => "Serialized size mismatch",
            Self::SystemProgramAddressMismatch => "System program address must match",
        }
    }
}

impl From<CounterError> for ProgramError {
    fn from(e: CounterError) -> Self {
        let code = match e {
            CounterError::CounterAddressMismatch => 1,
            CounterError::CounterMustBeEmpty => 2,
            CounterError::CounterMustBeOwnedBySystemProgram => 3,
            CounterError::CounterMustBeWriteable => 4,
            CounterError::CounterMustHaveZeroLamports => 5,
            CounterError::InvalidInstructionDiscriminator(_) => 6,
            CounterError::NotEnoughAccounts { .. } => 7,
            CounterError::PayerMustBeSigner => 8,
            CounterError::SerializeError => 9,
            CounterError::SerializedSizeMismatch { .. } => 10,
            CounterError::SystemProgramAddressMismatch => 11,
        };

        ProgramError::Custom(code)
    }
}

impl Display for CounterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ToStr for CounterError {
    fn to_str<E>(&self) -> &'static str {
        self.as_str()
    }
}
