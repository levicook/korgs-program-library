use {
    crate::{find_counter_v1, AccountDiscriminator, AccountDiscriminatorError, CounterV1},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    wincode::{ReadError, SchemaRead, SchemaWrite, WriteError},
};

pub struct SetCountV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: SetCountV1Accounts<'a>,
    pub args: SetCountV1Args,
}

pub struct SetCountV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
}

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct SetCountV1Args {
    pub count: u64,
}

#[derive(Debug)]
pub enum SetCountV1Error {
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

impl SetCountV1<'_> {
    /// Executes the set count instruction.
    ///
    /// Sets the counter's count to the specified value. Only the counter's owner may set the count.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`SetCountV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), SetCountV1Error> {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

        counter_state.count = self.args.count;

        let serialized = counter_state.serialize()?;

        if serialized.len() != CounterV1::size() {
            return Err(SetCountV1Error::SerializedSizeMismatch {
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for SetCountV1<'a> {
    type Error = SetCountV1Error;

    fn try_from(
        (program_id, accounts, args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = SetCountV1Accounts::try_from((program_id, accounts))?;
        let args = SetCountV1Args::deserialize(args)?;
        Ok(Self {
            program_id,
            accounts,
            args,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for SetCountV1Accounts<'a> {
    type Error = SetCountV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter] = accounts else {
            return Err(SetCountV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(SetCountV1Error::OwnerMustBeSigner);
        }

        if !counter.is_writable() {
            return Err(SetCountV1Error::CounterMustBeWriteable);
        }

        let (expected_counter, _bump) = find_counter_v1(program_id, owner.key());
        let observed_counter = counter.key();
        if observed_counter != &expected_counter {
            return Err(SetCountV1Error::CounterAddressMismatch {
                expected: expected_counter,
                observed: *observed_counter,
            });
        }

        let counter_data = counter.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::CounterV1Account, &counter_data)?;

        Ok(Self { owner, counter })
    }
}

impl SetCountV1Args {
    /// Deserializes the instruction arguments from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`wincode::ReadError`] if deserialization fails.
    pub fn deserialize(src: &[u8]) -> Result<Self, ReadError> {
        wincode::deserialize(src)
    }
}

impl From<AccountDiscriminatorError> for SetCountV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        SetCountV1Error::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for SetCountV1Error {
    fn from(err: ProgramError) -> Self {
        SetCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for SetCountV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}

impl From<WriteError> for SetCountV1Error {
    fn from(err: WriteError) -> Self {
        Self::SerializeError(err)
    }
}
