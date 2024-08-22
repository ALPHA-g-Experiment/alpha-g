use super::*;
use crate::padwing::BoardId;
use crate::padwing::PADWING_BOARDS;

#[test]
fn detector_length() {
    let detector_length = 2.304;
    let abs_diff = (detector_length - DETECTOR_LENGTH).abs();
    assert!(abs_diff < 1e-10);
}

#[test]
fn cathode_pads_radius() {
    let cathode_pads_radius = 0.19;
    let abs_diff = (cathode_pads_radius - CATHODE_PADS_RADIUS).abs();
    assert!(abs_diff < 1e-10);
}

#[test]
fn pwb_pad_columns() {
    assert_eq!(PWB_PAD_COLUMNS, 4);
}

#[test]
fn pwb_pad_rows() {
    assert_eq!(PWB_PAD_ROWS, 72);
}

#[test]
fn tpc_pwb_columns() {
    assert_eq!(TPC_PWB_COLUMNS, 8);
}

#[test]
fn tpc_pwb_rows() {
    assert_eq!(TPC_PWB_ROWS, 8);
}

#[test]
fn tpc_pad_columns() {
    assert_eq!(TPC_PAD_COLUMNS, 32);
}

#[test]
fn tpc_pad_rows() {
    assert_eq!(TPC_PAD_ROWS, 576);
}

#[test]
fn tpc_pads() {
    assert_eq!(TPC_PADS, 18432);
}

#[test]
fn pad_pitch_z() {
    let pad_pitch_z = 4e-3;
    let abs_diff = (pad_pitch_z - PAD_PITCH_Z).abs();
    assert!(abs_diff < 1e-10);
}

#[test]
fn pad_pitch_phi() {
    let pad_pitch_phi = 2.0 * std::f64::consts::PI / 32.0;
    let abs_diff = (pad_pitch_phi - PAD_PITCH_PHI).abs();
    assert!(abs_diff < 1e-10);
}

#[test]
fn try_from_index_tpc_pwb_column() {
    for i in 0..=7 {
        assert_eq!(TpcPwbColumn::try_from(i).unwrap(), TpcPwbColumn(i));
    }
    for i in 8..=19000 {
        assert!(TpcPwbColumn::try_from(i).is_err());
    }
}

#[test]
fn try_from_index_tpc_pwb_row() {
    for i in 0..=7 {
        assert_eq!(TpcPwbRow::try_from(i).unwrap(), TpcPwbRow(i));
    }
    for i in 8..=19000 {
        assert!(TpcPwbRow::try_from(i).is_err());
    }
}

#[test]
fn tpc_pwb_position_new() {
    for i in 0..=7 {
        for j in 0..=7 {
            assert_eq!(
                TpcPwbPosition::new(TpcPwbColumn(i), TpcPwbRow(j)),
                TpcPwbPosition {
                    column: TpcPwbColumn(i),
                    row: TpcPwbRow(j)
                }
            );
        }
    }
}

#[test]
fn tpc_pwb_position_column() {
    for i in 0..=7 {
        let position = TpcPwbPosition {
            column: TpcPwbColumn(i),
            row: TpcPwbRow(0),
        };
        assert_eq!(position.column(), TpcPwbColumn(i));
    }
}

#[test]
fn tpc_pwb_position_row() {
    for i in 0..=7 {
        let position = TpcPwbPosition {
            column: TpcPwbColumn(0),
            row: TpcPwbRow(i),
        };
        assert_eq!(position.row(), TpcPwbRow(i));
    }
}

fn all_different_str(map: [[&str; 8]; 8]) -> bool {
    for (i, row) in map.iter().enumerate() {
        for (j, board) in row.iter().enumerate() {
            for (k, row2) in map.iter().enumerate() {
                for (l, board2) in row2.iter().enumerate() {
                    if i == k && j == l {
                        continue;
                    }
                    if board == board2 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

#[test]
fn all_different_str_in_padwing_boards_maps() {
    assert!(all_different_str(PADWING_BOARDS_4418));
    assert!(all_different_str(PADWING_BOARDS_10418));
}

fn all_valid_str(map: [[&str; 8]; 8]) -> bool {
    for row in map.iter() {
        for name in row.iter() {
            if BoardId::try_from(*name).is_err() {
                return false;
            }
        }
    }
    true
}

#[test]
fn all_valid_str_in_padwing_boards_maps() {
    assert!(all_valid_str(PADWING_BOARDS_4418));
    assert!(all_valid_str(PADWING_BOARDS_10418));
}

// First index is column, second index is row.
// The value is the board name.
const REGRESSION_GATE_KEEPER_4418: [[&str; 8]; 8] = [
    ["12", "13", "14", "02", "11", "17", "18", "19"],
    ["20", "21", "22", "23", "24", "25", "26", "27"],
    ["46", "29", "08", "77", "10", "33", "34", "35"],
    ["36", "37", "01", "39", "76", "41", "42", "40"],
    ["44", "49", "07", "78", "03", "04", "45", "15"],
    ["52", "53", "54", "55", "56", "57", "58", "05"],
    ["60", "00", "06", "63", "64", "65", "66", "67"],
    ["68", "69", "70", "71", "72", "73", "74", "75"],
];

#[test]
fn tpc_pwb_position_missing_map() {
    for i in 0..4418 {
        for (name, _mac, _id) in PADWING_BOARDS {
            let board_id = BoardId::try_from(name).unwrap();
            match TpcPwbPosition::try_new(i, board_id) {
                Err(MapTpcPwbPositionError::MissingMap { run_number }) => assert_eq!(run_number, i),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn tpc_pwb_position_sim_correctness() {
    for name in PADWING_BOARDS_4418.iter().flatten() {
        let board_id = BoardId::try_from(*name).unwrap();

        let pos_5000 = TpcPwbPosition::try_new(5000, board_id).unwrap();
        let pos_sim = TpcPwbPosition::try_new(u32::MAX, board_id).unwrap();

        assert_eq!(pos_5000, pos_sim);
    }
}

#[test]
fn inverse_map_tpc_pwb_position_4418() {
    for run_number in 4418..=10000 {
        for (i, row) in REGRESSION_GATE_KEEPER_4418.iter().enumerate() {
            for (j, board) in row.iter().enumerate() {
                let position = TpcPwbPosition {
                    column: TpcPwbColumn(i),
                    row: TpcPwbRow(j),
                };
                assert_eq!(
                    TpcPwbPosition::try_new(run_number, BoardId::try_from(*board).unwrap())
                        .unwrap(),
                    position
                );
            }
        }
    }
}

#[test]
fn inverse_map_tpc_pwb_position_10418() {
    let mut count = 0;
    // new_board -> (column, row)
    let mut swapped = HashMap::new();
    swapped.insert("90", (2, 0));
    swapped.insert("85", (2, 3));
    swapped.insert("89", (4, 0));
    swapped.insert("87", (4, 3));
    swapped.insert("84", (4, 6));
    swapped.insert("91", (4, 7));
    swapped.insert("81", (5, 7));
    swapped.insert("44", (6, 2));

    for (name, _, _) in PADWING_BOARDS {
        println!("{name}");
        if let Some((col, row)) = swapped.get(name) {
            let position = TpcPwbPosition {
                column: TpcPwbColumn(*col),
                row: TpcPwbRow(*row),
            };
            assert_eq!(
                TpcPwbPosition::try_new(10418, BoardId::try_from(name).unwrap()).unwrap(),
                position
            );
            count += 1;
        } else if let Ok(position) =
            TpcPwbPosition::try_new(10418, BoardId::try_from(name).unwrap())
        {
            assert_eq!(
                position,
                TpcPwbPosition::try_new(4418, BoardId::try_from(name).unwrap()).unwrap()
            );
            count += 1;
        }
    }

    assert_eq!(count, 64);
}

#[test]
fn try_from_index_pwb_pad_column() {
    for i in 0..=3 {
        assert_eq!(PwbPadColumn::try_from(i).unwrap(), PwbPadColumn(i));
    }
    for i in 4..=19000 {
        assert!(PwbPadColumn::try_from(i).is_err());
    }
}

#[test]
fn try_from_index_pwb_pad_row() {
    for i in 0..=71 {
        assert_eq!(PwbPadRow::try_from(i).unwrap(), PwbPadRow(i));
    }
    for i in 72..=19000 {
        assert!(PwbPadRow::try_from(i).is_err());
    }
}

#[test]
fn pwb_pad_position_new() {
    for i in 0..=3 {
        for j in 0..=71 {
            assert_eq!(
                PwbPadPosition::new(PwbPadColumn(i), PwbPadRow(j)),
                PwbPadPosition {
                    column: PwbPadColumn(i),
                    row: PwbPadRow(j)
                }
            );
        }
    }
}

#[test]
fn pwb_pad_position_column() {
    for i in 0..=3 {
        let position = PwbPadPosition {
            column: PwbPadColumn(i),
            row: PwbPadRow(0),
        };
        assert_eq!(position.column(), PwbPadColumn(i));
    }
}

#[test]
fn pwb_pad_position_row() {
    for i in 0..=71 {
        let position = PwbPadPosition {
            column: PwbPadColumn(0),
            row: PwbPadRow(i),
        };
        assert_eq!(position.row(), PwbPadRow(i));
    }
}

#[test]
fn pwb_pad_position_all_exist() {
    for after in 'A'..='D' {
        for channel in 1..=72 {
            assert!(PwbPadPosition::try_new(
                0,
                AfterId::try_from(after).unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .is_ok());
        }
    }
}

#[test]
fn pwb_pad_position_correctness() {
    for (row, channel) in (19..=36).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('A').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(0),
                row: PwbPadRow(row),
            }
        );
    }
    for (row, channel) in (37..=54).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('A').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(0),
                row: PwbPadRow(row + 18),
            }
        );
    }
    for (row, channel) in (19..=36).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('B').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(0),
                row: PwbPadRow(row + 36),
            }
        );
    }
    for (row, channel) in (37..=54).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('B').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(0),
                row: PwbPadRow(row + 54),
            }
        );
    }
    for (row, channel) in (1..=18).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('A').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(1),
                row: PwbPadRow(row),
            }
        );
    }
    for (row, channel) in (55..=72).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('A').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(1),
                row: PwbPadRow(row + 18),
            }
        );
    }
    for (row, channel) in (1..=18).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('B').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(1),
                row: PwbPadRow(row + 36),
            }
        );
    }
    for (row, channel) in (55..=72).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('B').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(1),
                row: PwbPadRow(row + 54),
            }
        );
    }
    for (row, channel) in (55..=72).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('D').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(2),
                row: PwbPadRow(row),
            }
        );
    }
    for (row, channel) in (1..=18).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('D').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(2),
                row: PwbPadRow(row + 18),
            }
        );
    }
    for (row, channel) in (55..=72).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('C').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(2),
                row: PwbPadRow(row + 36),
            }
        );
    }
    for (row, channel) in (1..=18).rev().enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('C').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(2),
                row: PwbPadRow(row + 54),
            }
        );
    }
    for (row, channel) in (37..=54).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('D').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(3),
                row: PwbPadRow(row),
            }
        );
    }
    for (row, channel) in (19..=36).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('D').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(3),
                row: PwbPadRow(row + 18),
            }
        );
    }
    for (row, channel) in (37..=54).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('C').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(3),
                row: PwbPadRow(row + 36),
            }
        );
    }
    for (row, channel) in (19..=36).enumerate() {
        assert_eq!(
            PwbPadPosition::try_new(
                0,
                AfterId::try_from('C').unwrap(),
                PadChannelId::try_from(channel).unwrap()
            )
            .unwrap(),
            PwbPadPosition {
                column: PwbPadColumn(3),
                row: PwbPadRow(row + 54),
            }
        );
    }
}

#[test]
fn try_from_index_tpc_pad_column() {
    for i in 0..=31 {
        assert_eq!(TpcPadColumn::try_from(i).unwrap(), TpcPadColumn(i));
    }
    for i in 32..=19000 {
        assert!(TpcPadColumn::try_from(i).is_err());
    }
}

#[test]
fn from_tpc_pad_column_usize() {
    for i in 0..=31 {
        let pad_column = TpcPadColumn::try_from(i).unwrap();
        assert_eq!(usize::from(pad_column), i);
    }
}

#[test]
fn tpc_pad_column_ron_roundtrip() {
    for i in 0..=31 {
        let pad_column = TpcPadColumn::try_from(i).unwrap();
        let pad_column_ron = ron::to_string(&pad_column).unwrap();
        let pad_column_deserialized: TpcPadColumn = ron::from_str(&pad_column_ron).unwrap();
        assert_eq!(pad_column, pad_column_deserialized);
    }
}

#[test]
fn tpc_pad_column_phi() {
    for i in 0..=31 {
        let pad_column = TpcPadColumn::try_from(i).unwrap();
        assert_eq!(pad_column.phi(), (i as f64 + 0.5) * 2.0 * PI / 32.0);
    }
}

#[test]
fn try_from_index_tpc_pad_row() {
    for i in 0..=575 {
        assert_eq!(TpcPadRow::try_from(i).unwrap(), TpcPadRow(i));
    }
    for i in 576..=19000 {
        assert!(TpcPadRow::try_from(i).is_err());
    }
}

#[test]
fn from_tpc_pad_row_usize() {
    for i in 0..=575 {
        let pad_row = TpcPadRow::try_from(i).unwrap();
        assert_eq!(usize::from(pad_row), i);
    }
}

#[test]
fn tpc_pad_row_ron_roundtrip() {
    for i in 0..=575 {
        let pad_row = TpcPadRow::try_from(i).unwrap();
        let pad_row_ron = ron::to_string(&pad_row).unwrap();
        let pad_row_deserialized: TpcPadRow = ron::from_str(&pad_row_ron).unwrap();
        assert_eq!(pad_row, pad_row_deserialized);
    }
}

#[test]
fn tpc_pad_row_z() {
    let start = -0.5 * DETECTOR_LENGTH;
    for i in 0..=575 {
        let pad_row = TpcPadRow::try_from(i).unwrap();
        let z = start + (i as f64 + 0.5) * PAD_PITCH_Z;
        assert_eq!(pad_row.z(), z);
    }
}

#[test]
fn tpc_pad_position_new() {
    for column in 0..=31 {
        let board_column = TpcPwbColumn::try_from(column / 4).unwrap();
        let pad_column = PwbPadColumn::try_from(column % 4).unwrap();
        for row in 0..=575 {
            let board_row = TpcPwbRow::try_from(row / 72).unwrap();
            let pad_row = PwbPadRow::try_from(row % 72).unwrap();

            let board = TpcPwbPosition {
                column: board_column,
                row: board_row,
            };
            let pad = PwbPadPosition {
                column: pad_column,
                row: pad_row,
            };

            assert_eq!(
                TpcPadPosition::new(board, pad),
                TpcPadPosition {
                    column: TpcPadColumn(column),
                    row: TpcPadRow(row),
                }
            );
        }
    }
}

#[test]
fn tpc_pad_position_ron_roundtrip() {
    for row in 0..=575 {
        for column in 0..=31 {
            let pad_position = TpcPadPosition {
                column: TpcPadColumn(column),
                row: TpcPadRow(row),
            };
            let pad_position_ron = ron::to_string(&pad_position).unwrap();
            let pad_position_deserialized: TpcPadPosition =
                ron::from_str(&pad_position_ron).unwrap();
            assert_eq!(pad_position, pad_position_deserialized);
        }
    }
}

#[test]
fn tpc_pad_position_column() {
    for i in 0..=31 {
        let position = TpcPadPosition {
            column: TpcPadColumn(i),
            row: TpcPadRow(0),
        };
        assert_eq!(position.column, TpcPadColumn(i));
    }
}

#[test]
fn tpc_pad_position_row() {
    for i in 0..=575 {
        let position = TpcPadPosition {
            column: TpcPadColumn(0),
            row: TpcPadRow(i),
        };
        assert_eq!(position.row, TpcPadRow(i));
    }
}

#[test]
fn tpc_pad_position_bad_tpc_pwb_position() {
    for run_number in 0..=4417 {
        let board_id = BoardId::try_from("26").unwrap();
        let after_id = AfterId::try_from('A').unwrap();
        let channel_id = PadChannelId::try_from(1).unwrap();

        assert!(matches!(
            TpcPadPosition::try_new(run_number, board_id, after_id, channel_id),
            Err(MapTpcPadPositionError::BadTpcPwbPosition(_))
        ));
    }
}

#[test]
fn tpc_pad_position_try_new() {
    let run_number = 4418;
    for (column, row) in REGRESSION_GATE_KEEPER_4418.into_iter().enumerate() {
        for (row, name) in row.into_iter().enumerate() {
            let board_id = BoardId::try_from(name).unwrap();

            let bottom_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72),
            };
            let after_a = AfterId::try_from('A').unwrap();
            let channel_36 = PadChannelId::try_from(36).unwrap();
            assert_eq!(
                TpcPadPosition::try_new(run_number, board_id, after_a, channel_36).unwrap(),
                bottom_left_pad_position
            );

            let top_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72 + 71),
            };
            let after_b = AfterId::try_from('B').unwrap();
            let channel_37 = PadChannelId::try_from(37).unwrap();
            assert_eq!(
                TpcPadPosition::try_new(run_number, board_id, after_b, channel_37).unwrap(),
                top_left_pad_position
            );

            let bottom_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72),
            };
            let after_d = AfterId::try_from('D').unwrap();
            let channel_37 = PadChannelId::try_from(37).unwrap();
            assert_eq!(
                TpcPadPosition::try_new(run_number, board_id, after_d, channel_37).unwrap(),
                bottom_right_pad_position
            );

            let top_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72 + 71),
            };
            let after_c = AfterId::try_from('C').unwrap();
            let channel_36 = PadChannelId::try_from(36).unwrap();
            assert_eq!(
                TpcPadPosition::try_new(run_number, board_id, after_c, channel_36).unwrap(),
                top_right_pad_position
            );
        }
    }
}

#[test]
fn tpc_pad_position_z() {
    const DETECTOR_HALF_LENGTH: f64 = 0.5 * DETECTOR_LENGTH;
    for column in 0..=7 {
        for row in 0..=7 {
            let bottom_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72),
            };
            let z = (row as f64 * 72.0 + 0.5) * 4e-3 - DETECTOR_HALF_LENGTH;
            let abs_difference = (z - bottom_left_pad_position.z()).abs();
            assert!(abs_difference < 1e-10);

            let top_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72 + 71),
            };
            let z = (row as f64 * 72.0 + 71.5) * 4e-3 - DETECTOR_HALF_LENGTH;
            let abs_difference = (z - top_left_pad_position.z()).abs();
            assert!(abs_difference < 1e-10);

            let bottom_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72),
            };
            let z = (row as f64 * 72.0 + 0.5) * 4e-3 - DETECTOR_HALF_LENGTH;
            let abs_difference = (z - bottom_right_pad_position.z()).abs();
            assert!(abs_difference < 1e-10);

            let top_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72 + 71),
            };
            let z = (row as f64 * 72.0 + 71.5) * 4e-3 - DETECTOR_HALF_LENGTH;
            let abs_difference = (z - top_right_pad_position.z()).abs();
            assert!(abs_difference < 1e-10);
        }
    }
}

#[test]
fn tpc_pad_position_phi() {
    for column in 0..=7 {
        for row in 0..=7 {
            let bottom_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72),
            };
            let phi = (column as f64 * 4.0 + 0.5) * 2.0 * PI / 32.0;
            let abs_difference = (phi - bottom_left_pad_position.phi()).abs();
            assert!(abs_difference < 1e-10);

            let top_left_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4),
                row: TpcPadRow(row * 72 + 71),
            };
            let phi = (column as f64 * 4.0 + 0.5) * 2.0 * PI / 32.0;
            let abs_difference = (phi - top_left_pad_position.phi()).abs();
            assert!(abs_difference < 1e-10);

            let bottom_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72),
            };
            let phi = (column as f64 * 4.0 + 3.0 + 0.5) * 2.0 * PI / 32.0;
            let abs_difference = (phi - bottom_right_pad_position.phi()).abs();
            assert!(abs_difference < 1e-10);

            let top_right_pad_position = TpcPadPosition {
                column: TpcPadColumn(column * 4 + 3),
                row: TpcPadRow(row * 72 + 71),
            };
            let phi = (column as f64 * 4.0 + 3.0 + 0.5) * 2.0 * PI / 32.0;
            let abs_difference = (phi - top_right_pad_position.phi()).abs();
            assert!(abs_difference < 1e-10);
        }
    }
}
