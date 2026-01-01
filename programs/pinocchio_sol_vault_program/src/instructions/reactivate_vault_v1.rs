use {
    crate::{find_vault_v1, AccountDiscriminator, AccountDiscriminatorError, VaultV1},
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
    },
    pinocchio_system::instructions::Transfer,
};

pub struct ReactivateVaultV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: ReactivateVaultV1Accounts<'a>,
}

pub struct ReactivateVaultV1Accounts<'a> {
    pub payer: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub vault_bump: u8,
    pub system_program: &'a AccountInfo,
}

#[derive(Debug)]
pub enum ReactivateVaultV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    PayerMustBeSigner,
    PayerMustBeWriteable,
    VaultMustBeWriteable,
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
    SystemProgramAddressMismatch,
    AccountDiscriminatorError(AccountDiscriminatorError),
    SerializedSizeMismatch { expected: usize, observed: usize },
}

impl ReactivateVaultV1<'_> {
    /// Executes the reactivate vault instruction.
    ///
    /// Reactivates a deactivated vault account by:
    /// - Verifying the account discriminator is `DeactivatedAccount`
    /// - Resizing the account from 1 byte to `VaultV1::size()`
    /// - Transferring additional lamports from the payer to cover the increased rent requirement
    /// - Initializing the account with `VaultV1Account` discriminator and owner state
    ///
    /// The vault is restored to its initial state.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`ReactivateVaultV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), ReactivateVaultV1Error> {
        let rent = Rent::get()?;
        let rent_exempt_minimum_vault = rent.minimum_balance(VaultV1::size());

        // Calculate additional lamports needed
        let current_lamports = *self.accounts.vault.try_borrow_lamports()?;
        let additional_lamports_needed = rent_exempt_minimum_vault.saturating_sub(current_lamports);

        // Transfer lamports from payer to vault if needed using CPI
        if additional_lamports_needed > 0 {
            Transfer {
                from: self.accounts.payer,
                to: self.accounts.vault,
                lamports: additional_lamports_needed,
            }
            .invoke()?;
        }

        // Resize the account from 1 byte to VaultV1::size()
        self.accounts.vault.resize(VaultV1::size())?;

        // Initialize the vault state
        let owner = self.accounts.payer.key();
        let mut owner_bytes = [0u8; 32];
        owner_bytes.copy_from_slice(owner.as_ref());

        let state = VaultV1 {
            discriminator: AccountDiscriminator::VaultV1Account,
            owner: owner_bytes,
            bump: self.accounts.vault_bump,
        };

        let serialized = state.to_bytes();

        if serialized.len() != VaultV1::size() {
            return Err(ReactivateVaultV1Error::SerializedSizeMismatch {
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

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for ReactivateVaultV1<'a> {
    type Error = ReactivateVaultV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = ReactivateVaultV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for ReactivateVaultV1Accounts<'a> {
    type Error = ReactivateVaultV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [payer, vault, system_program] = accounts else {
            return Err(ReactivateVaultV1Error::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !payer.is_signer() {
            return Err(ReactivateVaultV1Error::PayerMustBeSigner);
        }

        if !payer.is_writable() {
            return Err(ReactivateVaultV1Error::PayerMustBeWriteable);
        }

        if !vault.is_writable() {
            return Err(ReactivateVaultV1Error::VaultMustBeWriteable);
        }

        let (expected_vault, vault_bump) = find_vault_v1(program_id, payer.key());
        let observed_vault = vault.key();
        if observed_vault != &expected_vault {
            return Err(ReactivateVaultV1Error::VaultAddressMismatch {
                expected: expected_vault,
                observed: *observed_vault,
            });
        }

        if system_program.key() != &pinocchio_system::ID {
            return Err(ReactivateVaultV1Error::SystemProgramAddressMismatch);
        }

        let vault_data = vault.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::DeactivatedAccount, &vault_data)?;

        Ok(Self {
            payer,
            vault,
            vault_bump,
            system_program,
        })
    }
}

impl From<AccountDiscriminatorError> for ReactivateVaultV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for ReactivateVaultV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}
