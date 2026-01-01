use {
    crate::instructions::{DeactivateVaultV1Ix, DeactivateVaultV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum DeactivateVaultV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    DeactivateVaultV1IxError(#[from] DeactivateVaultV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct DeactivateVaultV1SimpleTx(VersionedTransaction);

impl DeactivateVaultV1SimpleTx {
    /// Creates a new versioned transaction for deactivating a vault.
    ///
    /// # Errors
    ///
    /// Returns [`DeactivateVaultV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, DeactivateVaultV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = DeactivateVaultV1Ix::new(program_id, owner_pk).build()?;

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

impl From<DeactivateVaultV1SimpleTx> for VersionedTransaction {
    fn from(value: DeactivateVaultV1SimpleTx) -> Self {
        value.0
    }
}
