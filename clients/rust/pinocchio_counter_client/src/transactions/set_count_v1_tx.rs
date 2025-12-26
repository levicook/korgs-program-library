use {
    crate::instructions::{SetCountV1Ix, SetCountV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum SetCountV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    SetCountV1IxError(#[from] SetCountV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct SetCountV1SimpleTx(VersionedTransaction);

impl SetCountV1SimpleTx {
    /// Creates a new versioned transaction for setting a counter's count.
    ///
    /// # Arguments
    ///
    /// * `program_id` - The ID of the Pinocchio counter program.
    /// * `owner_kp` - The keypair of the counter's owner.
    /// * `count` - The count value to set.
    /// * `recent_blockhash` - The recent blockhash for the transaction.
    ///
    /// # Errors
    ///
    /// Returns [`SetCountV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        count: u64,
        recent_blockhash: Hash,
    ) -> Result<Self, SetCountV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = SetCountV1Ix::new(program_id, owner_pk, count).to_instruction(true)?;

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

impl From<SetCountV1SimpleTx> for VersionedTransaction {
    fn from(value: SetCountV1SimpleTx) -> Self {
        value.0
    }
}
