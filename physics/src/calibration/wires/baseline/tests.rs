use super::*;
use alpha_g_detector::alpha16::{aw_map::TPC_ANODE_WIRES, ADC_MAX, ADC_MIN};

fn all_within_limits(run_number: u32) -> bool {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();

        let baseline = try_wire_baseline(run_number, wire).unwrap();
        if let Some(baseline) = baseline {
            if baseline < ADC_MIN || baseline > ADC_MAX {
                return false;
            }
        }
    }
    true
}

#[test]
fn all_within_limits_in_baseline_map() {
    assert!(all_within_limits(7026));
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
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(0).unwrap()).unwrap(),
        Some(2981)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(32).unwrap()).unwrap(),
        Some(3036)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(64).unwrap()).unwrap(),
        Some(2907)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(96).unwrap()).unwrap(),
        Some(2866)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(128).unwrap()).unwrap(),
        Some(3079)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(160).unwrap()).unwrap(),
        Some(3024)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(192).unwrap()).unwrap(),
        Some(3002)
    );
    assert_eq!(
        try_wire_baseline(7026, TpcWirePosition::try_from(224).unwrap()).unwrap(),
        Some(2966)
    );
}
