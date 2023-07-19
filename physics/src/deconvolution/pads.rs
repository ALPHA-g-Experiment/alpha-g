// Internal representation of the pad signals is:
// [[Option<Vec<f64>>; TPC_PAD_ROWS]; TPC_PAD_COLUMNS] where an empty channel is
// `None`.

use crate::deconvolution::ls_deconvolution;
use alpha_g_detector::padwing::{
    map::{TPC_PAD_COLUMNS, TPC_PAD_ROWS},
    PWB_RATE,
};
use lazy_static::lazy_static;

// Width in nanoseconds of each pad signal bin.
const BIN_WIDTH: usize = (1.0e9 / PWB_RATE) as usize;

const RESPONSE_BYTES: &[u8] = include_bytes!("../../data/simulation/tpc_response/pads.json");

lazy_static! {
    // The format of the file is a serialized vector with the response every
    // nanosecond. Need to re-bin (by accumulating) by `BIN_WIDTH`.
    static ref PAD_RESPONSE: Vec<f64> = {
        let raw: Vec<f64> = serde_json::from_slice(RESPONSE_BYTES).unwrap();

        raw
            .chunks_exact(BIN_WIDTH)
            // Pad signals are inverted. So we need to flip the sign of the
            // response.
            .map(|chunk| -chunk.iter().sum::<f64>())
            .collect()
    };
}

// The row and column are assumed to be in the correct range.
// Furthermore, the signal is assumed to be `Some`.
pub(crate) fn pad_deconvolution(
    pad_signals: &[[Option<Vec<f64>>; TPC_PAD_ROWS]; TPC_PAD_COLUMNS],
    row: usize,
    column: usize,
) -> Vec<f64> {
    let signal = &pad_signals[column][row].as_ref().unwrap();
    // A 2D histogram of `best_offset` and `best_look_ahead` is not as
    // concentrated for the pad signals as it is for the wires. But it is not
    // too bad either. The "problem" is that there are way more pad signals
    // than wire signals. So this range cuts a bit of the tails of the
    // `best_offset`s and `best_look_ahead`s.
    // Nonetheless this range still contains the great majority of the
    // distribution. Just a bit more of a compromise than for the wires.
    ls_deconvolution(signal, &PAD_RESPONSE, 3..=5, 7..=12)
}

#[cfg(test)]
mod tests;
