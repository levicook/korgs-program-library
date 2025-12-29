use {
    crate::{find_counter_v1, AccountDiscriminator, AccountDiscriminatorError, CounterV1},
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

#[derive(Debug)]
pub enum DecrementCountV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    CounterMustBeWriteable,
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
    DeserializeError(ReadError),
    SerializeError(WriteError),
    SerializedSizeMismatch { expected: usize, observed: usize },
    AccountDiscriminatorError(AccountDiscriminatorError),
}

impl DecrementCountV1<'_> {
    /// Decrements count by 1. Only the owner may decrement.
    ///
    /// Count saturates at `0` and will not underflow.
    /// If the count is `0`, the count will remain at `0`.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`DecrementCountV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), DecrementCountV1Error> {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

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

        if !counter.is_writable() {
            return Err(DecrementCountV1Error::CounterMustBeWriteable);
        }

        let (expected_counter, _bump) = find_counter_v1(program_id, owner.key());
        let observed_counter = counter.key();
        if observed_counter != &expected_counter {
            return Err(DecrementCountV1Error::CounterAddressMismatch {
                expected: expected_counter,
                observed: *observed_counter,
            });
        }

        let counter_data = counter.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::CounterV1Account, &counter_data)?;

        Ok(Self { owner, counter })
    }
}

impl From<AccountDiscriminatorError> for DecrementCountV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        DecrementCountV1Error::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for DecrementCountV1Error {
    fn from(err: ProgramError) -> Self {
        DecrementCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for DecrementCountV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}

impl From<WriteError> for DecrementCountV1Error {
    fn from(err: WriteError) -> Self {
        Self::SerializeError(err)
    }
}
