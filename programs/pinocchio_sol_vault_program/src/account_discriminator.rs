use crate::VaultV1;

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AccountDiscriminator {
    VaultV1Account = 1,
    DeactivatedAccount = 255,
}

#[derive(Debug, PartialEq)]
pub enum AccountDiscriminatorError {
    Missing,
    DiscriminatorMismatch {
        expected: AccountDiscriminator,
        observed: AccountDiscriminator,
    },
    SerializedSizeMismatch {
        expected: usize,
        observed: usize,
    },
    Invalid(u8),
}

impl From<AccountDiscriminator> for u8 {
    fn from(discriminator: AccountDiscriminator) -> Self {
        match discriminator {
            AccountDiscriminator::VaultV1Account => 1,
            AccountDiscriminator::DeactivatedAccount => 255,
        }
    }
}

impl TryFrom<u8> for AccountDiscriminator {
    type Error = AccountDiscriminatorError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            1 => Ok(AccountDiscriminator::VaultV1Account),
            255 => Ok(AccountDiscriminator::DeactivatedAccount),
            _ => Err(AccountDiscriminatorError::Invalid(byte)),
        }
    }
}

impl AccountDiscriminator {
    /// Checks that account data has the expected discriminator and size.
    ///
    /// Validates:
    /// - The discriminator byte matches the expected discriminator
    /// - The account data size matches the expected size for that discriminator type
    ///
    /// # Errors
    ///
    /// Returns [`AccountDiscriminatorError`] if the discriminator is missing,
    /// doesn't match the expected value, is invalid, or if the account size is incorrect.
    pub fn check(
        expected_discriminator: AccountDiscriminator,
        data: &[u8],
    ) -> Result<(), AccountDiscriminatorError> {
        let Some(discriminator_byte) = data.first() else {
            return Err(AccountDiscriminatorError::Missing);
        };

        let observed_discriminator = AccountDiscriminator::try_from(*discriminator_byte)?;
        if observed_discriminator != expected_discriminator {
            return Err(AccountDiscriminatorError::DiscriminatorMismatch {
                expected: expected_discriminator,
                observed: observed_discriminator,
            });
        }

        let expected_size = expected_discriminator.expected_account_size();
        let observed_size = data.len();
        if observed_size != expected_size {
            return Err(AccountDiscriminatorError::SerializedSizeMismatch {
                expected: expected_size,
                observed: observed_size,
            });
        }

        Ok(())
    }

    fn expected_account_size(self) -> usize {
        match self {
            AccountDiscriminator::VaultV1Account => VaultV1::size(),
            AccountDiscriminator::DeactivatedAccount => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_discriminator_serialization() {
        // Verify VaultV1Account serializes to 1
        let vault_disc = AccountDiscriminator::VaultV1Account;
        let serialized = u8::from(vault_disc);
        assert_eq!(
            serialized, 1,
            "VaultV1Account should serialize to byte 1, got {}",
            serialized
        );

        // Verify DeactivatedAccount serializes to 255
        let deactivated_disc = AccountDiscriminator::DeactivatedAccount;
        let serialized = u8::from(deactivated_disc);
        assert_eq!(
            serialized, 255,
            "DeactivatedAccount should serialize to byte 255, got {}",
            serialized
        );
    }

    #[test]
    fn test_account_discriminator_uses_one_byte_of_memory() {
        assert_eq!(
            core::mem::size_of::<AccountDiscriminator>(),
            1,
            "AccountDiscriminator should be 1 byte with repr(u8)"
        );
    }
}
