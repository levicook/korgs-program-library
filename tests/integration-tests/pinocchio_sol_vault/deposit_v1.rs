use {
    crate::{
        litesvm_utils::demand_tx_success,
        pinocchio_sol_vault::{TestContext, TestResult},
    },
    pinocchio_sol_vault_client::{
        find_vault_v1_address,
        transactions::{DepositV1SimpleTx, InitializeVaultV1SimpleTx},
    },
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
    let tx_result = ctx.send_transaction(init_tx.into());
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
    let tx_result = ctx.send_transaction(deposit_tx.into());
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
