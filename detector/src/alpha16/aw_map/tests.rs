use super::*;
use std::collections::HashSet;

#[test]
fn tpc_anode_wires() {
    let tpc_anode_wires = 256;
    assert_eq!(tpc_anode_wires, TPC_ANODE_WIRES);
}

#[test]
fn anode_wire_pitch_phi() {
    let anode_wire_pitch_phi = 2.0 * std::f64::consts::PI / 256.0;
    let abs_diff = (anode_wire_pitch_phi - ANODE_WIRE_PITCH_PHI).abs();
    assert!(abs_diff < 1e-10);
}

#[test]
fn try_from_index_tpc_wire_position() {
    for i in 0..=255 {
        let wire_position = TpcWirePosition::try_from(i).unwrap();
        assert_eq!(wire_position, TpcWirePosition(i));
    }
    for i in 256..=19000 {
        let wire_position = TpcWirePosition::try_from(i);
        assert!(wire_position.is_err());
    }
}

fn all_different_str(map: [(&str, (usize, usize)); 8]) -> bool {
    let mut set = HashSet::new();
    for (s, _) in map.iter() {
        if !set.insert(s) {
            return false;
        }
    }
    true
}

#[test]
fn all_different_str_in_preamps_map() {
    assert!(all_different_str(PREAMPS_2941));
}

fn all_valid_str(map: [(&str, (usize, usize)); 8]) -> bool {
    for (s, _) in map.iter() {
        if BoardId::try_from(*s).is_err() {
            return false;
        }
    }
    true
}

#[test]
fn all_valid_str_in_preamps_map() {
    assert!(all_valid_str(PREAMPS_2941));
}

fn all_valid_preamps(map: [(&str, (usize, usize)); 8]) -> bool {
    let mut set = HashSet::new();
    for (_, (p1, p2)) in map.iter() {
        if !set.insert(p1) || !set.insert(p2) {
            return false;
        }
        if *p1 > 15 || *p2 > 15 {
            return false;
        }
    }
    set.len() == 16
}

#[test]
fn all_valid_preamps_in_preamps_map() {
    assert!(all_valid_preamps(PREAMPS_2941));
}

fn all_valid_channels(map: [usize; 32]) -> bool {
    let mut set = HashSet::new();
    for c in map.iter() {
        if !set.insert(c) {
            return false;
        }
        if *c > 31 {
            return false;
        }
    }
    set.len() == 32
}

#[test]
fn all_valid_channels_in_inv_channels_map() {
    assert!(all_valid_channels(INV_CHANNELS_2724));
}

#[test]
fn tpc_wire_position_missing_preamp_map() {
    let board_id = BoardId::try_from("09").unwrap();
    let adc32_channel_id = Adc32ChannelId::try_from(0).unwrap();
    for i in 0..=2940 {
        match TpcWirePosition::try_new(i, board_id, adc32_channel_id) {
            Err(MapTpcWirePositionError::MissingPreampMap { run_number }) => {
                assert_eq!(run_number, i);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn tpc_wire_position_correctness_2941() {
    let run_number = 2941;
    let chan_map: [u8; 32] = [
        2, 8, 1, 9, 0, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15, 16, 24, 17, 25, 18, 26, 19, 27, 20,
        28, 21, 29, 22, 30, 23, 31,
    ];

    let board_id = BoardId::try_from("09").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire);
    }

    let board_id = BoardId::try_from("10").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 32);
    }

    let board_id = BoardId::try_from("11").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 64);
    }

    let board_id = BoardId::try_from("12").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 96);
    }

    let board_id = BoardId::try_from("13").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 128);
    }

    let board_id = BoardId::try_from("14").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 160);
    }

    let board_id = BoardId::try_from("18").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 192);
    }

    let board_id = BoardId::try_from("16").unwrap();
    for (wire, _) in chan_map.iter().enumerate() {
        let channel_id = Adc32ChannelId::try_from(chan_map[wire]).unwrap();
        let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id).unwrap();
        assert_eq!(wire_position.0, wire + 224);
    }
}

#[test]
fn tpc_wire_position_phi() {
    for i in 0..TPC_ANODE_WIRES {
        let wire_position = TpcWirePosition::try_from(i).unwrap();
        let phi = (i as f64 + 0.5) * 2.0 * std::f64::consts::PI / 256.0;
        let abs_diff = (wire_position.phi() - phi).abs();
        assert!(abs_diff < 1e-10);
    }
}
