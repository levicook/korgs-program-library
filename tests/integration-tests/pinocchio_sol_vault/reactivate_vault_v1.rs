use {
    crate::{
        litesvm_utils::demand_tx_success,
        pinocchio_sol_vault::{TestContext, TestResult},
    },
    pinocchio_sol_vault_client::{
        find_vault_v1_address,
        transactions::{
            DeactivateVaultV1SimpleTx, InitializeVaultV1SimpleTx, ReactivateVaultV1SimpleTx,
        },
    },
    pinocchio_sol_vault_program::{AccountDiscriminator, VaultV1},
    solana_keypair::Signer,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    // Initialize vault
    let init_tx = InitializeVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(init_tx);
    demand_tx_success(&tx_result);

    let vault_pk = find_vault_v1_address(&ctx.program_id(), &owner_pk);

    // Deactivate vault
    let deactivate_tx = DeactivateVaultV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deactivate_tx);
    demand_tx_success(&tx_result);

    // Verify deactivated
    let vault_account = ctx.get_account(vault_pk).unwrap();
    assert_eq!(
        vault_account.data[0],
        u8::from(AccountDiscriminator::DeactivatedAccount)
    );

    // Reactivate
    let reactivate_tx =
        ReactivateVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;
    let tx_result = ctx.send_transaction(reactivate_tx);
    demand_tx_success(&tx_result);

    // Verify reactivated
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;

    assert_eq!(
        vault_account.data.len(),
        VaultV1::size(),
        "Vault should be restored to full size"
    );

    let vault = VaultV1::from_bytes(&vault_account.data)
        .map_err(|e| format!("Failed to deserialize vault: {:?}", e))?;
    assert_eq!(
        vault.discriminator,
        AccountDiscriminator::VaultV1Account,
        "Vault should have VaultV1Account discriminator"
    );

    assert_eq!(
        vault.owner().as_ref(),
        owner_pk.as_ref(),
        "Vault owner should match"
    );

    Ok(())
}
