use super::*;
use alpha_g_detector::alpha16::aw_map::TPC_ANODE_WIRES;

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

#[test]
fn try_wire_gain_correctness_sim() {
    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();
        let gain = try_wire_gain(u32::MAX, wire).unwrap();
        assert_eq!(gain, 1.0);
    }
}
