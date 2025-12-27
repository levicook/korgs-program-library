use {
    crate::{find_counter_address, AccountDiscriminator, CounterError, CounterV1},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    wincode::{ReadError, WriteError},
};

pub struct IncrementCountV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: IncrementCountV1Accounts<'a>,
}

pub struct IncrementCountV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
}

#[derive(Debug, PartialEq)]
pub enum IncrementCountV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    OwnerMustBeWritable,
    CounterMustBeWriteable,
    CounterAddressMismatch,
    CounterMustBeOwnedByProgram,
    DeserializeError,
    SerializeError,
    OwnerMismatch,
    SerializedSizeMismatch { expected: usize, observed: usize },
}

impl IncrementCountV1<'_> {
    /// Executes the increment count instruction.
    ///
    /// Increments the counter's count by 1. Only the counter's owner may increment.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`IncrementCountV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), IncrementCountV1Error> {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

        if counter_state.discriminator != AccountDiscriminator::CounterV1Account {
            return Err(IncrementCountV1Error::DeserializeError);
        }

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(IncrementCountV1Error::OwnerMismatch);
        }

        counter_state.count = counter_state.count.saturating_add(1);

        let serialized = counter_state.serialize()?;

        if serialized.len() != CounterV1::size() {
            return Err(IncrementCountV1Error::SerializedSizeMismatch {
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for IncrementCountV1<'a> {
    type Error = IncrementCountV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = IncrementCountV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for IncrementCountV1Accounts<'a> {
    type Error = IncrementCountV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter] = accounts else {
            return Err(IncrementCountV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(IncrementCountV1Error::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(IncrementCountV1Error::OwnerMustBeWritable);
        }

        let (counter_address, counter_bump) = find_counter_address(program_id, owner.key());

        if !counter.is_writable() {
            return Err(IncrementCountV1Error::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(IncrementCountV1Error::CounterAddressMismatch);
        }

        if !counter.is_owned_by(program_id) {
            return Err(IncrementCountV1Error::CounterMustBeOwnedByProgram);
        }

        Ok(Self {
            owner,
            counter,
            counter_bump,
        })
    }
}

impl From<IncrementCountV1Error> for CounterError {
    fn from(err: IncrementCountV1Error) -> Self {
        match err {
            IncrementCountV1Error::ProgramError(pe) => CounterError::ProgramError(pe),
            _ => CounterError::IncrementCountV1(err),
        }
    }
}

impl From<ProgramError> for IncrementCountV1Error {
    fn from(err: ProgramError) -> Self {
        IncrementCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for IncrementCountV1Error {
    fn from(_: ReadError) -> Self {
        Self::DeserializeError
    }
}

impl From<WriteError> for IncrementCountV1Error {
    fn from(_: WriteError) -> Self {
        Self::SerializeError
    }
}
