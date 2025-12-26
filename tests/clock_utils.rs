use solana_clock::{Clock, DEFAULT_MS_PER_SLOT, DEFAULT_SLOTS_PER_EPOCH};

/// Advances a Clock by the specified number of slots.
///
/// Updates all Clock fields appropriately.
/// - Increments `slot` by `n_slots`
/// - Recalculates `epoch` based on the new slot
/// - Updates `epoch_start_timestamp` if the epoch changed
/// - Advances `unix_timestamp` proportionally to slot duration
/// - Preserves `leader_schedule_epoch`
pub fn advance_clock(clock: Clock, n_slots: u64) -> Clock {
    let new_slot = clock.slot + n_slots;
    let new_epoch = new_slot / DEFAULT_SLOTS_PER_EPOCH;

    // Calculate time advancement: n_slots * milliseconds_per_slot / 1000 = seconds
    let seconds_to_advance = (n_slots * DEFAULT_MS_PER_SLOT) / 1000;
    let new_unix_timestamp = clock.unix_timestamp + seconds_to_advance as i64;

    // If epoch changed, update epoch_start_timestamp to the timestamp at the start of the new epoch
    let new_epoch_start_timestamp = if new_epoch != clock.epoch {
        // Calculate timestamp at the start of the new epoch
        // Slot at epoch start = new_epoch * DEFAULT_SLOTS_PER_EPOCH
        let slots_into_new_epoch = new_slot % DEFAULT_SLOTS_PER_EPOCH;
        let seconds_into_new_epoch = (slots_into_new_epoch * DEFAULT_MS_PER_SLOT) / 1000;
        new_unix_timestamp - seconds_into_new_epoch as i64
    } else {
        clock.epoch_start_timestamp
    };

    Clock {
        slot: new_slot,
        epoch: new_epoch,
        epoch_start_timestamp: new_epoch_start_timestamp,
        unix_timestamp: new_unix_timestamp,
        leader_schedule_epoch: clock.leader_schedule_epoch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_within_epoch() {
        let clock = Clock {
            slot: 1000,
            epoch: 0,
            epoch_start_timestamp: 0,
            unix_timestamp: 400,
            leader_schedule_epoch: 0,
        };

        let advanced = advance_clock(clock, 100);
        assert_eq!(advanced.slot, 1100);
        assert_eq!(advanced.epoch, 0);
        assert_eq!(advanced.epoch_start_timestamp, 0);
        assert_eq!(advanced.unix_timestamp, 440); // 400 + (100 * 400ms / 1000) = 400 + 40
    }

    #[test]
    fn test_advance_crosses_epoch_boundary() {
        let clock = Clock {
            slot: DEFAULT_SLOTS_PER_EPOCH - 100,
            epoch: 0,
            epoch_start_timestamp: 0,
            unix_timestamp: 172800, // 2 days in seconds (approximate)
            leader_schedule_epoch: 0,
        };

        let old_epoch_start_timestamp = clock.epoch_start_timestamp;
        let advanced = advance_clock(clock, 200); // Crosses into epoch 1
        assert_eq!(advanced.slot, DEFAULT_SLOTS_PER_EPOCH + 100);
        assert_eq!(advanced.epoch, 1);
        // epoch_start_timestamp should be set to timestamp at slot DEFAULT_SLOTS_PER_EPOCH
        assert!(advanced.epoch_start_timestamp > old_epoch_start_timestamp);
    }
}
