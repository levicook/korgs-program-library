use {
    crate::{
        find_counter_address, AccountDiscriminator, AccountDiscriminatorError, CounterV1,
        DEACTIVATED_ACCOUNT_SIZE,
    },
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
    },
    wincode::ReadError,
};

pub struct DeactivateCounterV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DeactivateCounterV1Accounts<'a>,
}

pub struct DeactivateCounterV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
}

#[derive(Debug)]
pub enum DeactivateCounterV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    OwnerMustBeWriteable,
    CounterMustBeWriteable,
    CounterAddressMismatch,
    CounterMustBeOwnedByProgram,
    DeserializeError(ReadError),
    OwnerMismatch,
    AccountDiscriminatorError(AccountDiscriminatorError),
}

impl DeactivateCounterV1<'_> {
    /// Executes the deactivate counter instruction.
    ///
    /// Deactivates a counter account by:
    /// - Verifying the account discriminator is `CounterV1Account`
    /// - Verifying the owner matches the counter's stored owner
    /// - Marking the account as deactivated with the `DeactivatedAccount` discriminator
    /// - Resizing the account to 1 byte (discriminator only)
    /// - Transferring all non-rent-exempt lamports to the owner
    ///
    /// The account remains with 1 byte of data and the rent-exempt minimum balance,
    /// preventing reinitialization attacks while allowing the owner to reclaim most lamports.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`DeactivateCounterV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), DeactivateCounterV1Error> {
        let counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data)?
        };

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(DeactivateCounterV1Error::OwnerMismatch);
        }

        {
            let mut data = self.accounts.counter.try_borrow_mut_data()?;
            data[0] = u8::from(AccountDiscriminator::DeactivatedAccount);
        }

        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(DEACTIVATED_ACCOUNT_SIZE);

        self.accounts.counter.resize(DEACTIVATED_ACCOUNT_SIZE)?;

        let total_lamports = *self.accounts.counter.try_borrow_lamports()?;
        let lamports_to_transfer = total_lamports.saturating_sub(rent_exempt_minimum);

        {
            *self.accounts.counter.try_borrow_mut_lamports()? -= lamports_to_transfer;
            *self.accounts.owner.try_borrow_mut_lamports()? += lamports_to_transfer;
        }

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for DeactivateCounterV1<'a> {
    type Error = DeactivateCounterV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = DeactivateCounterV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for DeactivateCounterV1Accounts<'a> {
    type Error = DeactivateCounterV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter] = accounts else {
            return Err(DeactivateCounterV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(DeactivateCounterV1Error::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(DeactivateCounterV1Error::OwnerMustBeWriteable);
        }

        let (counter_address, counter_bump) = find_counter_address(program_id, owner.key());

        if !counter.is_writable() {
            return Err(DeactivateCounterV1Error::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(DeactivateCounterV1Error::CounterAddressMismatch);
        }

        if !counter.is_owned_by(program_id) {
            return Err(DeactivateCounterV1Error::CounterMustBeOwnedByProgram);
        }

        let counter_data = counter.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::CounterV1Account, &counter_data)?;

        Ok(Self {
            owner,
            counter,
            counter_bump,
        })
    }
}

impl From<AccountDiscriminatorError> for DeactivateCounterV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for DeactivateCounterV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}

impl From<ReadError> for DeactivateCounterV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}
