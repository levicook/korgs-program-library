use {
    crate::instructions::{InitializeCounterV1Ix, InitializeCounterV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum InitializeCounterV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    InitializeCounterV1IxError(#[from] InitializeCounterV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct InitializeCounterV1SimpleTx(VersionedTransaction);

impl InitializeCounterV1SimpleTx {
    /// Creates a new versioned transaction for initializing a counter.
    ///
    /// # Errors
    ///
    /// Returns [`InitializeCounterV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        payer_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, InitializeCounterV1SimpleTxError> {
        let payer_pk = payer_kp.pubkey();

        let ix = InitializeCounterV1Ix::new(program_id, payer_pk).to_instruction(true)?;

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

impl From<InitializeCounterV1SimpleTx> for VersionedTransaction {
    fn from(value: InitializeCounterV1SimpleTx) -> Self {
        value.0
    }
}
