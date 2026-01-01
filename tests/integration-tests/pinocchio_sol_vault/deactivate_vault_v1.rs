use {
    crate::{
        litesvm_utils::demand_tx_success,
        pinocchio_sol_vault::{TestContext, TestResult},
    },
    pinocchio_sol_vault_client::{
        find_vault_v1_address,
        transactions::{DeactivateVaultV1SimpleTx, InitializeVaultV1SimpleTx},
    },
    pinocchio_sol_vault_program::{AccountDiscriminator, DEACTIVATED_ACCOUNT_SIZE},
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
    let tx_result = ctx.send_transaction(init_tx.into());
    demand_tx_success(&tx_result);

    let vault_pk = find_vault_v1_address(&ctx.program_id(), &owner_pk);
    let vault_balance_before = ctx.get_account(vault_pk).unwrap().lamports;

    // Deactivate
    let deactivate_tx =
        DeactivateVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;
    let tx_result = ctx.send_transaction(deactivate_tx.into());
    demand_tx_success(&tx_result);

    // Verify vault is deactivated
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;

    assert_eq!(
        vault_account.data.len(),
        DEACTIVATED_ACCOUNT_SIZE,
        "Vault should be resized to 1 byte"
    );

    assert_eq!(
        vault_account.data[0],
        u8::from(AccountDiscriminator::DeactivatedAccount),
        "Vault should have DeactivatedAccount discriminator"
    );

    // Vault should have less lamports (non-rent lamports transferred to owner)
    assert!(
        vault_account.lamports < vault_balance_before,
        "Vault should have fewer lamports after deactivation"
    );

    Ok(())
}
