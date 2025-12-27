use {
    pinocchio::pubkey::Pubkey,
    wincode::{SchemaRead, SchemaWrite},
};

pub const DEACTIVATED_ACCOUNT_SIZE: usize = 1;

#[repr(u8)]
#[derive(Debug, PartialEq, SchemaRead, SchemaWrite)]
pub enum AccountDiscriminator {
    #[wincode(tag = 1)]
    CounterV1Account = 1,

    #[wincode(tag = 255)]
    DeactivatedAccount = 255,
}

#[derive(Debug, PartialEq)]
pub enum AccountDiscriminatorError {
    Missing,
    Mismatch {
        expected: AccountDiscriminator,
        observed: AccountDiscriminator,
    },
    Invalid(u8),
}

impl From<AccountDiscriminator> for u8 {
    fn from(discriminator: AccountDiscriminator) -> Self {
        match discriminator {
            AccountDiscriminator::CounterV1Account => 1,
            AccountDiscriminator::DeactivatedAccount => 255,
        }
    }
}

impl TryFrom<u8> for AccountDiscriminator {
    type Error = AccountDiscriminatorError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            1 => Ok(AccountDiscriminator::CounterV1Account),
            255 => Ok(AccountDiscriminator::DeactivatedAccount),
            _ => Err(AccountDiscriminatorError::Invalid(byte)),
        }
    }
}

impl AccountDiscriminator {
    /// Checks that account data has the expected discriminator.
    ///
    /// # Errors
    ///
    /// Returns [`AccountDiscriminatorError`] if the discriminator is missing,
    /// doesn't match the expected value, or is invalid.
    pub fn check(
        expected: AccountDiscriminator,
        data: &[u8],
    ) -> Result<(), AccountDiscriminatorError> {
        let Some(discriminator_byte) = data.first() else {
            return Err(AccountDiscriminatorError::Missing);
        };

        let observed = AccountDiscriminator::try_from(*discriminator_byte)?;
        if observed != expected {
            return Err(AccountDiscriminatorError::Mismatch { expected, observed });
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct CounterV1 {
    pub discriminator: AccountDiscriminator,
    pub owner: Pubkey,
    pub bump: u8,
    pub count: u64,
    pub reserved: [u8; 31], // room for future fields
}

impl CounterV1 {
    /// Returns the size in bytes required to store a [`CounterV1`] account.
    #[must_use]
    pub const fn size() -> usize {
        if let wincode::TypeMeta::Static { size, .. } = <Self as wincode::SchemaWrite>::TYPE_META {
            size
        } else {
            // CounterV1 has a fixed layout, so TYPE_META is always Static.
            unreachable!()
        }
    }

    /// Serializes the counter state to bytes.
    ///
    /// # Errors
    ///
    /// Returns [`wincode::WriteResult`] if serialization fails.
    pub fn serialize(&self) -> wincode::WriteResult<Vec<u8>> {
        wincode::serialize(self)
    }

    /// Deserializes the counter state from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`wincode::ReadError`] if deserialization fails.
    pub fn deserialize(src: &[u8]) -> Result<Self, wincode::ReadError> {
        wincode::deserialize(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_serialization_roundtrip() -> wincode::Result<()> {
        // Verify that Counter can be serialized and deserialized without data loss
        let original = CounterV1 {
            discriminator: AccountDiscriminator::CounterV1Account,
            owner: [2; 32],
            bump: 1,
            count: 100,
            reserved: [0; 31],
        };

        let serialized = wincode::serialize(&original)?;
        assert_eq!(serialized.len(), CounterV1::size());

        let deserialized: CounterV1 = wincode::deserialize(&serialized)?;
        assert_eq!(original.discriminator, deserialized.discriminator);
        assert_eq!(original.owner, deserialized.owner);
        assert_eq!(original.bump, deserialized.bump);
        assert_eq!(original.count, deserialized.count);
        assert_eq!(original.reserved, deserialized.reserved);

        Ok(())
    }

    #[test]
    fn test_size_is_resolved_at_compile_time() {
        const CONST_SIZE: usize = CounterV1::size();
        assert_eq!(CONST_SIZE, CounterV1::size());
        const { assert!(CONST_SIZE > 0) }
    }

    #[test]
    fn test_account_discriminator_serialization() -> wincode::Result<()> {
        // Verify CounterV1Account serializes to 1
        let counter_disc = AccountDiscriminator::CounterV1Account;
        let serialized = wincode::serialize(&counter_disc)?;
        assert_eq!(
            serialized[0], 1,
            "CounterV1Account should serialize to byte 1, got {}",
            serialized[0]
        );

        // Verify DeactivatedAccount serializes to 255
        let deactivated_disc = AccountDiscriminator::DeactivatedAccount;
        let serialized = wincode::serialize(&deactivated_disc)?;
        assert_eq!(
            serialized[0], 255,
            "DeactivatedAccount should serialize to byte 255, got {}",
            serialized[0]
        );

        Ok(())
    }

    #[test]
    fn test_account_discriminator_uses_one_byte_of_memory() {
        assert_eq!(
            std::mem::size_of::<AccountDiscriminator>(),
            1,
            "AccountDiscriminator should be 1 byte with repr(u8)"
        );
    }
}
