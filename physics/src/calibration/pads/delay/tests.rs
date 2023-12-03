use super::*;

#[test]
fn try_pad_delay_map_error() {
    for run_number in 0..=9566 {
        assert!(try_pad_delay(run_number).is_err());
    }
}

#[test]
fn try_pad_delay_correctness_sim() {
    assert_eq!(try_pad_delay(u32::MAX).unwrap(), 100);
}

#[test]
fn try_pad_delay_correctness_9567() {
    assert_eq!(try_pad_delay(9567).unwrap(), 115);
}

#[test]
#[should_panic]
fn safe_guard_try_pad_delay() {
    let _ = try_pad_delay(u32::MAX - 1);
}
