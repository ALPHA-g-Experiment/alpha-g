use super::*;
use crate::padwing::BoardId;
use crate::padwing::PADWING_BOARDS;

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
            match tpc_pwb_position(i, board_id) {
                Err(MapTpcPwbPositionError::MissingMap { run_number }) => assert_eq!(run_number, i),
                _ => unreachable!(),
            }
        }
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
                    tpc_pwb_position(run_number, BoardId::try_from(*board).unwrap()).unwrap(),
                    position
                );
            }
        }
    }
}
