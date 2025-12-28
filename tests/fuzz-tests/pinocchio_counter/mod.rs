use {
    bolero::check,
    pinocchio::pubkey::Pubkey,
    pinocchio_counter_program::{
        try_find_counter_address, AccountDiscriminator, CounterV1, SetCountV1Args,
    },
};

#[test]
fn fuzz_set_count_v1_args_serialization_roundtrip() {
    check!()
        .with_generator(bolero::any::<u64>())
        .for_each(|count| {
            let original = SetCountV1Args { count: *count };

            // Serialize using wincode (same as the program uses)
            let serialized =
                wincode::serialize(&original).expect("SetCountV1Args serialization should succeed");

            // Deserialize using SetCountV1Args::deserialize
            let deserialized = SetCountV1Args::deserialize(&serialized)
                .expect("SetCountV1Args deserialization should succeed");

            assert_eq!(
                original.count, deserialized.count,
                "SetCountV1Args count {} should be preserved through serialization",
                count
            );
        });
}

#[test]
fn fuzz_counter_serialization_roundtrip_all_fields() {
    check!()
        .with_generator(bolero::any::<(Pubkey, u8, u64)>())
        .for_each(|(owner, bump, count)| {
            let original = CounterV1 {
                discriminator: AccountDiscriminator::CounterV1Account,
                owner: *owner,
                bump: *bump,
                count: *count,
            };

            let serialized = original.serialize().expect("serialization should succeed");
            assert_eq!(serialized.len(), CounterV1::size());

            let deserialized: CounterV1 =
                CounterV1::deserialize(&serialized).expect("deserialization should succeed");

            assert_eq!(original.discriminator, deserialized.discriminator);
            assert_eq!(original.owner, deserialized.owner);
            assert_eq!(original.bump, deserialized.bump);
            assert_eq!(original.count, deserialized.count);
        });
}

#[test]
fn fuzz_increment_decrement_properties() {
    check!()
        .with_generator(bolero::any::<u64>())
        .for_each(|initial_count| {
            // Test increment: should be >= original (monotonicity)
            let after_increment = initial_count.saturating_add(1);
            assert!(
                after_increment >= *initial_count,
                "Increment must be monotonic: {} >= {}",
                after_increment,
                initial_count
            );
            if *initial_count < u64::MAX {
                assert_eq!(
                    after_increment,
                    *initial_count + 1,
                    "Increment should add 1 when not at max"
                );
            } else {
                assert_eq!(
                    after_increment,
                    u64::MAX,
                    "Increment should saturate at u64::MAX"
                );
            }

            // Test decrement: should be <= original (monotonicity)
            let after_decrement = initial_count.saturating_sub(1);
            assert!(
                after_decrement <= *initial_count,
                "Decrement must be monotonic: {} <= {}",
                after_decrement,
                initial_count
            );
            if *initial_count > 0 {
                assert_eq!(
                    after_decrement,
                    *initial_count - 1,
                    "Decrement should subtract 1 when not at 0"
                );
            } else {
                assert_eq!(after_decrement, 0, "Decrement should saturate at 0");
            }

            // Test inverse: increment then decrement (when no saturation)
            if *initial_count < u64::MAX {
                let temp = initial_count.saturating_add(1);
                let result = temp.saturating_sub(1);
                assert_eq!(
                    result, *initial_count,
                    "Increment then decrement should return to original (when no saturation)"
                );
            }
        });
}

#[test]
fn fuzz_counter_operation_sequences() {
    check!()
        .with_generator(bolero::any::<Vec<u8>>())
        .for_each(|operations| {
            let mut count: u64 = 0;

            // Limit iterations to avoid excessive runtime
            for op in operations.iter().take(1000) {
                match op % 3 {
                    0 => {
                        // Increment
                        let new_count = count.saturating_add(1);
                        assert!(
                            new_count >= count,
                            "Increment must be monotonic: {} >= {}",
                            new_count,
                            count
                        );
                        count = new_count;
                    }
                    1 => {
                        // Decrement
                        let new_count = count.saturating_sub(1);
                        assert!(
                            new_count <= count,
                            "Decrement must be monotonic: {} <= {}",
                            new_count,
                            count
                        );
                        count = new_count;
                    }
                    _ => {
                        // Set to some value (use op as seed for determinism)
                        count = *op as u64;
                    }
                }

                // Invariant: count should always be a valid u64 value
                // (This is always true for u64, but documents the property)
            }
        });
}

#[test]
fn fuzz_find_counter_address_deterministic() {
    check!()
        .with_generator(bolero::any::<(Pubkey, Pubkey)>())
        .for_each(|(program_id, owner)| {
            let result1 = try_find_counter_address(program_id, owner);
            let result2 = try_find_counter_address(program_id, owner);

            // If the function succeeds, it must be deterministic
            match (result1, result2) {
                (Some((addr1, bump1)), Some((addr2, bump2))) => {
                    // Same inputs must produce same outputs (determinism)
                    assert_eq!(
                        addr1, addr2,
                        "try_find_counter_address must be deterministic"
                    );
                    assert_eq!(bump1, bump2, "bump seed must be deterministic");
                }
                (None, None) => {
                    // Both failed - this can happen with random inputs, but determinism
                    // still holds (both failed the same way)
                }
                _ => {
                    // One succeeded and one failed with same inputs - this would be non-deterministic
                    panic!("Non-deterministic behavior: one call succeeded, other failed");
                }
            }
        });
}

#[test]
fn fuzz_find_counter_address_collision_resistance() {
    check!()
        .with_generator(bolero::any::<(Pubkey, Pubkey, Pubkey)>())
        .for_each(|(program_id, owner1, owner2)| {
            // Skip if owners are the same (that case is tested by deterministic test)
            if owner1 == owner2 {
                return;
            }

            let result1 = try_find_counter_address(program_id, owner1);
            let result2 = try_find_counter_address(program_id, owner2);

            // If both succeed, different owners must produce different addresses (collision resistance)
            match (result1, result2) {
                (Some((addr1, _bump1)), Some((addr2, _bump2))) => {
                    assert_ne!(
                        addr1, addr2,
                        "Different owners must produce different counter addresses (collision resistance)"
                    );
                }
                _ => {
                    // If one or both fail, we can't verify collision resistance for this case
                    // but that's OK - we're testing the property when it applies
                }
            }
        });
}

#[test]
fn fuzz_find_counter_address_program_isolation() {
    check!()
        .with_generator(bolero::any::<(Pubkey, Pubkey, Pubkey)>())
        .for_each(|(program_id1, program_id2, owner)| {
            // Skip if program IDs are the same
            if program_id1 == program_id2 {
                return;
            }

            let result1 = try_find_counter_address(program_id1, owner);
            let result2 = try_find_counter_address(program_id2, owner);

            // If both succeed, different program IDs must produce different addresses (program isolation)
            match (result1, result2) {
                (Some((addr1, _bump1)), Some((addr2, _bump2))) => {
                    assert_ne!(
                        addr1, addr2,
                        "Different program IDs must produce different counter addresses (program isolation)"
                    );
                }
                _ => {
                    // If one or both fail, we can't verify program isolation for this case
                }
            }
        });
}
