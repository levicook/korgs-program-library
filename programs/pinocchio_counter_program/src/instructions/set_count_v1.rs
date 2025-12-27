use {
    crate::{find_counter_address, AccountDiscriminator, CounterError, CounterV1},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    wincode::{ReadError, SchemaRead, SchemaWrite, WriteError},
};

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct SetCountV1Args {
    pub count: u64,
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

pub struct SetCountV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: SetCountV1Accounts<'a>,
    pub args: SetCountV1Args,
}

pub struct SetCountV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
}

#[derive(Debug, PartialEq)]
pub enum SetCountV1Error {
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

        if counter_state.discriminator != AccountDiscriminator::CounterV1Account {
            return Err(SetCountV1Error::DeserializeError);
        }

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(SetCountV1Error::OwnerMismatch);
        }

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

        if !owner.is_writable() {
            return Err(SetCountV1Error::OwnerMustBeWritable);
        }

        let (counter_address, counter_bump) = find_counter_address(program_id, owner.key());

        if !counter.is_writable() {
            return Err(SetCountV1Error::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(SetCountV1Error::CounterAddressMismatch);
        }

        if !counter.is_owned_by(program_id) {
            return Err(SetCountV1Error::CounterMustBeOwnedByProgram);
        }

        Ok(Self {
            owner,
            counter,
            counter_bump,
        })
    }
}

impl From<SetCountV1Error> for CounterError {
    fn from(err: SetCountV1Error) -> Self {
        match err {
            SetCountV1Error::ProgramError(pe) => CounterError::ProgramError(pe),
            _ => CounterError::SetCountV1(err),
        }
    }
}

impl From<ProgramError> for SetCountV1Error {
    fn from(err: ProgramError) -> Self {
        SetCountV1Error::ProgramError(err)
    }
}

impl From<ReadError> for SetCountV1Error {
    fn from(_: ReadError) -> Self {
        Self::DeserializeError
    }
}

impl From<WriteError> for SetCountV1Error {
    fn from(_: WriteError) -> Self {
        Self::SerializeError
    }
}
