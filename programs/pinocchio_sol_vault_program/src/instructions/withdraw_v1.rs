use {
    crate::{find_vault_v1, AccountDiscriminator, AccountDiscriminatorError, VaultV1},
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
    },
};

pub struct WithdrawV1<'a> {
    pub program_id: &'a Pubkey,
    pub accounts: WithdrawV1Accounts<'a>,
    pub args: WithdrawV1Args,
}

pub struct WithdrawV1Accounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub vault_bump: u8,
}

#[derive(Debug)]
pub enum WithdrawV1Error {
    ProgramError(ProgramError),
    NotEnoughAccounts {
        expected: usize,
        observed: usize,
    },
    OwnerMustBeSigner,
    OwnerMustBeWriteable,
    VaultMustBeWriteable,
    VaultAddressMismatch {
        expected: Pubkey,
        observed: Pubkey,
    },
    VaultMustBeOwnedByProgram,
    AccountDiscriminatorError(AccountDiscriminatorError),
    InvalidInstructionData,
    OwnerMismatch {
        expected: Pubkey,
        observed: Pubkey,
    },
    InsufficientFunds {
        available: u64,
        requested: u64,
    },
    WouldViolateRentMinimum {
        available: u64,
        requested: u64,
        rent_minimum: u64,
    },
}

impl WithdrawV1<'_> {
    /// Executes the withdraw instruction.
    ///
    /// Transfers SOL from the vault to the owner, protecting the rent-exempt minimum.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] containing a [`WithdrawV1Error`] if execution fails.
    pub fn execute(&self) -> Result<(), WithdrawV1Error> {
        let amount = self.args.amount;
        // Verify owner in vault state matches signer
        let vault_state = {
            let vault_data = self.accounts.vault.try_borrow_data()?;
            VaultV1::from_bytes(&vault_data)?
        };

        let owner_key = self.accounts.owner.key();
        if vault_state.owner() != *owner_key {
            return Err(WithdrawV1Error::OwnerMismatch {
                expected: vault_state.owner(),
                observed: *owner_key,
            });
        }

        // Calculate rent-exempt minimum
        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(VaultV1::size());

        // Check available balance
        let vault_lamports = *self.accounts.vault.try_borrow_lamports()?;
        let available = vault_lamports.saturating_sub(rent_exempt_minimum);

        if available < amount {
            return Err(WithdrawV1Error::InsufficientFunds {
                available,
                requested: amount,
            });
        }

        // Verify we won't violate rent minimum
        let remaining_after_withdraw = vault_lamports.saturating_sub(amount);
        if remaining_after_withdraw < rent_exempt_minimum {
            return Err(WithdrawV1Error::WouldViolateRentMinimum {
                available,
                requested: amount,
                rent_minimum: rent_exempt_minimum,
            });
        }

        // Transfer lamports directly (vault is owned by program, so we can manipulate lamports)
        {
            *self.accounts.vault.try_borrow_mut_lamports()? -= amount;
            *self.accounts.owner.try_borrow_mut_lamports()? += amount;
        }

        Ok(())
    }
}

impl<'a> TryFrom<(&'a Pubkey, &'a [AccountInfo], &[u8])> for WithdrawV1<'a> {
    type Error = WithdrawV1Error;

    fn try_from(
        (program_id, accounts, args): (&'a Pubkey, &'a [AccountInfo], &[u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = WithdrawV1Accounts::try_from((program_id, accounts))?;
        let args = WithdrawV1Args::parse(args)?;
        Ok(Self {
            program_id,
            accounts,
            args,
        })
    }
}

pub struct WithdrawV1Args {
    pub amount: u64,
}

impl WithdrawV1Args {
    /// Parses withdraw instruction arguments from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`WithdrawV1Error::InvalidInstructionData`] if the instruction data is invalid.
    pub fn parse(data: &[u8]) -> Result<Self, WithdrawV1Error> {
        if data.len() < 8 {
            return Err(WithdrawV1Error::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| WithdrawV1Error::InvalidInstructionData)?,
        );
        Ok(Self { amount })
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo])> for WithdrawV1Accounts<'a> {
    type Error = WithdrawV1Error;

    fn try_from((program_id, accounts): (&Pubkey, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let [owner, vault] = accounts else {
            return Err(WithdrawV1Error::NotEnoughAccounts {
                expected: 2,
                observed: accounts.len(),
            });
        };

        if !owner.is_signer() {
            return Err(WithdrawV1Error::OwnerMustBeSigner);
        }

        if !owner.is_writable() {
            return Err(WithdrawV1Error::OwnerMustBeWriteable);
        }

        if !vault.is_writable() {
            return Err(WithdrawV1Error::VaultMustBeWriteable);
        }

        let (expected_vault, vault_bump) = find_vault_v1(program_id, owner.key());
        let observed_vault = vault.key();
        if observed_vault != &expected_vault {
            return Err(WithdrawV1Error::VaultAddressMismatch {
                expected: expected_vault,
                observed: *observed_vault,
            });
        }

        if !vault.is_owned_by(program_id) {
            return Err(WithdrawV1Error::VaultMustBeOwnedByProgram);
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

impl From<AccountDiscriminatorError> for WithdrawV1Error {
    fn from(err: AccountDiscriminatorError) -> Self {
        Self::AccountDiscriminatorError(err)
    }
}

impl From<ProgramError> for WithdrawV1Error {
    fn from(err: ProgramError) -> Self {
        Self::ProgramError(err)
    }
}
