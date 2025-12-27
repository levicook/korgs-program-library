#[kani::proof]
fn verify_saturating_add() {
    let x: u64 = kani::any();
    let result = x.saturating_add(1);
    assert!(result >= x);
    if x < u64::MAX {
        assert!(result == x + 1);
    } else {
        assert!(result == u64::MAX);
    }
}

#[kani::proof]
fn verify_saturating_sub() {
    let x: u64 = kani::any();
    let result = x.saturating_sub(1);
    assert!(result <= x);
    if x > 0 {
        assert!(result == x - 1);
    } else {
        assert!(result == 0);
    }
}
