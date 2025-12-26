use {
    crate::{
        DeactivateCounterV1, DecrementCountV1, IncrementCountV1, InitializeCounterV1,
        InstructionDiscriminator, SetCountV1,
    },
    pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult},
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, args) = InstructionDiscriminator::parse(instruction_data)?;

    match discriminator {
        InstructionDiscriminator::InitializeCounterV1 => {
            InitializeCounterV1::try_from((program_id, accounts, args))?.execute()
        }

        InstructionDiscriminator::DeactivateCounterV1 => {
            DeactivateCounterV1::try_from((program_id, accounts, args))?.execute()
        }

        InstructionDiscriminator::IncrementCountV1 => {
            IncrementCountV1::try_from((program_id, accounts, args))?.execute()
        }

        InstructionDiscriminator::DecrementCountV1 => {
            DecrementCountV1::try_from((program_id, accounts, args))?.execute()
        }

        InstructionDiscriminator::SetCountV1 => {
            SetCountV1::try_from((program_id, accounts, args))?.execute()
        }
    }
}
