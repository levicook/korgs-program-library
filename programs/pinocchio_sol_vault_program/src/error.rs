use {
    crate::{
        instructions::{
            DeactivateVaultV1Error, DepositV1Error, InitializeVaultV1Error, ReactivateVaultV1Error,
            WithdrawV1Error,
        },
        InstructionDiscriminatorError,
    },
    pinocchio::program_error::ProgramError,
};

// Code offsets for each error type
const INSTRUCTION_DISCRIMINATOR_ERROR_OFFSET: u32 = 0x000; // 0
const INITIALIZE_VAULT_V1_OFFSET: u32 = 0x100; // 256
const DEPOSIT_V1_OFFSET: u32 = 0x200; // 512
const WITHDRAW_V1_OFFSET: u32 = 0x300; // 768
const DEACTIVATE_VAULT_V1_OFFSET: u32 = 0x400; // 1024
const REACTIVATE_VAULT_V1_OFFSET: u32 = 0x500; // 1536

#[derive(Debug)]
pub enum InstructionError {
    ProgramError(ProgramError),
    InitializeVaultV1(InitializeVaultV1Error),
    DepositV1(DepositV1Error),
    WithdrawV1(WithdrawV1Error),
    DeactivateVaultV1(DeactivateVaultV1Error),
    ReactivateVaultV1(ReactivateVaultV1Error),
}

pub type InstructionResult<T> = Result<T, InstructionError>;

impl From<InstructionError> for ProgramError {
    fn from(e: InstructionError) -> Self {
        match e {
            InstructionError::ProgramError(pe) => pe,
            InstructionError::InitializeVaultV1(e) => ProgramError::Custom(
                INITIALIZE_VAULT_V1_OFFSET
                    + match e {
                        InitializeVaultV1Error::NotEnoughAccounts { .. } => 0x01,
                        InitializeVaultV1Error::PayerMustBeSigner => 0x02,
                        InitializeVaultV1Error::PayerMustBeWriteable => 0x03,
                        InitializeVaultV1Error::VaultMustBeWriteable => 0x04,
                        InitializeVaultV1Error::VaultAddressMismatch { .. } => 0x05,
                        InitializeVaultV1Error::VaultMustBeEmpty => 0x06,
                        InitializeVaultV1Error::VaultMustHaveZeroLamports => 0x07,
                        InitializeVaultV1Error::VaultMustBeOwnedBySystemProgram => 0x08,
                        InitializeVaultV1Error::SystemProgramAddressMismatch => 0x09,
                        InitializeVaultV1Error::SerializedSizeMismatch { .. } => 0x0a,
                        InitializeVaultV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::DepositV1(e) => ProgramError::Custom(
                DEPOSIT_V1_OFFSET
                    + match e {
                        DepositV1Error::NotEnoughAccounts { .. } => 0x01,
                        DepositV1Error::OwnerMustBeSigner => 0x02,
                        DepositV1Error::OwnerMustBeWriteable => 0x03,
                        DepositV1Error::VaultMustBeWriteable => 0x04,
                        DepositV1Error::VaultAddressMismatch { .. } => 0x05,
                        DepositV1Error::VaultMustBeOwnedByProgram => 0x06,
                        DepositV1Error::AccountDiscriminatorError(_) => 0x07,
                        DepositV1Error::InvalidInstructionData => 0x08,
                        DepositV1Error::OwnerMismatch { .. } => 0x09,
                        DepositV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::WithdrawV1(e) => ProgramError::Custom(
                WITHDRAW_V1_OFFSET
                    + match e {
                        WithdrawV1Error::NotEnoughAccounts { .. } => 0x01,
                        WithdrawV1Error::OwnerMustBeSigner => 0x02,
                        WithdrawV1Error::OwnerMustBeWriteable => 0x03,
                        WithdrawV1Error::VaultMustBeWriteable => 0x04,
                        WithdrawV1Error::VaultAddressMismatch { .. } => 0x05,
                        WithdrawV1Error::VaultMustBeOwnedByProgram => 0x06,
                        WithdrawV1Error::AccountDiscriminatorError(_) => 0x07,
                        WithdrawV1Error::InvalidInstructionData => 0x08,
                        WithdrawV1Error::OwnerMismatch { .. } => 0x09,
                        WithdrawV1Error::InsufficientFunds { .. } => 0x0a,
                        WithdrawV1Error::WouldViolateRentMinimum { .. } => 0x0b,
                        WithdrawV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::DeactivateVaultV1(e) => ProgramError::Custom(
                DEACTIVATE_VAULT_V1_OFFSET
                    + match e {
                        DeactivateVaultV1Error::NotEnoughAccounts { .. } => 0x01,
                        DeactivateVaultV1Error::OwnerMustBeSigner => 0x02,
                        DeactivateVaultV1Error::OwnerMustBeWriteable => 0x03,
                        DeactivateVaultV1Error::VaultMustBeWriteable => 0x04,
                        DeactivateVaultV1Error::VaultAddressMismatch { .. } => 0x05,
                        DeactivateVaultV1Error::AccountDiscriminatorError(_) => 0x06,
                        DeactivateVaultV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
            InstructionError::ReactivateVaultV1(e) => ProgramError::Custom(
                REACTIVATE_VAULT_V1_OFFSET
                    + match e {
                        ReactivateVaultV1Error::NotEnoughAccounts { .. } => 0x01,
                        ReactivateVaultV1Error::PayerMustBeSigner => 0x02,
                        ReactivateVaultV1Error::PayerMustBeWriteable => 0x03,
                        ReactivateVaultV1Error::VaultMustBeWriteable => 0x04,
                        ReactivateVaultV1Error::VaultAddressMismatch { .. } => 0x05,
                        ReactivateVaultV1Error::SystemProgramAddressMismatch => 0x06,
                        ReactivateVaultV1Error::AccountDiscriminatorError(_) => 0x07,
                        ReactivateVaultV1Error::SerializedSizeMismatch { .. } => 0x08,
                        ReactivateVaultV1Error::ProgramError(_) => {
                            unreachable!(
                                "ProgramError variant should be extracted before this point"
                            )
                        }
                    },
            ),
        }
    }
}

impl From<InstructionDiscriminatorError> for ProgramError {
    fn from(e: InstructionDiscriminatorError) -> Self {
        ProgramError::Custom(
            INSTRUCTION_DISCRIMINATOR_ERROR_OFFSET
                + match e {
                    InstructionDiscriminatorError::Missing => 0x01,
                    InstructionDiscriminatorError::Invalid(_) => 0x02,
                },
        )
    }
}

impl From<DeactivateVaultV1Error> for InstructionError {
    fn from(err: DeactivateVaultV1Error) -> Self {
        match err {
            DeactivateVaultV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::DeactivateVaultV1(err),
        }
    }
}

impl From<DepositV1Error> for InstructionError {
    fn from(err: DepositV1Error) -> Self {
        match err {
            DepositV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::DepositV1(err),
        }
    }
}

impl From<InitializeVaultV1Error> for InstructionError {
    fn from(err: InitializeVaultV1Error) -> Self {
        match err {
            InitializeVaultV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::InitializeVaultV1(err),
        }
    }
}

impl From<ReactivateVaultV1Error> for InstructionError {
    fn from(err: ReactivateVaultV1Error) -> Self {
        match err {
            ReactivateVaultV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::ReactivateVaultV1(err),
        }
    }
}

impl From<WithdrawV1Error> for InstructionError {
    fn from(err: WithdrawV1Error) -> Self {
        match err {
            WithdrawV1Error::ProgramError(pe) => InstructionError::ProgramError(pe),
            _ => InstructionError::WithdrawV1(err),
        }
    }
}
