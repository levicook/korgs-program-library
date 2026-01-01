#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum InstructionDiscriminator {
    InitializeVaultV1 = 1,
    DepositV1 = 2,
    WithdrawV1 = 3,
    DeactivateVaultV1 = 4,
    ReactivateVaultV1 = 5,
}

#[derive(Debug)]
pub enum InstructionDiscriminatorError {
    Missing,
    Invalid(u8),
}

impl InstructionDiscriminator {
    /// Parses the instruction discriminator from the first byte of instruction data.
    ///
    /// Returns the discriminator and the remaining instruction data.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionDiscriminatorError::Missing`] if the instruction data is empty
    /// or [`InstructionDiscriminatorError::Invalid`] if the discriminator byte is not a valid instruction type.
    pub fn parse(instruction_data: &[u8]) -> Result<(Self, &[u8]), InstructionDiscriminatorError> {
        let (first, rest) = instruction_data
            .split_first()
            .ok_or(InstructionDiscriminatorError::Missing)?;

        Ok((Self::try_from(first)?, rest))
    }
}

impl TryFrom<&u8> for InstructionDiscriminator {
    type Error = InstructionDiscriminatorError;

    fn try_from(byte: &u8) -> Result<Self, Self::Error> {
        match *byte {
            1 => Ok(InstructionDiscriminator::InitializeVaultV1),
            2 => Ok(InstructionDiscriminator::DepositV1),
            3 => Ok(InstructionDiscriminator::WithdrawV1),
            4 => Ok(InstructionDiscriminator::DeactivateVaultV1),
            5 => Ok(InstructionDiscriminator::ReactivateVaultV1),
            _ => Err(InstructionDiscriminatorError::Invalid(*byte)),
        }
    }
}

impl From<InstructionDiscriminator> for u8 {
    fn from(discriminator: InstructionDiscriminator) -> Self {
        match discriminator {
            InstructionDiscriminator::InitializeVaultV1 => 1,
            InstructionDiscriminator::DepositV1 => 2,
            InstructionDiscriminator::WithdrawV1 => 3,
            InstructionDiscriminator::DeactivateVaultV1 => 4,
            InstructionDiscriminator::ReactivateVaultV1 => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_initialize_vault_v1() {
        let instruction_data = [1u8, 0x42, 0x43];

        let (discriminator, args) = InstructionDiscriminator::parse(&instruction_data).unwrap();

        assert_eq!(discriminator, InstructionDiscriminator::InitializeVaultV1);
        assert_eq!(args, &[0x42, 0x43]);
    }

    #[test]
    fn test_parse_all_valid_discriminators() {
        let test_cases = [
            (1u8, InstructionDiscriminator::InitializeVaultV1),
            (2u8, InstructionDiscriminator::DepositV1),
            (3u8, InstructionDiscriminator::WithdrawV1),
            (4u8, InstructionDiscriminator::DeactivateVaultV1),
            (5u8, InstructionDiscriminator::ReactivateVaultV1),
        ];

        for (byte, expected) in test_cases {
            let instruction_data = [byte, 0x42, 0x43];
            let (discriminator, args) = InstructionDiscriminator::parse(&instruction_data).unwrap();

            assert_eq!(discriminator, expected);
            assert_eq!(args, &[0x42, 0x43]);
        }
    }

    #[test]
    fn test_parse_empty_data() {
        let result = InstructionDiscriminator::parse(&[]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InstructionDiscriminatorError::Missing
        ));
    }

    #[test]
    fn test_parse_invalid_discriminator() {
        let invalid_discriminators = [6u8, 255u8];

        for invalid_byte in invalid_discriminators {
            let instruction_data = [invalid_byte, 0x42];
            let result = InstructionDiscriminator::parse(&instruction_data);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                InstructionDiscriminatorError::Invalid(_)
            ));
        }
    }

    #[test]
    fn test_parse_single_byte_valid() {
        // Test that parsing works even when there's no additional data
        let instruction_data = [1u8];
        let (discriminator, args) = InstructionDiscriminator::parse(&instruction_data).unwrap();

        assert_eq!(discriminator, InstructionDiscriminator::InitializeVaultV1);
        assert_eq!(args, &[]);
    }
}
