use {
    crate::instructions::{WithdrawV1Ix, WithdrawV1IxError},
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_message::{v0, CompileError},
    solana_pubkey::Pubkey,
    solana_sanitize::SanitizeError,
    solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum WithdrawV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    WithdrawV1IxError(#[from] WithdrawV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct WithdrawV1SimpleTx(VersionedTransaction);

impl WithdrawV1SimpleTx {
    /// Creates a new versioned transaction for withdrawing SOL from a vault.
    ///
    /// # Errors
    ///
    /// Returns [`WithdrawV1SimpleTxError`] if instruction validation, message compilation,
    /// transaction signing, or transaction sanitization fails.
    pub fn try_new(
        program_id: Pubkey,
        owner_kp: Keypair,
        amount: u64,
        recent_blockhash: Hash,
    ) -> Result<Self, WithdrawV1SimpleTxError> {
        let owner_pk = owner_kp.pubkey();

        let ix = WithdrawV1Ix::new(program_id, owner_pk, amount).build()?;

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

impl From<WithdrawV1SimpleTx> for VersionedTransaction {
    fn from(value: WithdrawV1SimpleTx) -> Self {
        value.0
    }
}
