use {
    crate::{find_counter_address, AccountDiscriminator, CounterError, CounterV1},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    wincode::{ReadError, WriteError},
};

pub struct DecrementCountV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DecrementCountV1Accounts<'a>,
}

pub struct DecrementCountV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
}

#[derive(Debug, PartialEq)]
pub enum DecrementCountV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    CounterMustBeWriteable,
    CounterAddressMismatch,
    CounterMustBeOwnedByProgram,
    DeserializeError,
    SerializeError,
    OwnerMismatch,
    SerializedSizeMismatch { expected: usize, observed: usize },
}

impl DecrementCountV1<'_> {
    /// Executes the decrement count instruction.
    ///
    /// Decrements the counter's count by 1. Only the counter's owner may decrement.
    /// Uses saturating subtraction, so the count will not go below 0.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`DecrementCountV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), DecrementCountV1Error> {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

        if counter_state.discriminator != AccountDiscriminator::CounterV1Account {
            return Err(DecrementCountV1Error::DeserializeError);
        }

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(DecrementCountV1Error::OwnerMismatch);
        }

        counter_state.count = counter_state.count.saturating_sub(1);

        let serialized = counter_state.serialize()?;

        if serialized.len() != CounterV1::size() {
            return Err(DecrementCountV1Error::SerializedSizeMismatch {
                expected: CounterV1::size(),
                observed: serialized.len(),
            });
        }

        self.accounts
            .counter
            .try_borrow_mut_data()?
            .copy_from_slice(&serialized);

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for DecrementCountV1<'a> {
    type Error = DecrementCountV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = DecrementCountV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for DecrementCountV1Accounts<'a> {
    type Error = DecrementCountV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter] = accounts else {
            return Err(DecrementCountV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(DecrementCountV1Error::OwnerMustBeSigner);
        }

        let (counter_address, _bump) = find_counter_address(program_id, owner.key());

        if !counter.is_writable() {
            return Err(DecrementCountV1Error::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(DecrementCountV1Error::CounterAddressMismatch);
        }

        if !counter.is_owned_by(program_id) {
            return Err(DecrementCountV1Error::CounterMustBeOwnedByProgram);
        }

        Ok(Self { owner, counter })
    }
}

impl From<DecrementCountV1Error> for CounterError {
    fn from(err: DecrementCountV1Error) -> Self {
        match err {
            DecrementCountV1Error::ProgramError(pe) => CounterError::ProgramError(pe),
            _ => CounterError::DecrementCountV1(err),
        }
    }
}

impl From<ProgramError> for DecrementCountV1Error {
    fn from(err: ProgramError) -> Self {
        DecrementCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for DecrementCountV1Error {
    fn from(_: ReadError) -> Self {
        Self::DeserializeError
    }
}

impl From<WriteError> for DecrementCountV1Error {
    fn from(_: WriteError) -> Self {
        Self::SerializeError
    }
}
