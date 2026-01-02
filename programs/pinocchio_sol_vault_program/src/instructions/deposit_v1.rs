use {
    crate::{find_vault_v1, AccountDiscriminator, AccountDiscriminatorError, VaultV1},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    pinocchio_system::instructions::Transfer,
};

pub struct DepositV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: DepositV1Accounts<'a>,
    pub args: DepositV1Args,
}

pub struct DepositV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

#[derive(Debug)]
pub enum DepositV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts { expected: usize, observed: usize },
    OwnerMustBeSigner,
    OwnerMustBeWriteable,
    VaultMustBeWriteable,
    VaultAddressMismatch { expected: Pubkey, observed: Pubkey },
    VaultMustBeOwnedByProgram,
    SystemProgramAddressMismatch,
    AccountDiscriminatorError(AccountDiscriminatorError),
    InvalidInstructionData,
    OwnerMismatch { expected: Pubkey, observed: Pubkey },
}

impl DepositV1<'_> {
    /// Executes the deposit instruction.
    ///
    /// Transfers SOL from the owner to the vault account.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`DepositV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), DepositV1Error> {
        let amount = self.args.amount;
        // Verify owner in vault state matches signer
        let vault_state = {
            let vault_data = self.accounts.vault.try_borrow_data()?;
            VaultV1::from_bytes(&vault_data)?
        };

        let owner_key = self.accounts.owner.key();
        if vault_state.owner() != *owner_key {
            return Err(DepositV1Error::OwnerMismatch {
                expected: vault_state.owner(),
                observed: *owner_key,
            });
        }

        // Transfer lamports from owner to vault using CPI
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: amount,
        }
        .invoke()?;

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for DepositV1<'a> {
    type Error = DepositV1Error;

    fn try_from(
        (program_id, accounts, args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = DepositV1Accounts::try_from((program_id, accounts))?;
        let args = DepositV1Args::parse(args)?;
        Ok(Self {
            program_id,
            accounts,
            args,
        })
    }
}

pub struct DepositV1Args {
    pub amount: u64,
}

impl DepositV1Args {
    /// Parses deposit instruction arguments from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`DepositV1Error::InvalidInstructionData`] if the instruction data is invalid.
    pub fn parse(data: &[u8]) -> Result<Self, DepositV1Error> {
        if data.len() < 8 {
            return Err(DepositV1Error::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| DepositV1Error::InvalidInstructionData)?,
        );
        Ok(Self { amount })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for DepositV1Accounts<'a> {
    type Error = DepositV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, vault, system_program] = accounts else {
            return Err(DepositV1Error::NotEnoughAccounts {
                expected: 3,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(DepositV1Error::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(DepositV1Error::OwnerMustBeWriteable);
        }

        if !vault.is_writable() {
            return Err(DepositV1Error::VaultMustBeWriteable);
        }

        let (expected_vault, _bump) = find_vault_v1(program_id, owner.key());
        let observed_vault = vault.key();
        if observed_vault != &expected_vault {
            return Err(DepositV1Error::VaultAddressMismatch {
                expected: expected_vault,
                observed: *observed_vault,
            });
        }

        if !vault.is_owned_by(program_id) {
            return Err(DepositV1Error::VaultMustBeOwnedByProgram);
        }

        let vault_data = vault.try_borrow_data()?;
        AccountDiscriminator::check(AccountDiscriminator::VaultV1Account, &vault_data)?;

        if system_program.key() != &pinocchio_system::ID {
            return Err(DepositV1Error::SystemProgramAddressMismatch);
        }

        Ok(Self {
            owner,
            vault,
            system_program,
        })
    }
}

impl From<AccountDiscriminatorError> for DepositV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for DepositV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}
