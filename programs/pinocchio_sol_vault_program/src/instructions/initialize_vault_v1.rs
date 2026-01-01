use {
    crate::{find_vault_v1, AccountDiscriminator, VaultV1, VAULT_V1_SEED},
    pinocchio::{
        account_info::AccountInfo, instruction::Signer, program_error::ProgramError,
        pubkey::Pubkey, seeds,
    },
    pinocchio_system::create_account_with_minimum_balance_signed,
};

pub struct InitializeVaultV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: InitializeVaultV1Accounts<'a>,
}

pub struct InitializeVaultV1Accounts<'a> {
    pub payer: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub vault_bump: u8,
    pub system_program: &'a AccountInfo,
}

#[derive(Debug)]
pub enum InitializeVaultV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    PayerMustBeSigner,
    PayerMustBeWriteable,
    VaultMustBeWriteable,
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
    VaultMustBeEmpty,
    VaultMustHaveZeroLamports,
    VaultMustBeOwnedBySystemProgram,
    SystemProgramAddressMismatch,
    SerializedSizeMismatch { expected: usize, observed: usize },
}

impl InitializeVaultV1<'_> {
    /// Executes the initialize vault instruction.
    ///
    /// Initializes a new vault account owned by the program with the payer as the owner.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`InitializeVaultV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), InitializeVaultV1Error> {
        let owner = self.accounts.payer.key();
        let owner_ref = owner.as_ref();
        let bump_ref = &[self.accounts.vault_bump];
        let seeds = seeds!(VAULT_V1_SEED, owner_ref, bump_ref);
        let signer = Signer::from(&seeds);

        create_account_with_minimum_balance_signed(
            self.accounts.vault, // account
            VaultV1::size(),     // space
            self.program_id,     // account owner
            self.accounts.payer,
            None,
            &[signer],
        )?;

        let mut owner_bytes = [0u8; 32];
        owner_bytes.copy_from_slice(owner.as_ref());

        let state = VaultV1 {
            discriminator: AccountDiscriminator::VaultV1Account,
            owner: owner_bytes,
            bump: self.accounts.vault_bump,
        };

        let serialized = state.to_bytes();

        if serialized.len() != VaultV1::size() {
            return Err(InitializeVaultV1Error::SerializedSizeMismatch {
                expected: VaultV1::size(),
                observed: serialized.len(),
            });
        }

        self.accounts
            .vault
            .try_borrow_mut_data()?
            .copy_from_slice(&serialized);

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for InitializeVaultV1<'a> {
    type Error = InitializeVaultV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = InitializeVaultV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for InitializeVaultV1Accounts<'a> {
    type Error = InitializeVaultV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [payer, vault, system_program] = accounts else {
            return Err(InitializeVaultV1Error::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !payer.is_signer() {
            return Err(InitializeVaultV1Error::PayerMustBeSigner);
        }

        if !payer.is_writable() {
            return Err(InitializeVaultV1Error::PayerMustBeWriteable);
        }

        if !vault.is_writable() {
            return Err(InitializeVaultV1Error::VaultMustBeWriteable);
        }

        let (expected_vault, vault_bump) = find_vault_v1(program_id, payer.key());
        let observed_vault = vault.key();
        if observed_vault != &expected_vault {
            return Err(InitializeVaultV1Error::VaultAddressMismatch {
                expected: expected_vault,
                observed: *observed_vault,
            });
        }

        if !vault.data_is_empty() {
            return Err(InitializeVaultV1Error::VaultMustBeEmpty);
        }

        if vault.lamports() > 0 {
            return Err(InitializeVaultV1Error::VaultMustHaveZeroLamports);
        }

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(InitializeVaultV1Error::VaultMustBeOwnedBySystemProgram);
        }

        if system_program.key() != &pinocchio_system::ID {
            return Err(InitializeVaultV1Error::SystemProgramAddressMismatch);
        }

        Ok(Self {
            payer,
            vault,
            vault_bump,
            system_program,
        })
    }
}

impl From<ProgramError> for InitializeVaultV1Error {
    fn from(err: ProgramError) -> Self {
        InitializeVaultV1Error::ProgramError(err)
    }
}
