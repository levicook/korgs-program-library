use std::path::PathBuf;

use litesvm::{error::LiteSVMError, types::TransactionResult, LiteSVM};
use pinocchio_counter_client::{find_counter_address, transactions::CreateCounterV1SimpleTx};
use pinocchio_counter_program::{AccountDiscriminator, CounterV1};
use solana_account::Account;
use solana_hash::Hash;
use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

use crate::litesvm_utils::demand_tx_success;

struct TestContext {
    litesvm: LiteSVM,
    program_id: Pubkey,
}

impl TestContext {
    fn try_new() -> Result<Self, LiteSVMError> {
        let mut litesvm = LiteSVM::new();
        let program_kp = Keypair::new();
        let program_id = program_kp.pubkey();

        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/sbpf-solana-solana/release/pinocchio_counter_program.so")
            .canonicalize()?;

        litesvm.add_program_from_file(program_id, program_path)?;

        Ok(Self {
            litesvm,
            program_id,
        })
    }

    fn airdrop_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> TransactionResult {
        self.litesvm.airdrop(&pubkey, lamports)
    }

    fn get_account(&self, pubkey: Pubkey) -> Option<Account> {
        self.litesvm.get_account(&pubkey)
    }

    fn latest_blockhash(&self) -> Hash {
        self.litesvm.latest_blockhash()
    }

    fn send_transaction(&mut self, tx: impl Into<VersionedTransaction>) -> TransactionResult {
        self.litesvm.send_transaction(tx)
    }
}

#[test]
fn test_create_counter_v1_success() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = Keypair::new();
    let owner_pk = owner_kp.pubkey();

    ctx.airdrop_lamports(owner_pk, 1_000_000_000)
        .expect("Failed to airdrop lamports");

    let create_counter_tx =
        CreateCounterV1SimpleTx::try_new(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .expect("Failed to create counter transaction");

    let tx_result = ctx.send_transaction(create_counter_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, bump) = find_counter_address(&ctx.program_id, &owner_pk);

    let counter_account = ctx
        .get_account(counter_pk)
        .expect("Counter account not found");

    assert_eq!(counter_account.data.len(), CounterV1::size());
    assert_ne!(counter_account.lamports, 0);
    assert_eq!(
        counter_account.owner,
        ctx.program_id,
        "Owner mismatch expected {expected:?}, observed {observed:?}",
        expected = ctx.program_id,
        observed = counter_account.owner
    );

    let counter =
        CounterV1::deserialize(&counter_account.data).expect("Failed to deserialize counter state");

    assert_eq!(
        counter.discriminator,
        AccountDiscriminator::CounterV1,
        "Discriminator mismatch expected {expected:?}, observed {observed:?}",
        expected = AccountDiscriminator::CounterV1,
        observed = counter.discriminator
    );

    assert_eq!(
        counter.owner.as_ref(),
        owner_pk.as_ref(),
        "Owner mismatch expected {expected:?}, observed {observed:?}",
        expected = owner_pk,
        observed = counter.owner,
    );

    assert_eq!(
        counter.bump,
        bump,
        "Bump mismatch expected {expected:?}, observed {observed:?}",
        expected = bump,
        observed = counter.bump
    );

    assert_eq!(counter.count, 0);

    assert_eq!(counter.reserved, [0; 31]);
}
