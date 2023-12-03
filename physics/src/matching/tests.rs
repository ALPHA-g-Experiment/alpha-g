use super::*;
use alpha_g_detector::padwing::map::TpcPadColumn;
use std::collections::HashSet;
use std::f64::consts::PI;

#[test]
fn wire_index_to_pad_column_index() {
    for wire_index in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(wire_index).unwrap();

        let closest_pad_column = (0..TPC_PAD_COLUMNS)
            .min_by_key(|&index| {
                let column = TpcPadColumn::try_from(index).unwrap();

                let angle = (wire.phi() - column.phi()).abs();
                let angle = if angle > PI { 2.0 * PI - angle } else { angle };

                (angle * 1000000.0) as usize
            })
            .unwrap();

        assert_eq!(closest_pad_column, wire_to_pad_column(wire_index));
    }
}

#[test]
fn pad_column_index_to_wire_indices() {
    let mut seen = HashSet::new();
    for pad_column_index in 0..TPC_PAD_COLUMNS {
        let wire_indices = pad_column_to_wires(pad_column_index);

        for wire_index in wire_indices {
            seen.insert(wire_index);
            assert_eq!(pad_column_index, wire_to_pad_column(wire_index));
        }
    }

    assert_eq!(seen.len(), TPC_ANODE_WIRES);
}
