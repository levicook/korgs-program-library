use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_counter::{
            malicious_builders::{MaliciousDeactivateCounterV1Ix, MaliciousDeactivateCounterV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_counter_client::{
        find_counter_address,
        transactions::{DeactivateCounterV1SimpleTx, InitializeCounterV1SimpleTx},
    },
    pinocchio_counter_program::{AccountDiscriminator, CounterV1, DEACTIVATED_ACCOUNT_SIZE},
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
    solana_rent::Rent,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let initialize_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(initialize_tx);
    demand_tx_success(&tx_result);

    let (counter_pk, _) = find_counter_address(&ctx.program_id(), &owner_pk);
    let counter_account_before = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should exist")?;

    let counter_lamports_before = counter_account_before.lamports;
    assert_ne!(counter_lamports_before, 0, "Counter should have lamports");

    let owner_account_before = ctx
        .get_account(owner_pk)
        .ok_or("Owner account should exist")?;
    let owner_lamports_before = owner_account_before.lamports;

    ctx.advance_slot(1)?;

    // Now deactivate the counter
    let deactivate_tx =
        DeactivateCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    // Verify counter account is marked as deactivated (1 byte with DeactivatedAccount discriminator)
    let counter_account_after = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should still exist with DeactivatedAccount discriminator")?;

    assert_eq!(
        counter_account_after.data.len(),
        DEACTIVATED_ACCOUNT_SIZE,
        "Counter account should be resized to {expected:?} bytes, observed {observed:?}",
        expected = DEACTIVATED_ACCOUNT_SIZE,
        observed = counter_account_after.data.len()
    );

    assert_eq!(
        counter_account_after.data[0],
        u8::from(AccountDiscriminator::DeactivatedAccount),
        "Discriminator mismatch expected {expected:?}, observed {observed:?}",
        expected = u8::from(AccountDiscriminator::DeactivatedAccount),
        observed = counter_account_after.data[0]
    );

    // Calculate rent-exempt minimum for 1-byte account
    let rent = Rent::default();
    let rent_exempt_deactivated_account = rent.minimum_balance(DEACTIVATED_ACCOUNT_SIZE);

    // Verify lamports were transferred to owner (all except rent-exempt minimum for deactivated account)
    // Note: Owner also pays transaction fees, so we check that they received at least the expected amount
    let owner_account_after = ctx
        .get_account(owner_pk)
        .ok_or("Owner account should exist")?;
    let owner_lamports_after = owner_account_after.lamports;
    let lamports_received = owner_lamports_after.saturating_sub(owner_lamports_before);
    let expected_lamports_received = counter_lamports_before - rent_exempt_deactivated_account;

    // Owner should receive the counter's lamports minus rent-exempt minimum (minus transaction fees)
    assert!(
        lamports_received >= expected_lamports_received.saturating_sub(10_000), // Allow up to 10k for fees
        "Owner should have received counter's lamports minus rent-exempt minimum. Received: {lamports_received}, Expected: ~{expected_lamports_received}"
    );

    // Verify counter account has rent-exempt minimum for 1-byte account
    assert_eq!(
        counter_account_after.lamports,
        rent_exempt_deactivated_account,
        "Lamports mismatch expected {expected:?}, observed {observed:?}",
        expected = rent_exempt_deactivated_account,
        observed = counter_account_after.lamports
    );

    Ok(())
}

#[test]
fn fails_when_owner_not_signer() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::deactivate_counter_v1::MaliciousDeactivateCounterV1Ix::with_owner_not_signer)
    .with_different_signer(fee_payer_kp)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x202", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_writable() -> TestResult {
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

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::deactivate_counter_v1::MaliciousDeactivateCounterV1Ix::with_counter_not_writable)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x204", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_address_mismatch() -> TestResult {
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

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::deactivate_counter_v1::MaliciousDeactivateCounterV1Ix::with_random_counter_address)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x205", &tx_result);

    Ok(())
}

#[test]
fn fails_when_owner_mismatch_address_validation() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();
    let other_owner_kp = ctx.create_funded_keypair();

    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to deactivate with different owner
    // This will fail at address validation (0x205 = CounterAddressMismatch) because the counter address
    // is derived from the owner, so using a different owner means the address won't match.
    // This is actually a good security property - address validation prevents
    // owner mismatch attacks. The stored owner check would only be reached
    // if address validation passed, which requires using the correct owner.
    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
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
    // Address validation fails first, which is the expected security behavior
    demand_logs_contain("failed: custom program error: 0x205", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
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

    ctx.advance_slot(1)?;

    let malicious_ix = MaliciousDeactivateCounterV1Ix::from_valid(ctx.program_id(), owner_pk);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        // Missing counter - only 1 account instead of 2
    ]);

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_instruction(instruction)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x201", &tx_result);

    Ok(())
}

#[test]
fn fails_with_invalid_instruction_discriminator() -> TestResult {
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

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
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

    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDeactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(super::malicious_builders::deactivate_counter_v1::MaliciousDeactivateCounterV1Ix::with_empty_data)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x1", &tx_result);

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

    // Corrupt the discriminator to simulate an invalid account state that should be rejected
    counter.discriminator = AccountDiscriminator::DeactivatedAccount;

    let corrupted_data = counter.serialize()?;
    let mut corrupted_account = counter_account;
    corrupted_account.data = corrupted_data;

    ctx.set_account(counter_pk, corrupted_account)?;
    ctx.advance_slot(1)?;

    let deactivate_tx =
        DeactivateCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(deactivate_tx);

    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x20b", &tx_result);

    Ok(())
}
