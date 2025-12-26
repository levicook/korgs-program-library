use {
    crate::{find_counter_address, AccountDiscriminator, CounterError, CounterV1},
    pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult},
};

pub struct DecrementCountV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DecrementCountV1Accounts<'a>,
}

pub struct DecrementCountV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
}

impl DecrementCountV1<'_> {
    /// Executes the decrement count instruction.
    ///
    /// Decrements the counter's count by 1. Only the counter's owner may decrement.
    /// Uses saturating subtraction, so the count will not go below 0.
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

        counter_state.count = counter_state.count.saturating_sub(1);

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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for DecrementCountV1<'a> {
    type Error = CounterError;

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
