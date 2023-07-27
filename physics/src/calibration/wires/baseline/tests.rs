use super::*;
use alpha_g_detector::alpha16::{aw_map::TPC_ANODE_WIRES, ADC_MAX, ADC_MIN};

fn all_within_limits(run_number: u32) -> bool {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();

        let baseline = try_wire_baseline(run_number, wire);
        if let Ok(baseline) = baseline {
            if !(ADC_MIN..=ADC_MAX).contains(&baseline) {
                return false;
            }
        }
    }
    true
}

#[test]
fn all_within_limits_in_baseline_map() {
    assert!(all_within_limits(7026));
    assert!(all_within_limits(u32::MAX));
}

#[test]
fn try_wire_baseline_wire_baseline_map_error() {
    for run_number in 0..=7025 {
        for i in 0..TPC_ANODE_WIRES {
            let wire = TpcWirePosition::try_from(i).unwrap();
            assert!(try_wire_baseline(run_number, wire).is_err());
        }
    }
}

#[test]
fn try_wire_baseline_correctness_7026() {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();
        let baseline = try_wire_baseline(7026, wire);
        assert!(baseline.is_ok());
    }

    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(0).unwrap()).unwrap(),
        2981
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(32).unwrap()).unwrap(),
        3036
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(64).unwrap()).unwrap(),
        2907
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(96).unwrap()).unwrap(),
        2866
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(128).unwrap()).unwrap(),
        3079
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(160).unwrap()).unwrap(),
        3024
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(192).unwrap()).unwrap(),
        3002
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(224).unwrap()).unwrap(),
        2966
    );
}

#[test]
fn try_wire_baseline_correctness_sim() {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();
        let baseline = try_wire_baseline(u32::MAX, wire).unwrap();
        assert_eq!(baseline, 3000);
    }
}

#[test]
#[should_panic]
fn safe_guard_try_wire_baseline() {
    let wire = TpcWirePosition::try_from(0).unwrap();
    let _ = try_wire_baseline(u32::MAX - 1, wire);
}
