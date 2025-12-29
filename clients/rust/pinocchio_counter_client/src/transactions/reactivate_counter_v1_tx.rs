use {
    crate::instructions::{ReactivateCounterV1Ix, ReactivateCounterV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum ReactivateCounterV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    ReactivateCounterV1IxError(#[from] ReactivateCounterV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct ReactivateCounterV1SimpleTx(VersionedTransaction);

impl ReactivateCounterV1SimpleTx {
    /// Creates a new versioned transaction for reactivating a counter.
    ///
    /// # Errors
    ///
    /// Returns [`ReactivateCounterV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        payer_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, ReactivateCounterV1SimpleTxError> {
        let payer_pk = payer_kp.pubkey();

        let ix = ReactivateCounterV1Ix::new(program_id, payer_pk).to_instruction(true)?;

        let message = VersionedMessage::V0(v0::Message::try_compile(
            &payer_pk,
            &[ix],
            &[],
            recent_blockhash,
        )?);

        let tx = VersionedTransaction::try_new(message, &[payer_kp])?;
        tx.sanitize()?;

        Ok(Self(tx))
    }
}

impl From<ReactivateCounterV1SimpleTx> for VersionedTransaction {
    fn from(value: ReactivateCounterV1SimpleTx) -> Self {
        value.0
    }
}
