use {
    crate::{find_counter_address, AccountDiscriminator, CounterError, CounterV1},
    pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult},
    wincode::{ReadError, SchemaRead, SchemaWrite},
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

impl SetCountV1<'_> {
    /// Executes the set count instruction.
    ///
    /// Sets the counter's count to the specified value. Only the counter's owner may set the count.
    ///
    /// # Errors
    ///
    /// Returns a [`ProgramResult`] containing a [`CounterError`] if execution fails.
    pub fn execute(&self) -> ProgramResult {
        let mut counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data).map_err(|_| CounterError::SerializeError)?
        };

        if counter_state.discriminator != AccountDiscriminator::CounterV1 {
            return Err(CounterError::SerializeError.into());
        }

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(CounterError::OwnerMismatch.into());
        }

        counter_state.count = self.args.count;

        let serialized = counter_state
            .serialize()
            .map_err(|_| CounterError::SerializeError)?;

        if serialized.len() != CounterV1::size() {
            let counter_error = CounterError::SerializedSizeMismatch {
                expected: CounterV1::size(),
                observed: serialized.len(),
            };
            return Err(counter_error.into());
        }

        self.accounts
            .counter
            .try_borrow_mut_data()?
            .copy_from_slice(&serialized);

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for SetCountV1<'a> {
    type Error = CounterError;

    fn try_from(
        (program_id, accounts, args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = SetCountV1Accounts::try_from((program_id, accounts))?;
        let args = SetCountV1Args::deserialize(args).map_err(|_| CounterError::SerializeError)?;
        Ok(Self {
            program_id,
            accounts,
            args,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for SetCountV1Accounts<'a> {
    type Error = CounterError;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter] = accounts else {
            return Err(CounterError::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(CounterError::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(CounterError::OwnerMustBeWritable);
        }

        let (counter_address, counter_bump) = find_counter_address(program_id, owner.key());

        if !counter.is_writable() {
            return Err(CounterError::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(CounterError::CounterAddressMismatch);
        }

        if !counter.is_owned_by(program_id) {
            return Err(CounterError::CounterMustBeOwnedByProgram);
        }

        Ok(Self {
            owner,
            counter,
            counter_bump,
        })
    }
}
