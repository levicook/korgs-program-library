use {
    crate::{find_counter_address, AccountDiscriminator, AccountDiscriminatorError, CounterV1},
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
}

#[derive(Debug)]
pub enum IncrementCountV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    CounterMustBeWriteable,
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
    CounterMustBeOwnedByProgram,
    DeserializeError(ReadError),
    SerializeError(WriteError),
    SerializedSizeMismatch { expected: usize, observed: usize },
    AccountDiscriminatorError(AccountDiscriminatorError),
}

impl IncrementCountV1<'_> {
    /// Increments the count by 1. Only the owner may increment.
    ///
    /// Count saturates at `u64::MAX` and will not overflow.
    /// If the count is `u64::MAX`, the count will remain at `u64::MAX`.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`IncrementCountV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), IncrementCountV1Error> {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

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

        if !counter.is_writable() {
            return Err(IncrementCountV1Error::CounterMustBeWriteable);
        }

        let (expected_counter, _bump) = find_counter_address(program_id, owner.key());
        let observed_counter = counter.key();
        if observed_counter != &expected_counter {
            return Err(IncrementCountV1Error::CounterAddressMismatch {
                expected: expected_counter,
                observed: *observed_counter,
            });
        }

        if !counter.is_owned_by(program_id) {
            return Err(IncrementCountV1Error::CounterMustBeOwnedByProgram);
        }

        let counter_data = counter.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::CounterV1Account, &counter_data)?;

        Ok(Self { owner, counter })
    }
}

impl From<AccountDiscriminatorError> for IncrementCountV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        IncrementCountV1Error::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for IncrementCountV1Error {
    fn from(err: ProgramError) -> Self {
        IncrementCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for IncrementCountV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}

impl From<WriteError> for IncrementCountV1Error {
    fn from(err: WriteError) -> Self {
        Self::SerializeError(err)
    }
}
