use {
    crate::instructions::{DecrementCountV1Ix, DecrementCountV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum DecrementCountV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    DecrementCountV1IxError(#[from] DecrementCountV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct DecrementCountV1SimpleTx(VersionedTransaction);

impl DecrementCountV1SimpleTx {
    /// Creates a new versioned transaction for decrementing a counter.
    ///
    /// # Errors
    ///
    /// Returns [`DecrementCountV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, DecrementCountV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = DecrementCountV1Ix::new(program_id, owner_pk).to_instruction(true)?;

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

impl From<DecrementCountV1SimpleTx> for VersionedTransaction {
    fn from(value: DecrementCountV1SimpleTx) -> Self {
        value.0
    }
}
