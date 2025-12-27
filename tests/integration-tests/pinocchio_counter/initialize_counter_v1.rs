use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_counter::{
            malicious_builders::{MaliciousInitializeCounterV1Ix, MaliciousInitializeCounterV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_counter_client::{
        find_counter_address,
        transactions::{DeactivateCounterV1SimpleTx, InitializeCounterV1SimpleTx},
    },
    pinocchio_counter_program::{AccountDiscriminator, CounterV1},
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let init_counter_tx =
        InitializeCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, bump) = find_counter_address(&ctx.program_id(), &owner_pk);

    let counter_account = ctx
        .get_account(counter_pk)
        .ok_or("Counter account not found")?;

    assert_eq!(counter_account.data.len(), CounterV1::size());

    assert_eq!(
        counter_account.data[0],
        u8::from(AccountDiscriminator::CounterV1Account)
    );

    assert_ne!(counter_account.lamports, 0, "Counter should have lamports");

    assert_eq!(
        counter_account.owner,
        ctx.program_id(),
        "Owner mismatch expected {expected:?}, observed {observed:?}",
        expected = ctx.program_id(),
        observed = counter_account.owner
    );

    let counter = CounterV1::deserialize(&counter_account.data)?;

    assert_eq!(
        counter.discriminator,
        AccountDiscriminator::CounterV1Account,
        "Discriminator mismatch expected {expected:?}, observed {observed:?}",
        expected = AccountDiscriminator::CounterV1Account,
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

    Ok(())
}

// ============================================================================
// Malicious Transaction Tests - Account Validation Failures
// ============================================================================

#[test]
fn fails_when_payer_not_signer() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::initialize_counter_v1::MaliciousInitializeCounterV1Ix::with_payer_not_signer)
    .with_different_signer(fee_payer_kp) // Sign with different keypair
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x102", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_writable() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::initialize_counter_v1::MaliciousInitializeCounterV1Ix::with_counter_not_writable)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x103", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::initialize_counter_v1::MaliciousInitializeCounterV1Ix::with_random_counter_address)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x104", &tx_result);

    Ok(())
}

#[test]
fn fails_when_system_program_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::initialize_counter_v1::MaliciousInitializeCounterV1Ix::with_random_system_program)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x108", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_has_pre_existing_data() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let init_counter_tx1 = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx1);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
    let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
    assert!(!counter_account.data.is_empty(), "Counter should have data");

    ctx.advance_slot(1)?;

    let init_counter_tx2 =
        InitializeCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result2 = ctx.send_transaction(init_counter_tx2);
    demand_tx_failure(&tx_result2);
    demand_logs_contain("failed: custom program error: 0x105", &tx_result2);

    Ok(())
}

#[test]
fn fails_when_reinitializing_deactivated_counter() -> TestResult {
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

    // Deactivate the counter
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
    let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
    // Deactivated counter should have 1 byte (discriminator)
    assert_eq!(
        counter_account.data.len(),
        1,
        "Deactivated counter should be 1 byte"
    );

    ctx.advance_slot(1)?;

    // Attempt to re-initialize the deactivated counter - should fail
    let reinit_counter_tx =
        InitializeCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(reinit_counter_tx);
    demand_tx_failure(&tx_result);
    // Should fail because counter.data_is_empty() is false (has 1 byte discriminator)
    demand_logs_contain("failed: custom program error: 0x105", &tx_result);

    Ok(())
}

#[test]
fn fails_with_invalid_instruction_discriminator() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(|ix| ix.with_invalid_discriminator(255))
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x2", &tx_result);

    Ok(())
}

#[test]
fn fails_with_empty_instruction_data() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::initialize_counter_v1::MaliciousInitializeCounterV1Ix::with_empty_data)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x1", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);

    let malicious_ix = MaliciousInitializeCounterV1Ix::from_valid(ctx.program_id(), owner_pk);
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

    let malicious_tx = MaliciousInitializeCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_instruction(instruction)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x101", &tx_result);

    Ok(())
}
