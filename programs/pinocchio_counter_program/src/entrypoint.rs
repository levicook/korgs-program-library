use {
    crate::{DeactivateCounterV1, InitializeCounterV1, InstructionDiscriminator},
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

        InstructionDiscriminator::DecrementCountV1 => todo!(),
        InstructionDiscriminator::IncrementCountV1 => todo!(),
        InstructionDiscriminator::SetCountV1 => todo!(),
    }
}
