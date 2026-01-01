use {
    bolero::check,
    pinocchio::pubkey::Pubkey,
    pinocchio_sol_vault_program::{
        try_find_vault_v1, AccountDiscriminator, VaultV1,
    },
};

#[test]
fn fuzz_vault_v1_serialization_roundtrip() {
    check!()
        .with_generator(bolero::any::<(Pubkey, u8)>())
        .for_each(|(owner, bump)| {
            let mut owner_bytes = [0u8; 32];
            owner_bytes.copy_from_slice(owner.as_ref());
            
            let original = VaultV1 {
                discriminator: AccountDiscriminator::VaultV1Account,
                owner: owner_bytes,
                bump: *bump,
            };

            let serialized = original.to_bytes();
            assert_eq!(serialized.len(), VaultV1::size());

            let deserialized = VaultV1::from_bytes(&serialized)
                .expect("VaultV1 deserialization should succeed");

            assert_eq!(original.discriminator, deserialized.discriminator);
            assert_eq!(original.owner, deserialized.owner);
            assert_eq!(original.bump, deserialized.bump);
        });
}

#[test]
fn fuzz_deposit_v1_args_serialization_roundtrip() {
    check!()
        .with_generator(bolero::any::<u64>())
        .for_each(|amount| {
            let serialized = amount.to_le_bytes();
            let deserialized = u64::from_le_bytes(
                serialized.try_into().expect("Should be 8 bytes")
            );

            assert_eq!(
                *amount, deserialized,
                "Deposit amount {} should be preserved through serialization",
                amount
            );
        });
}

#[test]
fn fuzz_withdraw_v1_args_serialization_roundtrip() {
    check!()
        .with_generator(bolero::any::<u64>())
        .for_each(|amount| {
            let serialized = amount.to_le_bytes();
            let deserialized = u64::from_le_bytes(
                serialized.try_into().expect("Should be 8 bytes")
            );

            assert_eq!(
                *amount, deserialized,
                "Withdraw amount {} should be preserved through serialization",
                amount
            );
        });
}

#[test]
fn fuzz_find_vault_address_deterministic() {
    check!()
        .with_generator(bolero::any::<(Pubkey, Pubkey)>())
        .for_each(|(program_id, owner)| {
            let result1 = try_find_vault_v1(program_id, owner);
            let result2 = try_find_vault_v1(program_id, owner);

            // If the function succeeds, it must be deterministic
            match (result1, result2) {
                (Some((addr1, bump1)), Some((addr2, bump2))) => {
                    assert_eq!(
                        addr1, addr2,
                        "try_find_vault_v1 must be deterministic"
                    );
                    assert_eq!(bump1, bump2, "bump seed must be deterministic");
                }
                (None, None) => {
                    // Both failed - determinism still holds
                }
                _ => {
                    panic!("Non-deterministic behavior: one call succeeded, other failed");
                }
            }
        });
}

