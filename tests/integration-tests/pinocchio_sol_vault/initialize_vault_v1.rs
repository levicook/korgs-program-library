use {
    crate::{
        litesvm_utils::{demand_logs_contain, demand_tx_failure, demand_tx_success},
        pinocchio_sol_vault::{
            malicious_builders::{MaliciousInitializeVaultV1Ix, MaliciousInitializeVaultV1Tx},
            TestContext, TestResult,
        },
    },
    pinocchio_sol_vault_client::{find_vault_v1, transactions::InitializeVaultV1SimpleTx},
    pinocchio_sol_vault_program::{AccountDiscriminator, VaultV1},
    solana_account::Account,
    solana_instruction::AccountMeta,
    solana_keypair::Signer,
    solana_pubkey::Pubkey,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let init_vault_tx =
        InitializeVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(init_vault_tx);
    demand_tx_success(&tx_result);

    let (vault_pk, bump) = find_vault_v1(&ctx.program_id(), &owner_pk);

    let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;

    assert_eq!(vault_account.data.len(), VaultV1::size());

    assert_eq!(
        vault_account.data[0],
        u8::from(AccountDiscriminator::VaultV1Account)
    );

    assert_ne!(vault_account.lamports, 0, "Vault should have lamports");

    assert_eq!(
        vault_account.owner,
        ctx.program_id(),
        "Owner mismatch expected {expected:?}, observed {observed:?}",
        expected = ctx.program_id(),
        observed = vault_account.owner
    );

    let vault = VaultV1::from_bytes(&vault_account.data)
        .map_err(|e| format!("Failed to deserialize vault: {:?}", e))?;

    assert_eq!(
        vault.discriminator,
        AccountDiscriminator::VaultV1Account,
        "Discriminator mismatch expected {expected:?}, observed {observed:?}",
        expected = AccountDiscriminator::VaultV1Account,
        observed = vault.discriminator
    );

    assert_eq!(
        vault.owner().as_ref(),
        owner_pk.as_ref(),
        "Owner mismatch expected {expected:?}, observed {observed:?}",
        expected = owner_pk,
        observed = vault.owner(),
    );

    assert_eq!(
        vault.bump,
        bump,
        "Bump mismatch expected {expected:?}, observed {observed:?}",
        expected = bump,
        observed = vault.bump
    );

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

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousInitializeVaultV1Ix::with_payer_not_signer)
    .with_different_signer(fee_payer_kp)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x102", &tx_result);

    Ok(())
}

#[test]
fn fails_when_vault_not_writable() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousInitializeVaultV1Ix::with_vault_not_writable)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x104", &tx_result);

    Ok(())
}

#[test]
fn fails_when_vault_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousInitializeVaultV1Ix::with_random_vault_address)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x105", &tx_result);

    Ok(())
}

#[test]
fn fails_when_system_program_address_mismatch() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousInitializeVaultV1Ix::with_random_system_program)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x109", &tx_result);

    Ok(())
}

#[test]
fn fails_when_not_enough_accounts() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let (vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner_pk);

    let malicious_ix = MaliciousInitializeVaultV1Ix::from_valid(ctx.program_id(), owner_pk);
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

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
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

// ============================================================================
// Malicious Transaction Tests - State Validation Failures
// ============================================================================

#[test]
fn fails_when_vault_has_pre_existing_data() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault first time
    let init_vault_tx1 = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_vault_tx1);
    demand_tx_success(&tx_result);

    let (vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner_pk);
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault should exist")?;
    assert!(!vault_account.data.is_empty(), "Vault should have data");

    ctx.advance_slot(1)?;

    // Attempt to re-initialize - should fail
    let init_vault_tx2 =
        InitializeVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result2 = ctx.send_transaction(init_vault_tx2);
    demand_tx_failure(&tx_result2);
    demand_logs_contain("failed: custom program error: 0x106", &tx_result2);

    Ok(())
}

#[test]
fn fails_when_vault_has_non_zero_lamports() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let (vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner_pk);

    let vault_account = Account {
        lamports: 1_000_000,
        owner: Pubkey::new_unique(),
        ..Default::default()
    };
    ctx.set_account(vault_pk, vault_account)?;

    ctx.advance_slot(1)?;

    // Attempt to initialize - should fail because vault has non-zero lamports
    let init_vault_tx =
        InitializeVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(init_vault_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x107", &tx_result);

    Ok(())
}

#[test]
fn fails_when_reinitializing_deactivated_vault() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault
    use pinocchio_sol_vault_client::transactions::DeactivateVaultV1SimpleTx;
    let init_vault_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(init_vault_tx);
    demand_tx_success(&tx_result);

    ctx.advance_slot(1)?;

    // Deactivate the vault
    let deactivate_tx = DeactivateVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;

    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    let (vault_pk, _) = find_vault_v1(&ctx.program_id(), &owner_pk);
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault should exist")?;
    // Deactivated vault should have 1 byte (discriminator)
    assert_eq!(
        vault_account.data.len(),
        1,
        "Deactivated vault should be 1 byte"
    );
    assert_eq!(
        vault_account.data[0],
        u8::from(AccountDiscriminator::DeactivatedAccount),
        "Deactivated vault should have DeactivatedAccount discriminator"
    );

    ctx.advance_slot(1)?;

    // Attempt to re-initialize the deactivated vault - should fail
    let reinit_vault_tx =
        InitializeVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(reinit_vault_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x106", &tx_result);

    Ok(())
}

// ============================================================================
// Malicious Transaction Tests - Instruction Validation Failures
// ============================================================================

#[test]
fn fails_with_invalid_instruction_discriminator() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(|ix: MaliciousInitializeVaultV1Ix| {
        ix.with_invalid_discriminator(255)
    })
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

    let malicious_tx = MaliciousInitializeVaultV1Tx::from_valid(
        ctx.program_id(),
        owner_kp,
        ctx.latest_blockhash(),
    )
    .with_malicious_instruction(MaliciousInitializeVaultV1Ix::with_empty_data)
    .build();

    let tx_result = ctx.send_transaction(malicious_tx);
    demand_tx_failure(&tx_result);
    demand_logs_contain("failed: custom program error: 0x1", &tx_result);

    Ok(())
}
