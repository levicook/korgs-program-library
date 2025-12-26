use crate::{CounterError, CounterResult};

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum InstructionDiscriminator {
    InitializeCounterV1 = 1,
    DeactivateCounterV1 = 2,

    DecrementCountV1 = 3,
    IncrementCountV1 = 4,
    SetCountV1 = 5,
}

impl InstructionDiscriminator {
    /// Parses the instruction discriminator from the first byte of instruction data.
    ///
    /// Returns the discriminator and the remaining instruction data.
    ///
    /// # Errors
    ///
    /// Returns [`CounterError::InvalidInstructionDiscriminator`] if:
    /// - The instruction data is empty
    /// - The discriminator byte is not a valid instruction type
    pub fn parse(instruction_data: &[u8]) -> CounterResult<(Self, &[u8])> {
        let (first, rest) = instruction_data
            .split_first()
            .ok_or(CounterError::InvalidInstructionDiscriminator(0))?;
        Ok((Self::try_from(first)?, rest))
    }
}

impl TryFrom<&u8> for InstructionDiscriminator {
    type Error = CounterError;

    fn try_from(byte: &u8) -> Result<Self, Self::Error> {
        match *byte {
            1 => Ok(InstructionDiscriminator::InitializeCounterV1),
            2 => Ok(InstructionDiscriminator::DeactivateCounterV1),
            3 => Ok(InstructionDiscriminator::DecrementCountV1),
            4 => Ok(InstructionDiscriminator::IncrementCountV1),
            5 => Ok(InstructionDiscriminator::SetCountV1),
            _ => Err(CounterError::InvalidInstructionDiscriminator(*byte)),
        }
    }
}

impl From<InstructionDiscriminator> for u8 {
    fn from(discriminator: InstructionDiscriminator) -> Self {
        match discriminator {
            InstructionDiscriminator::InitializeCounterV1 => 1,
            InstructionDiscriminator::DeactivateCounterV1 => 2,
            InstructionDiscriminator::DecrementCountV1 => 3,
            InstructionDiscriminator::IncrementCountV1 => 4,
            InstructionDiscriminator::SetCountV1 => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::CounterError};

    #[test]
    fn test_parse_initialize_counter_v1() {
        let instruction_data = [1u8, 0x42, 0x43];

        let (discriminator, args) = InstructionDiscriminator::parse(&instruction_data).unwrap();

        assert_eq!(discriminator, InstructionDiscriminator::InitializeCounterV1);
        assert_eq!(args, &[0x42, 0x43]);
    }

    #[test]
    fn test_parse_all_valid_discriminators() {
        let test_cases = [
            (1u8, InstructionDiscriminator::InitializeCounterV1),
            (2u8, InstructionDiscriminator::DeactivateCounterV1),
            (3u8, InstructionDiscriminator::DecrementCountV1),
            (4u8, InstructionDiscriminator::IncrementCountV1),
            (5u8, InstructionDiscriminator::SetCountV1),
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
        assert_eq!(
            result.unwrap_err(),
            CounterError::InvalidInstructionDiscriminator(0)
        );
    }

    #[test]
    fn test_parse_invalid_discriminator() {
        let invalid_discriminators = [0u8, 6u8, 255u8];

        for invalid_byte in invalid_discriminators {
            let instruction_data = [invalid_byte, 0x42];
            let result = InstructionDiscriminator::parse(&instruction_data);

            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                CounterError::InvalidInstructionDiscriminator(invalid_byte)
            );
        }
    }

    #[test]
    fn test_parse_single_byte_valid() {
        // Test that parsing works even when there's no additional data
        let instruction_data = [1u8];
        let (discriminator, args) = InstructionDiscriminator::parse(&instruction_data).unwrap();

        assert_eq!(discriminator, InstructionDiscriminator::InitializeCounterV1);
        assert_eq!(args, &[]);
    }
}
