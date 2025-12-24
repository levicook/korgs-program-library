use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey, ProgramResult};

#[cfg(not(feature = "no-entrypoint"))]
pinocchio::entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello from my program!");
    Ok(())
}
