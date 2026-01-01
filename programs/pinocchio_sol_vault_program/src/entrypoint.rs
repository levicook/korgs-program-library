use {
    crate::{
        DeactivateVaultV1, DepositV1, InitializeVaultV1, InstructionDiscriminator,
        InstructionDiscriminatorError, InstructionError, ReactivateVaultV1, WithdrawV1,
    },
    pinocchio::{
        account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, args) = InstructionDiscriminator::parse(instruction_data)
        .map_err(handle_instruction_discriminator_error)?;

    match discriminator {
        InstructionDiscriminator::InitializeVaultV1 => {
            InitializeVaultV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::DepositV1 => {
            DepositV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::WithdrawV1 => {
            WithdrawV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::DeactivateVaultV1 => {
            DeactivateVaultV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }

        InstructionDiscriminator::ReactivateVaultV1 => {
            ReactivateVaultV1::try_from((program_id, accounts, args))
                .and_then(|ix| ix.execute())
                .map_err(handle_instruction_error)?;
        }
    }

    Ok(())
}

fn handle_instruction_discriminator_error(err: InstructionDiscriminatorError) -> ProgramError {
    let error_msg = format!("Instruction Discriminator Error: {:?}", err);
    msg!(&error_msg);
    err.into()
}

fn handle_instruction_error<E: std::fmt::Debug + Into<InstructionError>>(err: E) -> ProgramError {
    let instruction_error: InstructionError = err.into();
    let error_msg = format!("Instruction Error: {:?}", instruction_error);
    msg!(&error_msg);
    instruction_error.into()
}
