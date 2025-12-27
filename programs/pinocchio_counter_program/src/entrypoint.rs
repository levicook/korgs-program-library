use {
    crate::{
        CounterError, DeactivateCounterV1, DecrementCountV1, IncrementCountV1, InitializeCounterV1,
        InstructionDiscriminator, SetCountV1,
    },
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
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
            InitializeCounterV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::DeactivateCounterV1 => {
            DeactivateCounterV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::IncrementCountV1 => {
            IncrementCountV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::DecrementCountV1 => {
            DecrementCountV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::SetCountV1 => {
            SetCountV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }
    }

    Ok(())
}

fn handle_instruction_error<E: std::fmt::Debug + Into<CounterError>>(err: E) -> ProgramError {
    let counter_error: CounterError = err.into();

    let error_msg = format!("Error: {}", counter_error);
    pinocchio::msg!(&error_msg);

    counter_error.into()
}
