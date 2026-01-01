use {
    crate::instructions::{DepositV1Ix, DepositV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum DepositV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    DepositV1IxError(#[from] DepositV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct DepositV1SimpleTx(VersionedTransaction);

impl DepositV1SimpleTx {
    /// Creates a new versioned transaction for depositing SOL into a vault.
    ///
    /// # Errors
    ///
    /// Returns [`DepositV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        amount: u64,
        recent_blockhash: Hash,
    ) -> Result<Self, DepositV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = DepositV1Ix::new(program_id, owner_pk, amount).build()?;

        let message = VersionedMessage::V0(v0::Message::try_compile(
            &owner_pk,
            &[ix],
            &[],
            recent_blockhash,
        )?);

        let tx = VersionedTransaction::try_new(message, &[owner_kp])?;
        tx.sanitize()?;

        Ok(Self(tx))
    }
}

impl From<DepositV1SimpleTx> for VersionedTransaction {
    fn from(value: DepositV1SimpleTx) -> Self {
        value.0
    }
}
