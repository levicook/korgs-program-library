use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_counter::{
            malicious_builders::{MaliciousReactivateCounterV1Ix, MaliciousReactivateCounterV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_counter_client::{
        find_counter_v1_address,
        transactions::{
            DeactivateCounterV1SimpleTx, IncrementCountV1SimpleTx, InitializeCounterV1SimpleTx,
            ReactivateCounterV1SimpleTx,
        },
    },
    pinocchio_counter_program::{AccountDiscriminator, CounterV1, DEACTIVATED_ACCOUNT_SIZE},
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
    solana_rent::Rent,
};

#[test]
fn succeeds_full_lifecycle() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Step 1: Initialize the counter
    let initialize_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(initialize_tx);
    demand_tx_success(&tx_result);

    let counter_pk = find_counter_v1_address(&ctx.program_id(), &owner_pk);
    let counter_account_initial = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should exist")?;

    assert_eq!(
        counter_account_initial.data[0],
        u8::from(AccountDiscriminator::CounterV1Account),
        "Counter should have CounterV1Account discriminator"
    );

    ctx.advance_slot(1)?;

    // Step 2: Deactivate the counter
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    let counter_account_deactivated = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should still exist")?;

    assert_eq!(
        counter_account_deactivated.data.len(),
        DEACTIVATED_ACCOUNT_SIZE,
        "Counter account should be resized to 1 byte"
    );

    assert_eq!(
        counter_account_deactivated.data[0],
        u8::from(AccountDiscriminator::DeactivatedAccount),
        "Counter should have DeactivatedAccount discriminator"
    );

    let owner_lamports_before_reactivate = ctx
        .get_account(owner_pk)
        .ok_or("Owner account should exist")?
        .lamports;

    ctx.advance_slot(1)?;

    // Step 3: Reactivate the counter
    let reactivate_tx =
        ReactivateCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(reactivate_tx);
    demand_tx_success(&tx_result);

    // Verify counter account is reactivated
    let counter_account_reactivated = ctx
        .get_account(counter_pk)
        .ok_or("Counter account should still exist")?;

    assert_eq!(
        counter_account_reactivated.data.len(),
        CounterV1::size(),
        "Counter account should be resized to CounterV1::size()"
    );

    assert_eq!(
        counter_account_reactivated.data[0],
        u8::from(AccountDiscriminator::CounterV1Account),
        "Counter should have CounterV1Account discriminator"
    );

    let counter = CounterV1::deserialize(&counter_account_reactivated.data)?;
    assert_eq!(
        counter.discriminator,
        AccountDiscriminator::CounterV1Account,
        "Counter discriminator should be CounterV1Account"
    );
    assert_eq!(
        counter.owner.as_ref(),
        owner_pk.as_ref(),
        "Counter owner should match"
    );
    assert_eq!(counter.count, 0, "Counter count should be reset to 0");

    // Verify lamports were transferred from owner to counter for rent
    let rent = Rent::default();
    let rent_exempt_minimum_counter = rent.minimum_balance(CounterV1::size());
    let rent_exempt_minimum_deactivated = rent.minimum_balance(DEACTIVATED_ACCOUNT_SIZE);
    let additional_lamports_needed = rent_exempt_minimum_counter - rent_exempt_minimum_deactivated;

    let owner_lamports_after_reactivate = ctx
        .get_account(owner_pk)
        .ok_or("Owner account should exist")?
        .lamports;

    let lamports_spent =
        owner_lamports_before_reactivate.saturating_sub(owner_lamports_after_reactivate);

    // Owner should have spent additional lamports for rent (plus transaction fees)
    assert!(
        lamports_spent >= additional_lamports_needed,
        "Owner should have spent at least {additional_lamports_needed} lamports for rent, but spent {lamports_spent}"
    );

    // Counter should have rent-exempt minimum for full size
    assert_eq!(
        counter_account_reactivated.lamports, rent_exempt_minimum_counter,
        "Counter should have rent-exempt minimum for CounterV1::size()"
    );

    Ok(())
}

#[test]
fn succeeds_after_increment_and_deactivate() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize
    let initialize_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(initialize_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Increment a few times
    for _ in 0..5 {
        let increment_tx = IncrementCountV1SimpleTx::try_new(
            ctx.program_id(),
            owner_kp.insecure_clone(),
            ctx.latest_blockhash(),
        )?;
        let tx_result = ctx.send_transaction(increment_tx);
        demand_tx_success(&tx_result);
        ctx.advance_slot(1)?;
    }

    // Verify count is 5
    let counter_pk = find_counter_v1_address(&ctx.program_id(), &owner_pk);
    let counter_account = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
    let counter = CounterV1::deserialize(&counter_account.data)?;
    assert_eq!(counter.count, 5, "Counter should have count of 5");

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Reactivate - count should be reset to 0
    let reactivate_tx =
        ReactivateCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;
    let tx_result = ctx.send_transaction(reactivate_tx);
    demand_tx_success(&tx_result);

    let counter_account_reactivated = ctx.get_account(counter_pk).ok_or("Counter should exist")?;
    let counter_reactivated = CounterV1::deserialize(&counter_account_reactivated.data)?;
    assert_eq!(
        counter_reactivated.count, 0,
        "Counter count should be reset to 0 after reactivation"
    );

    Ok(())
}

#[test]
fn fails_when_payer_not_signer() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with payer not as signer
    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousReactivateCounterV1Ix::with_payer_not_signer)
    .with_different_signer(fee_payer_kp)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x602", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_writable() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with counter not writable
    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousReactivateCounterV1Ix::with_counter_not_writable)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x604", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with wrong counter address
    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousReactivateCounterV1Ix::with_random_counter_address)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x605", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with missing accounts
    let malicious_ix = MaliciousReactivateCounterV1Ix::from_valid(ctx.program_id(), owner_pk);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        // Missing counter and system_program - only 1 account instead of 3
    ]);

    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_instruction(instruction)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x601", &tx_result);

    Ok(())
}

#[test]
fn fails_when_counter_not_deactivated() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize (but don't deactivate)
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate a counter that's not deactivated
    let reactivate_tx =
        ReactivateCounterV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(reactivate_tx);
    demand_tx_failure(&tx_result);
    // Should fail with AccountDiscriminatorError because it expects DeactivatedAccount
    demand_logs_contain("failed: custom program error: 0x60c", &tx_result);

    Ok(())
}

#[test]
fn fails_with_invalid_instruction_discriminator() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with invalid discriminator
    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
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

    // Initialize
    let init_counter_tx = InitializeCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_counter_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate
    let deactivate_tx = DeactivateCounterV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Try to reactivate with empty instruction data
    let malicious_tx = MaliciousReactivateCounterV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousReactivateCounterV1Ix::with_empty_data)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x1", &tx_result);

    Ok(())
}
