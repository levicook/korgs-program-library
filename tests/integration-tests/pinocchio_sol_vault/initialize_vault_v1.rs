use {
    crate::{
        litesvm_utils::demand_tx_success,
        pinocchio_sol_vault::{TestContext, TestResult},
    },
    pinocchio_sol_vault_client::{find_vault_v1, transactions::InitializeVaultV1SimpleTx},
    pinocchio_sol_vault_program::{AccountDiscriminator, VaultV1},
    solana_keypair::Signer,
};

#[test]
fn succeeds() -> TestResult {
    let mut ctx = TestContext::try_new()?;
    let owner_kp = ctx.create_funded_keypair();
    let owner_pk = owner_kp.pubkey();

    let init_vault_tx =
        InitializeVaultV1SimpleTx::try_new(ctx.program_id(), owner_kp, ctx.latest_blockhash())?;

    let tx_result = ctx.send_transaction(init_vault_tx.into());
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
