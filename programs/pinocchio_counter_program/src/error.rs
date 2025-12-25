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
    pub fn as_str(&self) -> &'static str {
        use CounterError::*;
        match self {
            CounterAddressMismatch => "Counter address must match",
            CounterMustBeEmpty => "Counter must be empty",
            CounterMustBeOwnedBySystemProgram => "Counter must be owned by system program",
            CounterMustBeWriteable => "Counter must be writable",
            CounterMustHaveZeroLamports => "Counter must have zero lamports",
            InvalidInstructionDiscriminator(_) => "Invalid instruction discriminator",
            NotEnoughAccounts { .. } => "Not enough accounts",
            PayerMustBeSigner => "Payer must be a signer",
            SerializeError => "Serialization error",
            SerializedSizeMismatch { .. } => "Serialized size mismatch",
            SystemProgramAddressMismatch => "System program address must match",
        }
    }
}

impl From<CounterError> for ProgramError {
    fn from(e: CounterError) -> Self {
        use CounterError::*;

        let code = match e {
            CounterAddressMismatch => 1,
            CounterMustBeEmpty => 2,
            CounterMustBeOwnedBySystemProgram => 3,
            CounterMustBeWriteable => 4,
            CounterMustHaveZeroLamports => 5,
            InvalidInstructionDiscriminator(_) => 6,
            NotEnoughAccounts { .. } => 7,
            PayerMustBeSigner => 8,
            SerializeError => 9,
            SerializedSizeMismatch { .. } => 10,
            SystemProgramAddressMismatch => 11,
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
