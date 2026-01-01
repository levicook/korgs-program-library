use {
    crate::instructions::{ReactivateVaultV1Ix, ReactivateVaultV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum ReactivateVaultV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    ReactivateVaultV1IxError(#[from] ReactivateVaultV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct ReactivateVaultV1SimpleTx(VersionedTransaction);

impl ReactivateVaultV1SimpleTx {
    /// Creates a new versioned transaction for reactivating a vault.
    ///
    /// # Errors
    ///
    /// Returns [`ReactivateVaultV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        payer_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, ReactivateVaultV1SimpleTxError> {
        let payer_pk = payer_kp.pubkey();

        let ix = ReactivateVaultV1Ix::new(program_id, payer_pk).build()?;

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

impl From<ReactivateVaultV1SimpleTx> for VersionedTransaction {
    fn from(value: ReactivateVaultV1SimpleTx) -> Self {
        value.0
    }
}
