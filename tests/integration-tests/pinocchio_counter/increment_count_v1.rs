use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_counter::{
            malicious_builders::{MaliciousIncrementCountV1Ix, MaliciousIncrementCountV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_counter_client::{
        find_counter_address,
        transactions::{
            DeactivateCounterV1SimpleTx, IncrementCountV1SimpleTx, InitializeCounterV1SimpleTx,
            SetCountV1SimpleTx,
        },
    },
    pinocchio_counter_program::{AccountDiscriminator, CounterV1},
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
};

// ============================================================================
// Increment Count Tests
// ============================================================================

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize counter first
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
    let counter_account_before = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should exist")?;

    let counter_before = CounterV1::deserialize(&counter_account_before.data)?;
    assert_eq!(counter_before.count, 0);

    ctx.advance_slot(1)?;

    // Increment the counter
    let increment_tx =
        IncrementCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(increment_tx);
    demand_tx_success(&tx_result);

    // Verify count was incremented
    let counter_account_after = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should still exist")?;

    let counter_after = CounterV1::deserialize(&counter_account_after.data)?;
    assert_eq!(counter_after.count, 1);
    assert_eq!(counter_after.owner, counter_before.owner);
    assert_eq!(counter_after.bump, counter_before.bump);

    Ok(())
}

#[test]
fn succeeds_multiple_times() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize counter
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Increment multiple times
    for expected_count in 1..=5 {
        let increment_tx = IncrementCountV1SimpleTx::try_new(
            ctx.program_id(),
            owner_kp.insecure_clone(),
            ctx.latest_blockhash(),
        )?;

        let tx_result = ctx.send_transaction(increment_tx);
        demand_tx_success(&tx_result);

        ctx.advance_slot(1)?;

        let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
        let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
        let counter = CounterV1::deserialize(&counter_account.data)?;
        assert_eq!(counter.count, expected_count);
    }

    Ok(())
}

#[test]
fn fails_when_owner_not_signer() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    // Initialize counter first
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx =
        MaliciousIncrementCountV1Tx::from_valid(ctx.program_id(), owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(MaliciousIncrementCountV1Ix::with_owner_not_signer)
            .with_different_signer(fee_payer_kp)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x302", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_writable() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize counter first
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx =
        MaliciousIncrementCountV1Tx::from_valid(ctx.program_id(), owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(MaliciousIncrementCountV1Ix::with_counter_not_writable)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x304", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize counter first
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx =
        MaliciousIncrementCountV1Tx::from_valid(ctx.program_id(), owner_kp, ctx.latest_blockhash())
            .with_malicious_instruction(MaliciousIncrementCountV1Ix::with_random_counter_address)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x305", &tx_result);

    Ok(())
}

#[test]
fn fails_when_owner_mismatch_address_validation() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();
    let other_owner_kp = ctx.create_funded_keypair();

    // Initialize counter with first owner
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to increment with different owner
    // This will fail at address validation (0x305 = CounterAddressMismatch) because the counter address
    // is derived from the owner, so using a different owner means the address won't match.
    let malicious_tx = MaliciousIncrementCountV1Tx::from_valid(
        ctx.program_id(),
        other_owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(|ix| {
        // Use correct counter address but wrong owner signer
        let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
        ix.with_counter_address(counter_pk)
    })
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x305", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize counter first
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_ix = MaliciousIncrementCountV1Ix::from_valid(ctx.program_id(), owner_pk);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        // Missing counter - only 1 account instead of 2
    ]);

    let malicious_tx =
        MaliciousIncrementCountV1Tx::from_valid(ctx.program_id(), owner_kp, ctx.latest_blockhash())
            .with_instruction(instruction)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x301", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_initialized() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Don't initialize counter - try to increment non-existent counter
    // This will fail at address validation since the counter PDA doesn't exist
    let increment_tx =
        IncrementCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(increment_tx);
    // The transaction will fail because the counter account doesn't exist
    // The exact error depends on how LiteSVM handles missing accounts
    demand_tx_failure(&tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_has_invalid_discriminator() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);

    let counter_account = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should exist")?;

    let mut counter = CounterV1::deserialize(&counter_account.data)?;
    assert_eq!(
        counter.discriminator,
        AccountDiscriminator::CounterV1Account,
        "Counter should have CounterV1Account discriminator before corruption"
    );

    counter.discriminator = AccountDiscriminator::DeactivatedAccount;

    let corrupted_data = counter.serialize()?;
    let mut corrupted_account = counter_account;
    corrupted_account.data = corrupted_data;

    ctx.set_account(counter_pk, corrupted_account)?;
    ctx.advance_slot(1)?;

    let increment_tx =
        IncrementCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(increment_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x30b", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_is_deactivated() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let increment_tx =
        IncrementCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(increment_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x30b", &tx_result);

    Ok(())
}

#[test]
fn saturates_at_max_when_incrementing_from_max() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize counter
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Set counter to u64::MAX
    let set_count_tx = SetCountV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        u64::MAX,
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(set_count_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
    let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
    let counter_before = CounterV1::deserialize(&counter_account.data)?;
    assert_eq!(
        counter_before.count,
        u64::MAX,
        "Counter should be at u64::MAX"
    );

    ctx.advance_slot(1)?;

    // Increment from u64::MAX - should saturate and remain at u64::MAX
    let increment_tx =
        IncrementCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(increment_tx);
    demand_tx_success(&tx_result);

    let counter_account_after = ctx
        .get_account(counter_pk)
        .ok_or("Counter should still exist")?;
    let counter_after = CounterV1::deserialize(&counter_account_after.data)?;
    assert_eq!(
        counter_after.count,
        u64::MAX,
        "Incrementing from u64::MAX should saturate and remain at u64::MAX"
    );

    Ok(())
}
