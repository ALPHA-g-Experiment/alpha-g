use super::*;
use alpha_g_detector::padwing::map::{TpcPadColumn, TpcPadRow, TPC_PAD_COLUMNS, TPC_PAD_ROWS};

#[test]
fn try_is_pad_masked_error() {
    let run_number = 0;

    for row in 0..TPC_PAD_ROWS {
        let row = TpcPadRow::try_from(row).unwrap();
        for column in 0..TPC_PAD_COLUMNS {
            let column = TpcPadColumn::try_from(column).unwrap();
            let pad_position = TpcPadPosition { row, column };

            assert!(try_is_pad_masked(run_number, pad_position).is_err());
        }
    }
}

#[test]
fn try_is_pad_masked_12405() {
    assert!(try_is_pad_masked(
        12405,
        TpcPadPosition {
            column: TpcPadColumn::try_from(31).unwrap(),
            row: TpcPadRow::try_from(25).unwrap(),
        }
    )
    .unwrap());

    assert!(try_is_pad_masked(
        12405,
        TpcPadPosition {
            column: TpcPadColumn::try_from(27).unwrap(),
            row: TpcPadRow::try_from(359).unwrap(),
        }
    )
    .unwrap());
}

#[test]
fn try_is_pad_masked_11084() {
    assert!(try_is_pad_masked(
        11084,
        TpcPadPosition {
            column: TpcPadColumn::try_from(31).unwrap(),
            row: TpcPadRow::try_from(25).unwrap(),
        }
    )
    .unwrap());

    assert!(!try_is_pad_masked(
        11084,
        TpcPadPosition {
            column: TpcPadColumn::try_from(27).unwrap(),
            row: TpcPadRow::try_from(359).unwrap(),
        }
    )
    .unwrap());
}

#[test]
fn try_is_pad_masked_9277() {
    assert!(try_is_pad_masked(
        9277,
        TpcPadPosition {
            column: TpcPadColumn::try_from(27).unwrap(),
            row: TpcPadRow::try_from(359).unwrap(),
        }
    )
    .unwrap());

    assert!(!try_is_pad_masked(
        9277,
        TpcPadPosition {
            column: TpcPadColumn::try_from(0).unwrap(),
            row: TpcPadRow::try_from(0).unwrap(),
        }
    )
    .unwrap());
}

#[test]
fn try_is_pad_masked_sim() {
    for column in 0..TPC_PAD_COLUMNS {
        let column = TpcPadColumn::try_from(column).unwrap();
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            let pad_position = TpcPadPosition { row, column };

            assert!(!try_is_pad_masked(u32::MAX, pad_position).unwrap());
        }
    }
}
