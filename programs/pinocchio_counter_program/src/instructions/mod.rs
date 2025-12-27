mod discriminator;

mod deactivate_counter_v1;
mod initialize_counter_v1;

mod decrement_count_v1;
mod increment_count_v1;
mod set_count_v1;

pub use {
    deactivate_counter_v1::{DeactivateCounterV1, DeactivateCounterV1Error},
    decrement_count_v1::{DecrementCountV1, DecrementCountV1Error},
    discriminator::InstructionDiscriminator,
    increment_count_v1::{IncrementCountV1, IncrementCountV1Error},
    initialize_counter_v1::{InitializeCounterV1, InitializeCounterV1Error},
    set_count_v1::{SetCountV1, SetCountV1Args, SetCountV1Error},
};
