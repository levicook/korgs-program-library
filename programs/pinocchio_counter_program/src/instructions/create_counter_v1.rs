use pinocchio::{
    account_info::AccountInfo, instruction::Signer, pubkey::Pubkey, seeds, ProgramResult,
};
use pinocchio_system::create_account_with_minimum_balance_signed;

use crate::{find_counter_address, CounterError, CounterV1, COUNTER_SEED};

pub struct CreateCounterV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: CreateCounterV1Accounts<'a>,
}

pub struct CreateCounterV1Accounts<'a> {
    pub payer: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
    pub system_program: &'a AccountInfo,
}

impl CreateCounterV1<'_> {
    /// Executes the create counter instruction.
    ///
    /// Creates a new counter account owned by the program with the payer as the owner.
    ///
    /// # Errors
    ///
    /// Returns a [`ProgramResult`] error if:
    /// - Account creation fails (insufficient funds, account already exists, etc.)
    /// - State serialization fails
    /// - Serialized state size doesn't match expected size
    /// - Data mutation fails
    pub fn execute(&self) -> ProgramResult {
        let owner = self.accounts.payer.key();
        let owner_ref = owner.as_ref();
        let bump_ref = &[self.accounts.counter_bump];
        let seeds = seeds!(COUNTER_SEED, owner_ref, bump_ref);
        let signer = Signer::from(&seeds);

        create_account_with_minimum_balance_signed(
            self.accounts.counter, // account
            CounterV1::size(),     // space,
            self.program_id,       // account owner
            self.accounts.payer,
            None,
            &[signer],
        )?;

        let state = CounterV1 {
            owner: *owner,
            bump: self.accounts.counter_bump,
            ..Default::default()
        };

        let serialized = state
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for CreateCounterV1<'a> {
    type Error = CounterError;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = CreateCounterV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for CreateCounterV1Accounts<'a> {
    type Error = CounterError;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [payer, counter, system_program] = accounts else {
            return Err(CounterError::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !payer.is_signer() {
            return Err(CounterError::PayerMustBeSigner);
        }

        let (counter_address, counter_bump) = find_counter_address(program_id, payer.key());

        if !counter.is_writable() {
            return Err(CounterError::CounterMustBeWriteable);
        }

        if counter.key() != &counter_address {
            return Err(CounterError::CounterAddressMismatch);
        }

        if !counter.data_is_empty() {
            return Err(CounterError::CounterMustBeEmpty);
        }

        if counter.lamports() > 0 {
            return Err(CounterError::CounterMustHaveZeroLamports);
        }

        if !counter.is_owned_by(&pinocchio_system::ID) {
            return Err(CounterError::CounterMustBeOwnedBySystemProgram);
        }

        if system_program.key() != &pinocchio_system::ID {
            return Err(CounterError::SystemProgramAddressMismatch);
        }

        Ok(Self {
            payer,
            counter,
            counter_bump,
            system_program,
        })
    }
}
