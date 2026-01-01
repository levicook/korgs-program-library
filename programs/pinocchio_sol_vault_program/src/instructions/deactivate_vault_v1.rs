use {
    crate::{find_vault_v1, AccountDiscriminator, AccountDiscriminatorError},
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
    },
};

pub const DEACTIVATED_ACCOUNT_SIZE: usize = 1;

pub struct DeactivateVaultV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DeactivateVaultV1Accounts<'a>,
}

pub struct DeactivateVaultV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub vault_bump: u8,
}

#[derive(Debug)]
pub enum DeactivateVaultV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    OwnerMustBeWriteable,
    VaultMustBeWriteable,
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
    AccountDiscriminatorError(AccountDiscriminatorError),
}

impl DeactivateVaultV1<'_> {
    /// Executes the deactivate vault instruction.
    ///
    /// Deactivates a vault account by:
    /// - Verifying the account discriminator is `VaultV1Account`
    /// - Marking the account as deactivated with the `DeactivatedAccount` discriminator
    /// - Resizing the account to 1 byte (discriminator only)
    /// - Transferring all non-rent-exempt lamports to the owner
    ///
    /// The account remains with 1 byte of data and the rent-exempt minimum balance,
    /// preventing reinitialization attacks while allowing the owner to reclaim most lamports.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`DeactivateVaultV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), DeactivateVaultV1Error> {
        {
            let mut data = self.accounts.vault.try_borrow_mut_data()?;
            data[0] = u8::from(AccountDiscriminator::DeactivatedAccount);
        }

        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(DEACTIVATED_ACCOUNT_SIZE);

        self.accounts.vault.resize(DEACTIVATED_ACCOUNT_SIZE)?;

        let total_lamports = *self.accounts.vault.try_borrow_lamports()?;
        let lamports_to_transfer = total_lamports.saturating_sub(rent_exempt_minimum);

        {
            *self.accounts.vault.try_borrow_mut_lamports()? -= lamports_to_transfer;
            *self.accounts.owner.try_borrow_mut_lamports()? += lamports_to_transfer;
        }

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for DeactivateVaultV1<'a> {
    type Error = DeactivateVaultV1Error;

    fn try_from(
        (program_id, accounts, _args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = DeactivateVaultV1Accounts::try_from((program_id, accounts))?;
        Ok(Self {
            program_id,
            accounts,
        })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for DeactivateVaultV1Accounts<'a> {
    type Error = DeactivateVaultV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, vault] = accounts else {
            return Err(DeactivateVaultV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(DeactivateVaultV1Error::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(DeactivateVaultV1Error::OwnerMustBeWriteable);
        }

        if !vault.is_writable() {
            return Err(DeactivateVaultV1Error::VaultMustBeWriteable);
        }

        let (expected_vault, vault_bump) = find_vault_v1(program_id, owner.key());
        let observed_vault = vault.key();
        if observed_vault != &expected_vault {
            return Err(DeactivateVaultV1Error::VaultAddressMismatch {
                expected: expected_vault,
                observed: *observed_vault,
            });
        }

        let vault_data = vault.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::VaultV1Account, &vault_data)?;

        Ok(Self {
            owner,
            vault,
            vault_bump,
        })
    }
}

impl From<AccountDiscriminatorError> for DeactivateVaultV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for DeactivateVaultV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}
