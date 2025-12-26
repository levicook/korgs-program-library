use pinocchio::pubkey::Pubkey;
use wincode::{SchemaRead, SchemaWrite};

#[repr(u8)]
#[derive(Debug, PartialEq, SchemaRead, SchemaWrite)]
pub enum AccountDiscriminator {
    CounterV1 = 1,
}

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct CounterV1 {
    pub discriminator: AccountDiscriminator,
    pub owner: Pubkey,
    pub bump: u8,
    pub count: u64,
    pub reserved: [u8; 31],
}

impl CounterV1 {
    /// Returns the size in bytes required to store a [`CounterV1`] account.
    ///
    /// # Panics
    ///
    /// Panics if the type metadata indicates a dynamic size, which should never
    /// happen for [`CounterV1`] as it has a fixed layout.
    #[must_use]
    pub const fn size() -> usize {
        match <Self as wincode::SchemaWrite>::TYPE_META {
            wincode::TypeMeta::Static { size, .. } => size,
            wincode::TypeMeta::Dynamic => panic!("CounterV1 should have static size"),
        }
    }

    /// Serializes the counter state to bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`wincode::WriteResult`] error if serialization fails.
    pub fn serialize(&self) -> wincode::WriteResult<Vec<u8>> {
        wincode::serialize(self)
    }

    /// Deserializes the counter state from bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`wincode::ReadError`] error if deserialization fails.
    pub fn deserialize(src: &[u8]) -> Result<Self, wincode::ReadError> {
        wincode::deserialize(src)
    }
}

impl Default for CounterV1 {
    fn default() -> Self {
        Self {
            discriminator: AccountDiscriminator::CounterV1,
            owner: Pubkey::default(),
            bump: 0,
            count: 0,
            reserved: [0; 31],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_serialization_roundtrip() -> wincode::Result<()> {
        // Verify that Counter can be serialized and deserialized without data loss
        let original = CounterV1 {
            discriminator: AccountDiscriminator::CounterV1,
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
}
