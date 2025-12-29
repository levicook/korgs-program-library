use {
    crate::{find_counter_v1, AccountDiscriminator, AccountDiscriminatorError, CounterV1},
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
    },
    pinocchio_system::instructions::Transfer,
    wincode::{ReadError, WriteError},
};

pub struct ReactivateCounterV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: ReactivateCounterV1Accounts<'a>,
}

pub struct ReactivateCounterV1Accounts<'a> {
    pub payer: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
    pub system_program: &'a AccountInfo,
}

#[derive(Debug)]
pub enum ReactivateCounterV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    PayerMustBeSigner,
    PayerMustBeWriteable,
    CounterMustBeWriteable,
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
    SystemProgramAddressMismatch,
    DeserializeError(ReadError),
    SerializeError(WriteError),
    SerializedSizeMismatch { expected: usize, observed: usize },
    AccountDiscriminatorError(AccountDiscriminatorError),
}

impl ReactivateCounterV1<'_> {
    /// Executes the reactivate counter instruction.
    ///
    /// Reactivates a deactivated counter account by:
    /// - Verifying the account discriminator is `DeactivatedAccount`
    /// - Resizing the account from 1 byte to `CounterV1::size()`
    /// - Transferring additional lamports from the payer to cover the increased rent requirement
    /// - Initializing the account with `CounterV1Account` discriminator and default state (count = 0)
    ///
    /// The counter is restored to its initial state with count = 0.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`ReactivateCounterV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), ReactivateCounterV1Error> {
        let rent = Rent::get()?;
        let rent_exempt_minimum_counter = rent.minimum_balance(CounterV1::size());

        // Calculate additional lamports needed
        let current_lamports = *self.accounts.counter.try_borrow_lamports()?;
        let additional_lamports_needed =
            rent_exempt_minimum_counter.saturating_sub(current_lamports);

        // Transfer lamports from payer to counter if needed using CPI
        if additional_lamports_needed > 0 {
            Transfer {
                from: self.accounts.payer,
                to: self.accounts.counter,
                lamports: additional_lamports_needed,
            }
            .invoke()?;
        }

        // Resize the account from 1 byte to CounterV1::size()
        self.accounts.counter.resize(CounterV1::size())?;

        // Initialize the counter state
        let owner = self.accounts.payer.key();
        let state = CounterV1 {
            discriminator: AccountDiscriminator::CounterV1Account,
            owner: *owner,
            bump: self.accounts.counter_bump,
            count: 0,
        };

        let serialized = state.serialize()?;

        if serialized.len() != CounterV1::size() {
            return Err(ReactivateCounterV1Error::SerializedSizeMismatch {
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for ReactivateCounterV1<'a> {
    type Error = ReactivateCounterV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = ReactivateCounterV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for ReactivateCounterV1Accounts<'a> {
    type Error = ReactivateCounterV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [payer, counter, system_program] = accounts else {
            return Err(ReactivateCounterV1Error::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !payer.is_signer() {
            return Err(ReactivateCounterV1Error::PayerMustBeSigner);
        }

        if !payer.is_writable() {
            return Err(ReactivateCounterV1Error::PayerMustBeWriteable);
        }

        if !counter.is_writable() {
            return Err(ReactivateCounterV1Error::CounterMustBeWriteable);
        }

        let (expected_counter, counter_bump) = find_counter_v1(program_id, payer.key());
        let observed_counter = counter.key();
        if observed_counter != &expected_counter {
            return Err(ReactivateCounterV1Error::CounterAddressMismatch {
                expected: expected_counter,
                observed: *observed_counter,
            });
        }

        if system_program.key() != &pinocchio_system::ID {
            return Err(ReactivateCounterV1Error::SystemProgramAddressMismatch);
        }

        let counter_data = counter.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::DeactivatedAccount, &counter_data)?;

        Ok(Self {
            payer,
            counter,
            counter_bump,
            system_program,
        })
    }
}

impl From<AccountDiscriminatorError> for ReactivateCounterV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for ReactivateCounterV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}

impl From<ReadError> for ReactivateCounterV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}

impl From<WriteError> for ReactivateCounterV1Error {
    fn from(err: WriteError) -> Self {
        Self::SerializeError(err)
    }
}
