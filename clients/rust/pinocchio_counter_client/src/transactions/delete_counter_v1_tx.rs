use {
    crate::instructions::{DeactivateCounterV1Ix, DeactivateCounterV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum DeleteCounterV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    DeleteCounterIxError(#[from] DeactivateCounterV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct DeleteCounterV1SimpleTx(VersionedTransaction);

impl DeleteCounterV1SimpleTx {
    /// Creates a new versioned transaction for deleting a counter.
    ///
    /// # Errors
    ///
    /// Returns [`DeleteCounterV1SimpleTxError`] if:
    /// - Instruction validation fails (see [`DeleteCounterV1IxError`])
    /// - Message compilation fails
    /// - Transaction signing fails
    /// - Transaction sanitization fails
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, DeleteCounterV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = DeactivateCounterV1Ix::new(program_id, owner_pk).to_instruction(true)?;

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

impl From<DeleteCounterV1SimpleTx> for VersionedTransaction {
    fn from(value: DeleteCounterV1SimpleTx) -> Self {
        value.0
    }
}
