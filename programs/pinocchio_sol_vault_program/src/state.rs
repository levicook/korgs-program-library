use {
    crate::AccountDiscriminator,
    pinocchio::{program_error::ProgramError, pubkey::Pubkey},
};

pub const VAULT_V1_SIZE: usize = 1 + 32 + 1; // discriminator + owner + bump = 34 bytes

#[repr(C)]
pub struct VaultV1 {
    pub discriminator: AccountDiscriminator,
    pub owner: [u8; 32], // Pubkey as bytes to avoid alignment padding
    pub bump: u8,
}

impl VaultV1 {
    /// Returns the size in bytes required to store a [`VaultV1`] account.
    #[must_use]
    pub const fn size() -> usize {
        VAULT_V1_SIZE
    }

    /// Deserializes vault state from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError`] if deserialization fails.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        use pinocchio::program_error::ProgramError;

        if data.len() != Self::size() {
            return Err(ProgramError::InvalidAccountData);
        }

        let discriminator_byte = data[0];
        let discriminator = AccountDiscriminator::try_from(discriminator_byte)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let mut owner = [0u8; 32];
        owner.copy_from_slice(&data[1..33]);

        let bump = data[33];

        Ok(Self {
            discriminator,
            owner,
            bump,
        })
    }

    /// Serializes vault state to bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError`] if serialization fails.
    pub fn to_bytes(&self) -> [u8; Self::size()] {
        let mut bytes = [0u8; Self::size()];
        bytes[0] = u8::from(self.discriminator);
        bytes[1..33].copy_from_slice(&self.owner);
        bytes[33] = self.bump;
        bytes
    }

    /// Returns the owner as a [`Pubkey`].
    #[must_use]
    pub fn owner(&self) -> Pubkey {
        Pubkey::try_from(self.owner.as_ref()).expect("Owner bytes should always be valid Pubkey")
    }

    /// Sets the owner from a [`Pubkey`].
    pub fn set_owner(&mut self, owner: Pubkey) {
        self.owner.copy_from_slice(owner.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_serialization_roundtrip() {
        let original = VaultV1 {
            discriminator: AccountDiscriminator::VaultV1Account,
            owner: [2; 32],
            bump: 1,
        };

        let serialized = original.to_bytes();
        assert_eq!(serialized.len(), VaultV1::size());

        let deserialized = VaultV1::from_bytes(&serialized).unwrap();
        assert_eq!(original.discriminator, deserialized.discriminator);
        assert_eq!(original.owner, deserialized.owner);
        assert_eq!(original.bump, deserialized.bump);
    }

    #[test]
    fn test_vault_size_is_const() {
        const CONST_SIZE: usize = VaultV1::size();
        assert_eq!(CONST_SIZE, VaultV1::size());
        const { assert!(CONST_SIZE > 0) }
    }
}
