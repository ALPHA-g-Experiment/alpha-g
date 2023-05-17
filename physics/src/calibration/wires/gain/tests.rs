use super::*;
use alpha_g_detector::alpha16::aw_map::TPC_ANODE_WIRES;

fn all_within_limits(run_number: u32) -> bool {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();

        let gain = try_wire_gain(run_number, wire);
        if let Ok(gain) = gain {
            // This 1.5 is somewhat arbitrary. Have a look at the
            // ARBITRARY_LARGE_SUPPRESSION_THRESHOLD value in `main.rs` of the
            // `alpha-g-wire-gain-calibration` executable.
            // If this test fails then there are a couple of things to do:
            // 1. Check which wire is causing this to fail and look at some
            //   waveforms. Mask the wire i.e. remove it from the calibration
            //   file if it is strange.
            // 2. If there is no need to mask this wire, check the pivot wire.
            //   If the pivot wire is strange then run the calibration again
            //   with this wire masked.
            //   Recall that the pivot will be the one with exactly 1.0 gain.
            // 3. If everything looks fine, then increase the
            //   ARBITRARY_LARGE_SUPPRESSION_THRESHOLD value in `main.rs` of the
            //   `alpha-g-wire-gain-calibration` executable.
            //   Also increase this 1.5 value here.
            if gain > 1.5 {
                return false;
            }
        }
    }
    true
}

#[test]
fn all_within_limits_in_gain_map() {
    assert!(all_within_limits(9277));
}

#[test]
fn try_wire_gain_wire_gain_map_error() {
    for run_number in 0..=9276 {
        for i in 0..TPC_ANODE_WIRES {
            let wire = TpcWirePosition::try_from(i).unwrap();
            assert!(try_wire_gain(run_number, wire).is_err());
        }
    }
}

#[test]
fn try_wire_gain_correctness_9277() {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();
        let gain = try_wire_gain(9277, wire);
        assert!(gain.is_ok());
    }

    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(0).unwrap()).unwrap(),
        1.1678439324476453
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(32).unwrap()).unwrap(),
        1.0491624717198518
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(64).unwrap()).unwrap(),
        1.0873347124407264
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(96).unwrap()).unwrap(),
        1.290917387251699
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(128).unwrap()).unwrap(),
        1.3295441389947555
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(160).unwrap()).unwrap(),
        1.2381314918699662
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(192).unwrap()).unwrap(),
        1.0467826211111837
    );
    assert_eq!(
        try_wire_gain(9277, TpcWirePosition::try_from(224).unwrap()).unwrap(),
        1.1702237830563136
    );
}
