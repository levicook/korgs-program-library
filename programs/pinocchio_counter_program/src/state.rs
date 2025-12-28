use {
    crate::AccountDiscriminator,
    pinocchio::pubkey::Pubkey,
    wincode::{SchemaRead, SchemaWrite},
};

pub const DEACTIVATED_ACCOUNT_SIZE: usize = 1;

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct CounterV1 {
    pub discriminator: AccountDiscriminator,
    pub owner: Pubkey,
    pub bump: u8,
    pub count: u64,
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
        };

        let serialized = wincode::serialize(&original)?;
        assert_eq!(serialized.len(), CounterV1::size());

        let deserialized: CounterV1 = wincode::deserialize(&serialized)?;
        assert_eq!(original.discriminator, deserialized.discriminator);
        assert_eq!(original.owner, deserialized.owner);
        assert_eq!(original.bump, deserialized.bump);
        assert_eq!(original.count, deserialized.count);

        Ok(())
    }

    #[test]
    fn test_counter_size_is_resolved_at_compile_time() {
        const CONST_SIZE: usize = CounterV1::size();
        assert_eq!(CONST_SIZE, CounterV1::size());
        const { assert!(CONST_SIZE > 0) }
    }
}
