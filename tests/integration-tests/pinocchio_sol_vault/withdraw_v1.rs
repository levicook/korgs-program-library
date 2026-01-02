use {
    crate::{
        litesvm_utils::demand_tx_success,
        pinocchio_sol_vault::{TestContext, TestResult},
    },
    pinocchio_sol_vault_client::{
        find_vault_v1_address,
        transactions::{DepositV1SimpleTx, InitializeVaultV1SimpleTx, WithdrawV1SimpleTx},
    },
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

    // Deposit some SOL
    let deposit_amount = 100_000_000; // 0.1 SOL
    let deposit_tx = DepositV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp.insecure_clone(),
        deposit_amount,
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(deposit_tx);
    demand_tx_success(&tx_result);

    // Get balances before withdraw
    let vault_pk = find_vault_v1_address(&ctx.program_id(), &owner_pk);
    let vault_balance_before = ctx.get_account(vault_pk).unwrap().lamports;
    let owner_balance_before = ctx.get_account(owner_pk).unwrap().lamports;
    let withdraw_amount = 50_000_000; // 0.05 SOL

    // Withdraw
    let withdraw_tx = WithdrawV1SimpleTx::try_new(
        ctx.program_id(),
        owner_kp,
        withdraw_amount,
        ctx.latest_blockhash(),
    )?;
    let tx_result = ctx.send_transaction(withdraw_tx);
    demand_tx_success(&tx_result);

    // Verify balances
    let vault_account = ctx.get_account(vault_pk).ok_or("Vault account not found")?;
    let owner_account = ctx.get_account(owner_pk).ok_or("Owner account not found")?;

    assert_eq!(
        vault_account.lamports,
        vault_balance_before - withdraw_amount,
        "Vault balance should decrease by withdraw amount"
    );

    assert!(
        owner_account.lamports > owner_balance_before,
        "Owner balance should increase"
    );

    Ok(())
}
