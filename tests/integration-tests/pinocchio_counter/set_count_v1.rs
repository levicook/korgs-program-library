use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_counter::{
            malicious_builders::{MaliciousSetCountV1Ix, MaliciousSetCountV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_counter_client::{
        find_counter_address,
        transactions::{
            DeactivateCounterV1SimpleTx, InitializeCounterV1SimpleTx, SetCountV1SimpleTx,
        },
    },
    pinocchio_counter_program::{AccountDiscriminator, CounterV1},
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
};

// ============================================================================
// Set Count Tests
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

    // Set count to 42
    let set_count_tx =
        SetCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, 42, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(set_count_tx);
    demand_tx_success(&tx_result);

    // Verify count was set
    let counter_account_after = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should still exist")?;

    let counter_after = CounterV1::deserialize(&counter_account_after.data)?;
    assert_eq!(counter_after.count, 42);
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

    // Set count to different values
    for count in [10, 100, 1000, 0, u64::MAX] {
        let set_count_tx = SetCountV1SimpleTx::try_new(
            ctx.program_id(),
            owner_kp.insecure_clone(),
            count,
            ctx.latest_blockhash(),
        )?;

        let tx_result = ctx.send_transaction(set_count_tx);
        demand_tx_success(&tx_result);

        ctx.advance_slot(1)?;

        let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
        let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
        let counter = CounterV1::deserialize(&counter_account.data)?;
        assert_eq!(
            counter.count, count,
            "Expected count {} after set, got {}",
            count, counter.count
        );
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

    let malicious_tx = MaliciousSetCountV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        50,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(
        super::malicious_builders::set_count_v1::MaliciousSetCountV1Ix::with_owner_not_signer,
    )
    .with_different_signer(fee_payer_kp)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x502", &tx_result);

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

    let malicious_tx = MaliciousSetCountV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        50,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(
        super::malicious_builders::set_count_v1::MaliciousSetCountV1Ix::with_counter_not_writable,
    )
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x504", &tx_result);

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

    let malicious_tx = MaliciousSetCountV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        50,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(
        super::malicious_builders::set_count_v1::MaliciousSetCountV1Ix::with_random_counter_address,
    )
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x505", &tx_result);

    Ok(())
}

#[test]
fn fails_when_owner_mismatch() -> TestResult {
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

    // Try to set count with different owner
    // This will fail at address validation (0x1) because the counter address
    // is derived from the owner, so using a different owner means the address won't match.
    let malicious_tx = MaliciousSetCountV1Tx::from_valid(
        ctx.program_id(),
        other_owner_kp,
        50,
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
    demand_logs_contain("failed: custom program error: 0x505", &tx_result);

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

    let malicious_ix = MaliciousSetCountV1Ix::from_valid(ctx.program_id(), owner_pk, 50);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        // Missing counter - only 1 account instead of 2
    ]);

    let malicious_tx =
        MaliciousSetCountV1Tx::from_valid(ctx.program_id(), owner_kp, 50, ctx.latest_blockhash())
            .with_instruction(instruction)
            .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x501", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_initialized() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Don't initialize counter - try to set count on non-existent counter
    // This will fail at address validation since the counter PDA doesn't exist
    let set_count_tx =
        SetCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, 100, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(set_count_tx);
    // The transaction will fail because the counter account doesn't exist
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

    let set_count_tx =
        SetCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, 42, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(set_count_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x50b", &tx_result);

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

    let set_count_tx =
        SetCountV1SimpleTx::try_new(ctx.program_id(), owner_kp, 100, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(set_count_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x50b", &tx_result);

    Ok(())
}
