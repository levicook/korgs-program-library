use solana_hash::Hash;
use solana_keypair::{Keypair, Signer};
use solana_message::{v0, CompileError};
use solana_pubkey::Pubkey;
use solana_sanitize::SanitizeError;
use solana_transaction::{versioned::VersionedTransaction, SignerError, VersionedMessage};

use crate::instructions::{CreateCounterV1Ix, CreateCounterV1IxError};

#[derive(Debug, thiserror::Error)]
pub enum CreateCounterV1SimpleTxError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    CreateCounterIxError(#[from] CreateCounterV1IxError),

    #[error(transparent)]
    SanitizeError(#[from] SanitizeError),

    #[error(transparent)]
    SignerError(#[from] SignerError),
}

pub struct CreateCounterV1SimpleTx(VersionedTransaction);

impl CreateCounterV1SimpleTx {
    /// Creates a new versioned transaction for creating a counter.
    ///
    /// # Errors
    ///
    /// Returns [`CreateCounterV1SimpleTxError`] if:
    /// - Instruction validation fails (see [`CreateCounterV1IxError`])
    /// - Message compilation fails
    /// - Transaction signing fails
    /// - Transaction sanitization fails
    pub fn try_new(
        program_id: Pubkey,
        payer_kp: Keypair,
        recent_blockhash: Hash,
    ) -> Result<Self, CreateCounterV1SimpleTxError> {
        let payer_pk = payer_kp.pubkey();

        let ix = CreateCounterV1Ix::new(program_id, payer_pk).to_instruction(true)?;

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

impl From<CreateCounterV1SimpleTx> for VersionedTransaction {
    fn from(value: CreateCounterV1SimpleTx) -> Self {
        value.0
    }
}
