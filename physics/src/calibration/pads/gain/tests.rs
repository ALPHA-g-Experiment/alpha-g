use super::*;
use alpha_g_detector::padwing::map::{TpcPadColumn, TpcPadRow, TPC_PAD_COLUMNS, TPC_PAD_ROWS};

#[test]
fn try_pad_gain_pad_gain_map_error() {
    // Too slow to run all run numbers on CI. Just test limits.
    for run_number in [0, 9276] {
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            for column in 0..TPC_PAD_COLUMNS {
                let column = TpcPadColumn::try_from(column).unwrap();
                let pad_position = TpcPadPosition { row, column };

                assert!(try_pad_gain(run_number, pad_position).is_err());
            }
        }
    }
}

#[test]
fn try_pad_gain_correctness_9277() {
    let mut missing = 0;
    for row in 0..TPC_PAD_ROWS {
        let row = TpcPadRow::try_from(row).unwrap();
        for column in 0..TPC_PAD_COLUMNS {
            let column = TpcPadColumn::try_from(column).unwrap();
            let pad_position = TpcPadPosition { row, column };

            let gain = try_pad_gain(9277, pad_position);
            if gain.is_err() {
                missing += 1;
            }
        }
    }
    assert_eq!(missing, 721);

    assert_eq!(
        try_pad_gain(
            9277,
            TpcPadPosition {
                column: TpcPadColumn::try_from(28).unwrap(),
                row: TpcPadRow::try_from(15).unwrap(),
            }
        )
        .unwrap(),
        1.7449521618659345
    );
    assert_eq!(
        try_pad_gain(
            9277,
            TpcPadPosition {
                column: TpcPadColumn::try_from(5).unwrap(),
                row: TpcPadRow::try_from(237).unwrap(),
            }
        )
        .unwrap(),
        1.0920076097769569
    );
    assert_eq!(
        try_pad_gain(
            9277,
            TpcPadPosition {
                column: TpcPadColumn::try_from(2).unwrap(),
                row: TpcPadRow::try_from(30).unwrap(),
            }
        )
        .unwrap(),
        1.4466143195282601
    );
}

#[test]
fn try_pad_gain_correctness_sim() {
    for column in 0..TPC_PAD_COLUMNS {
        let column = TpcPadColumn::try_from(column).unwrap();
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            let pad_position = TpcPadPosition { row, column };

            let gain = try_pad_gain(u32::MAX, pad_position).unwrap();
            assert_eq!(gain, 1.0);
        }
    }
}

#[test]
#[should_panic]
fn safe_guard_try_pad_gain() {
    let column = TpcPadColumn::try_from(0).unwrap();
    let row = TpcPadRow::try_from(0).unwrap();
    let pad_position = TpcPadPosition { row, column };

    let _ = try_pad_gain(u32::MAX - 1, pad_position);
}
