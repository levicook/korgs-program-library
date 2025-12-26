use {
    crate::instructions::{IncrementCountV1Ix, IncrementCountV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum IncrementCountV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    IncrementCountV1IxError(#[from] IncrementCountV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct IncrementCountV1SimpleTx(VersionedTransaction);

impl IncrementCountV1SimpleTx {
    /// Creates a new versioned transaction for incrementing a counter.
    ///
    /// # Errors
    ///
    /// Returns [`IncrementCountV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, IncrementCountV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = IncrementCountV1Ix::new(program_id, owner_pk).to_instruction(true)?;

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

impl From<IncrementCountV1SimpleTx> for VersionedTransaction {
    fn from(value: IncrementCountV1SimpleTx) -> Self {
        value.0
    }
}
