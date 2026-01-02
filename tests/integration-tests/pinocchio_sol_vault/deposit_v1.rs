use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_sol_vault::{
            malicious_builders::{MaliciousDepositV1Ix, MaliciousDepositV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_sol_vault_client::{
        find_vault_v1, find_vault_v1_address,
        transactions::{DepositV1SimpleTx, InitializeVaultV1SimpleTx},
    },
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    // Get initial balances
    let vault_pk = find_vault_v1_address(&ctx.program_id(), &owner_pk);
    let initial_vault_balance = ctx.get_account(vault_pk).unwrap().lamports;
    let deposit_amount = 100_000_000; // 0.1 SOL

    // Deposit
    let deposit_tx = DepositV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp,
        deposit_amount,
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deposit_tx);
    demand_tx_success(&tx_result);

    // Verify vault balance increased
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;
    assert_eq!(
        vault_account.lamports,
        initial_vault_balance + deposit_amount,
        "Vault balance should increase by deposit amount"
    );

    Ok(())
}

#[test]
fn succeeds_with_multiple_deposits() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    let vault_pk = find_vault_v1_address(&ctx.program_id(), &owner_pk);
    let initial_vault_balance = ctx.get_account(vault_pk).unwrap().lamports;

    ctx.advance_slot(1)?;

    // Make multiple deposits
    let deposit_amount = 50_000_000; // 0.05 SOL
    for i in 1..=3 {
        let deposit_tx = DepositV1SimpleTx::try_new(
            ctx.program_id(),
            owner_kp.insecure_clone(),
            deposit_amount,
            ctx.latest_blockhash(),
        )?;
        let tx_result = ctx.send_transaction(deposit_tx);
        demand_tx_success(&tx_result);

        ctx.advance_slot(1)?;

        let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;
        assert_eq!(
            vault_account.lamports,
            initial_vault_balance + (deposit_amount * i),
            "Vault balance should accumulate deposits"
        );
    }

    Ok(())
}

// ============================================================================
// Malicious Transaction Tests - Account Validation Failures
// ============================================================================

#[test]
fn fails_when_owner_not_signer() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let fee_payer_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousDepositV1Ix::with_owner_not_signer)
    .with_different_signer(fee_payer_kp)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x202", &tx_result);

    Ok(())
}

#[test]
fn fails_when_vault_not_writable() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousDepositV1Ix::with_vault_not_writable)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x204", &tx_result);

    Ok(())
}

#[test]
fn fails_when_vault_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousDepositV1Ix::with_random_vault_address)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x205", &tx_result);

    Ok(())
}

#[test]
fn fails_when_system_program_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousDepositV1Ix::with_random_system_program)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x207", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let (vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner_pk);

    let malicious_ix = MaliciousDepositV1Ix::from_valid(ctx.program_id(), owner_pk, 100_000_000);
    let instruction = malicious_ix.build_with_accounts(vec![
        AccountMeta {
            pubkey: owner_pk,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: vault_pk,
            is_signer: false,
            is_writable: true,
        },
        // Missing system_program - only 2 accounts instead of 3
    ]);

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_instruction(instruction)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x201", &tx_result);

    Ok(())
}

// ============================================================================
// Malicious Transaction Tests - State Validation Failures
// ============================================================================

#[test]
fn fails_when_vault_not_initialized() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Don't initialize vault - try to deposit to uninitialized vault
    let deposit_tx = DepositV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(deposit_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x206", &tx_result);

    Ok(())
}

#[test]
fn fails_when_owner_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner1_kp = ctx.create_funded_keypair();
    let owner2_kp = ctx.create_funded_keypair();
    let owner1_pk = owner1_kp.pubkey();

    // Initialize vault for owner1
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner1_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Get owner1's vault address
    let (owner1_vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner1_pk);

    // Try to deposit as owner2, but pass owner1's vault address
    // This should pass the vault address check (we're providing owner1's vault)
    // But fail on owner mismatch check (vault state says owner1, but signer is owner2)
    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner2_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(|ix| ix.with_vault_address(owner1_vault_pk))
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x205", &tx_result);

    Ok(())
}

// ============================================================================
// Malicious Transaction Tests - Instruction Validation Failures
// ============================================================================

#[test]
fn fails_with_invalid_instruction_data() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousDepositV1Ix::with_invalid_data)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x209", &tx_result);

    Ok(())
}

#[test]
fn fails_with_invalid_instruction_discriminator() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    // Initialize vault first
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    let malicious_tx = MaliciousDepositV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        100_000_000,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(|ix: MaliciousDepositV1Ix| ix.with_invalid_discriminator(255))
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x2", &tx_result);

    Ok(())
}
