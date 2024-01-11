use super::*;

#[test]
fn try_wire_delay_map_error() {
    for run_number in 0..=6999 {
        assert!(try_wire_delay(run_number).is_err());
    }
}

#[test]
fn try_wire_delay_correctness_sim() {
    assert_eq!(try_wire_delay(u32::MAX).unwrap(), 100);
}

#[test]
fn try_wire_delay_correctness_9567() {
    assert_eq!(try_wire_delay(9567).unwrap(), 129);
}

#[test]
#[should_panic]
fn safe_guard_try_wire_delay() {
    let _ = try_wire_delay(u32::MAX - 1);
}
