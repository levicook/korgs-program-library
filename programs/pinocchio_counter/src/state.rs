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
    pub const fn size() -> usize {
        match <Self as wincode::SchemaWrite>::TYPE_META {
            wincode::TypeMeta::Static { size, .. } => size,
            wincode::TypeMeta::Dynamic => panic!("CounterV1 should have static size"),
        }
    }

    pub fn serialize(&self) -> wincode::WriteResult<Vec<u8>> {
        wincode::serialize(self)
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
