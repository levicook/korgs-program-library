use std::path::PathBuf;

use litesvm::{error::LiteSVMError, types::TransactionResult, LiteSVM};
use pinocchio_counter_client::{find_counter_address, transactions::CreateCounterV1SimpleTx};
use pinocchio_counter_program::{AccountDiscriminator, CounterV1};
use solana_account::Account;
use solana_clock::Clock;
use solana_hash::Hash;
use solana_instruction::AccountMeta;
use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

use crate::clock_utils::advance_clock;
use crate::litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success};
use crate::malicious_builders::{MaliciousCreateCounterV1Ix, MaliciousCreateCounterV1Tx};

struct TestContext {
    svm: LiteSVM,
    program_id: Pubkey,
}

impl TestContext {
    fn try_new() -> Result<Self, LiteSVMError> {
        let mut litesvm = LiteSVM::new();
        let program_id = Pubkey::new_unique();

        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/sbpf-solana-solana/release/pinocchio_counter_program.so")
            .canonicalize()?;

        litesvm.add_program_from_file(program_id, program_path)?;

        Ok(Self {
            svm: litesvm,
            program_id,
        })
    }

    fn airdrop_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> TransactionResult {
        self.svm.airdrop(&pubkey, lamports)
    }

    fn create_funded_keypair(&mut self) -> Keypair {
        let kp = Keypair::new();
        self.airdrop_lamports(kp.pubkey(), 1_000_000_000)
            .expect("Failed to airdrop lamports");
        kp
    }

    pub fn advance_slot(&mut self, n_slots: u64) {
        let current_clock = self.svm.get_sysvar::<Clock>();
        let new_clock = advance_clock(current_clock, n_slots);
        self.svm.set_sysvar(&new_clock);
        self.svm.expire_blockhash();
    }

    fn get_account(&self, pubkey: Pubkey) -> Option<Account> {
        self.svm.get_account(&pubkey)
    }

    fn latest_blockhash(&self) -> Hash {
        self.svm.latest_blockhash()
    }

    fn send_transaction(&mut self, tx: impl Into<VersionedTransaction>) -> TransactionResult {
        self.svm.send_transaction(tx)
    }
}

#[test]
fn test_create_counter_v1_success() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

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

// ============================================================================
// Malicious Transaction Tests - Account Validation Failures
// ============================================================================

#[test]
fn test_create_counter_fails_when_payer_not_signer() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_payer_not_signer())
            .with_different_signer(fee_payer_kp) // Sign with different keypair
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x8", &tx_result);
}

#[test]
fn test_create_counter_fails_when_counter_not_writable() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_counter_not_writable())
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x4", &tx_result);
}

#[test]
fn test_create_counter_fails_when_counter_address_mismatch() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_random_counter_address())
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x1", &tx_result);
}

#[test]
fn test_create_counter_fails_when_system_program_address_mismatch() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_random_system_program())
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0xb", &tx_result);
}

#[test]
fn test_create_counter_fails_when_counter_has_pre_existing_data() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let create_counter_tx = CreateCounterV1SimpleTx::try_new(
        ctx.program_id,
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )
    .expect("Failed to create counter transaction");

    let tx_result = ctx.send_transaction(create_counter_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id, &owner_pk);
    let counter_account = ctx.get_account(counter_pk).expect("Counter should exist");
    assert!(!counter_account.data.is_empty(), "Counter should have data");

    ctx.advance_slot(1);

    let recreate_tx =
        CreateCounterV1SimpleTx::try_new(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .expect("Failed to create counter transaction");

    let tx_result2 = ctx.send_transaction(recreate_tx);
    demand_tx_failure(&tx_result2);
    demand_logs_contain("failed: custom program error: 0x2", &tx_result2);
}

#[test]
fn test_create_counter_fails_with_invalid_instruction_discriminator() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_invalid_discriminator(255))
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x6", &tx_result);
}

#[test]
fn test_create_counter_fails_with_empty_instruction_data() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(|ix| ix.with_empty_data())
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x6", &tx_result);
}

#[test]
fn test_create_counter_fails_when_not_enough_accounts() {
    let mut ctx = TestContext::try_new().expect("Failed to create test context");
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let (counter_pk, _) = find_counter_address(&ctx.program_id, &owner_pk);

    let malicious_ix = MaliciousCreateCounterV1Ix::from_valid(ctx.program_id, owner_pk);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: counter_pk,
            is_signer: false,
            is_writable: true,
        },
        // Missing system_program - only 2 accounts instead of 3
    ]);

    let malicious_tx =
        MaliciousCreateCounterV1Tx::from_valid(ctx.program_id, owner_kp, ctx.latest_blockhash())
            .with_instruction(instruction)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x7", &tx_result);
}

// NOTE: The following error cases are difficult to test via integration tests because they require
// creating accounts at the counter PDA address with specific states (lamports but empty data, or
// wrong owner). Since PDAs cannot sign transactions, we cannot use create_account to set up these
// states. These cases would be better tested at the program unit test level where we can directly
// construct AccountInfo with the desired state:
// - CounterMustHaveZeroLamports (0x5): Requires counter PDA with lamports > 0 but empty data
// - CounterMustBeOwnedBySystemProgram (0x3): Requires counter PDA owned by wrong program
