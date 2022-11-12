use super::*;

#[test]
fn histogram_single_short_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(1, 1);
    assert_eq!(hist.data, [1.0]);
}

#[test]
fn histogram_multiple_short_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(31250000, 1);
    hist.update(31249999, 1);
    assert_eq!(hist.data, [2.0]);

    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(31250000, 1);
    hist.update(31250000, 1);
    hist.update(31250000, 1);
    assert_eq!(hist.data, [2.0, 1.0]);
}

#[test]
fn histogram_single_long_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(93750000, 1);
    assert_eq!(hist.data, [2.0 / 3.0, 1.0 / 3.0]);
}

#[test]
fn histogram_multiple_long_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(93750000, 1);
    hist.update(93750000, 1);
    hist.update(93750000, 1);
    assert_eq!(
        hist.data,
        [2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 1.0 / 3.0]
    );
}

#[test]
fn histogram_single_very_long_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(218750000, 1);
    assert_eq!(hist.data, [2.0 / 7.0, 2.0 / 7.0, 2.0 / 7.0, 1.0 / 7.0]);
}

#[test]
fn histogram_multiple_very_long_update() {
    let mut hist = Histogram::new(0, 1.0, HistoStyle::Output);
    hist.update(218750000, 1);
    hist.update(218750000, 1);
    assert_eq!(
        hist.data,
        [
            2.0 / 7.0,
            2.0 / 7.0,
            2.0 / 7.0,
            2.0 / 7.0,
            2.0 / 7.0,
            2.0 / 7.0,
            2.0 / 7.0,
            0.0
        ]
    );
}
