use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::{CreateCounterV1, InstructionDiscriminator};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, args) = InstructionDiscriminator::parse(instruction_data)?;

    match discriminator {
        InstructionDiscriminator::CreateCounterV1 => {
            CreateCounterV1::try_from((program_id, accounts, args))?.execute()
        }
        InstructionDiscriminator::DeleteCounterV1 => todo!(),
        InstructionDiscriminator::DecrementCountV1 => todo!(),
        InstructionDiscriminator::IncrementCountV1 => todo!(),
        InstructionDiscriminator::SetCountV1 => todo!(),
    }
}
