use super::*;
use alpha_g_detector::padwing::{
    map::{TpcPadColumn, TpcPadRow, TPC_PAD_COLUMNS, TPC_PAD_ROWS},
    PWB_MAX, PWB_MIN,
};

fn all_within_limits(run_number: u32) -> bool {
    for row in 0..TPC_PAD_ROWS {
        let row = TpcPadRow::try_from(row).unwrap();
        for column in 0..TPC_PAD_COLUMNS {
            let column = TpcPadColumn::try_from(column).unwrap();
            let pad_position = TpcPadPosition { row, column };

            let baseline = try_pad_baseline(run_number, pad_position);
            if let Ok(baseline) = baseline {
                if !(PWB_MIN..=PWB_MAX).contains(&baseline) {
                    return false;
                }
            }
        }
    }
    true
}

#[test]
fn all_within_limits_in_baseline_map() {
    assert!(all_within_limits(9277));
    assert!(all_within_limits(u32::MAX));
}

#[test]
fn try_pad_baseline_map_error() {
    // Too slow to run all run numbers on CI. Just check limits.
    for run_number in [0, 9276] {
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            for column in 0..TPC_PAD_COLUMNS {
                let column = TpcPadColumn::try_from(column).unwrap();
                let pad_position = TpcPadPosition { row, column };

                assert!(try_pad_baseline(run_number, pad_position).is_err());
            }
        }
    }
}

#[test]
fn try_pad_baseline_correctness_9277() {
    let mut missing = 0;
    for row in 0..TPC_PAD_ROWS {
        let row = TpcPadRow::try_from(row).unwrap();
        for column in 0..TPC_PAD_COLUMNS {
            let column = TpcPadColumn::try_from(column).unwrap();
            let pad_position = TpcPadPosition { row, column };

            let baseline = try_pad_baseline(9277, pad_position);
            if baseline.is_err() {
                missing += 1;
            }
        }
    }
    assert_eq!(missing, 721);

    assert_eq!(
        try_pad_baseline(
            9277,
            TpcPadPosition {
                row: TpcPadRow::try_from(0).unwrap(),
                column: TpcPadColumn::try_from(0).unwrap(),
            }
        )
        .unwrap(),
        1665
    );
    assert_eq!(
        try_pad_baseline(
            9277,
            TpcPadPosition {
                row: TpcPadRow::try_from(429).unwrap(),
                column: TpcPadColumn::try_from(14).unwrap(),
            }
        )
        .unwrap(),
        1729
    );
    assert_eq!(
        try_pad_baseline(
            9277,
            TpcPadPosition {
                row: TpcPadRow::try_from(200).unwrap(),
                column: TpcPadColumn::try_from(5).unwrap(),
            }
        )
        .unwrap(),
        1714
    );
}

#[test]
fn try_pad_baseline_correctness_sim() {
    for column in 0..TPC_PAD_COLUMNS {
        let column = TpcPadColumn::try_from(column).unwrap();
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            let pad_position = TpcPadPosition { row, column };

            let baseline = try_pad_baseline(u32::MAX, pad_position).unwrap();
            assert_eq!(baseline, 1725);
        }
    }
}

#[test]
#[should_panic]
fn safe_guard_try_pad_baseline() {
    let column = TpcPadColumn::try_from(0).unwrap();
    let row = TpcPadRow::try_from(0).unwrap();
    let pad_position = TpcPadPosition { row, column };

    let _ = try_pad_baseline(u32::MAX - 1, pad_position);
}
