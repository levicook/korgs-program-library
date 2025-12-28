use {
    crate::{find_counter_address, AccountDiscriminator, CounterV1, COUNTER_SEED},
    pinocchio::{
        account_info::AccountInfo, instruction::Signer, program_error::ProgramError,
        pubkey::Pubkey, seeds,
    },
    pinocchio_system::create_account_with_minimum_balance_signed,
    wincode::{ReadError, WriteError},
};

pub struct InitializeCounterV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: InitializeCounterV1Accounts<'a>,
}

pub struct InitializeCounterV1Accounts<'a> {
    pub payer: &'a AccountInfo,
    pub counter: &'a AccountInfo,
    pub counter_bump: u8,
    pub system_program: &'a AccountInfo,
}

#[derive(Debug)]
pub enum InitializeCounterV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    PayerMustBeSigner,
    CounterMustBeWriteable,
    CounterAddressMismatch { expected: Pubkey, observed: Pubkey },
    CounterMustBeEmpty,
    CounterMustHaveZeroLamports,
    CounterMustBeOwnedBySystemProgram,
    SystemProgramAddressMismatch,
    DeserializeError(ReadError),
    SerializeError(WriteError),
    SerializedSizeMismatch { expected: usize, observed: usize },
}

impl InitializeCounterV1<'_> {
    /// Executes the initialize counter instruction.
    ///
    /// Initializes a new counter account owned by the program with the payer as the owner.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`InitializeCounterV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), InitializeCounterV1Error> {
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
            discriminator: AccountDiscriminator::CounterV1Account,
            owner: *owner,
            bump: self.accounts.counter_bump,
            count: 0,
            reserved: [0; 31],
        };

        let serialized = state.serialize()?;

        if serialized.len() != CounterV1::size() {
            return Err(InitializeCounterV1Error::SerializedSizeMismatch {
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for InitializeCounterV1<'a> {
    type Error = InitializeCounterV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = InitializeCounterV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for InitializeCounterV1Accounts<'a> {
    type Error = InitializeCounterV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [payer, counter, system_program] = accounts else {
            return Err(InitializeCounterV1Error::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !payer.is_signer() {
            return Err(InitializeCounterV1Error::PayerMustBeSigner);
        }

        if !counter.is_writable() {
            return Err(InitializeCounterV1Error::CounterMustBeWriteable);
        }

        let (expected_counter, counter_bump) = find_counter_address(program_id, payer.key());
        let observed_counter = counter.key();
        if observed_counter != &expected_counter {
            return Err(InitializeCounterV1Error::CounterAddressMismatch {
                expected: expected_counter,
                observed: *observed_counter,
            });
        }

        if !counter.data_is_empty() {
            return Err(InitializeCounterV1Error::CounterMustBeEmpty);
        }

        if counter.lamports() > 0 {
            return Err(InitializeCounterV1Error::CounterMustHaveZeroLamports);
        }

        if !counter.is_owned_by(&pinocchio_system::ID) {
            return Err(InitializeCounterV1Error::CounterMustBeOwnedBySystemProgram);
        }

        if system_program.key() != &pinocchio_system::ID {
            return Err(InitializeCounterV1Error::SystemProgramAddressMismatch);
        }

        Ok(Self {
            payer,
            counter,
            counter_bump,
            system_program,
        })
    }
}

impl From<ProgramError> for InitializeCounterV1Error {
    fn from(err: ProgramError) -> Self {
        InitializeCounterV1Error::ProgramError(err)
    }
}

impl From<ReadError> for InitializeCounterV1Error {
    fn from(err: ReadError) -> Self {
        Self::DeserializeError(err)
    }
}

impl From<WriteError> for InitializeCounterV1Error {
    fn from(err: WriteError) -> Self {
        Self::SerializeError(err)
    }
}
