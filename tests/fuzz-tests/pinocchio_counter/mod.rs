use {
    bolero::check,
    pinocchio_counter_program::{AccountDiscriminator, CounterV1},
    pinocchio::pubkey::Pubkey,
};

#[test]
fn fuzz_counter_serialization_roundtrip() {
    check!()
        .with_generator(bolero::any::<u64>())
        .for_each(|count| {
            let original = CounterV1 {
                discriminator: AccountDiscriminator::CounterV1Account,
                owner: Pubkey::default(),
                bump: 0,
                count: *count,
                reserved: [0; 31],
            };

            let serialized = original.serialize().expect("serialization should succeed");
            assert_eq!(serialized.len(), CounterV1::size());

            let deserialized: CounterV1 = CounterV1::deserialize(&serialized)
                .expect("deserialization should succeed");
            
            assert_eq!(original.discriminator, deserialized.discriminator);
            assert_eq!(original.owner, deserialized.owner);
            assert_eq!(original.bump, deserialized.bump);
            assert_eq!(original.count, deserialized.count);
            assert_eq!(original.reserved, deserialized.reserved);
        });
}

