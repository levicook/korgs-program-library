use {
    crate::{
        find_counter_address, AccountDiscriminator, CounterError, CounterV1,
        DEACTIVATED_ACCOUNT_SIZE,
    },
    pinocchio::{
        account_info::AccountInfo,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
};

pub struct DeactivateCounterV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DeactivateCounterV1Accounts<'a>,
}

pub struct DeactivateCounterV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
    pub system_program: &'a AccountInfo,
}

impl DeactivateCounterV1<'_> {
    /// Executes the deactivate counter instruction.
    ///
    /// Deactivates a counter account by:
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
    /// Returns a [`ProgramResult`] containing a [`CounterError`] if execution fails.
    pub fn execute(&self) -> ProgramResult {
        let counter_state = {
            let counter_data = self.accounts.counter.try_borrow_data()?;
            CounterV1::deserialize(&counter_data).map_err(|_| CounterError::SerializeError)?
        };

        if counter_state.owner != *self.accounts.owner.key() {
            return Err(CounterError::OwnerMismatch.into());
        }

        {
            let mut data = self.accounts.counter.try_borrow_mut_data()?;
            data[0] = AccountDiscriminator::DeactivatedAccount as u8;
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
    type Error = CounterError;

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
    type Error = CounterError;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, counter, system_program] = accounts else {
            return Err(CounterError::NotEnoughAccounts {
                expected: 3,
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

        if system_program.key() != &pinocchio_system::ID {
            return Err(CounterError::SystemProgramAddressMismatch);
        }

        Ok(Self {
            owner,
            counter,
            counter_bump,
            system_program,
        })
    }
}
