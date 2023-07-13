// Internal representation of the pad signals is:
// [[Option<Vec<f64>>; TPC_PAD_ROWS]; TPC_PAD_COLUMNS] where an empty channel is
// `None`.

use alpha_g_detector::padwing::{
    map::{TPC_PAD_COLUMNS, TPC_PAD_ROWS},
    PWB_RATE,
};
use lazy_static::lazy_static;

// Width in nanoseconds of each pad signal bin.
const BIN_WIDTH: usize = (1.0e9 / PWB_RATE) as usize;

const RESPONSE_BYTES: &[u8] = include_bytes!("../data/simulation/tpc_response/pads.json");
const SVD_BYTES: &[u8] = include_bytes!("../data/optimization/pad_analysis/largest_svd.json");

lazy_static! {
    // The format of the file is a serialized vector with the response every
    // nanosecond. Need to re-bin (by accumulating) by `BIN_WIDTH`.
    static ref PAD_RESPONSE: Vec<f64> = {
        let raw: Vec<f64> = serde_json::from_slice(RESPONSE_BYTES).unwrap();

        raw
            .chunks_exact(BIN_WIDTH)
            .map(|chunk| chunk.iter().sum())
            .collect()
    };
    // Opposed to the wire signals, all the pad signals are always 510 samples
    // long. I am just handling the `BIG_R_MATRIX` and the `LARGEST_SVD` in
    // exactly the same way as the wires just to be future proof in case that
    // this ever changes. It is also easier to follow if both are consistent.
    static ref BIG_R_MATRIX: faer_core::Mat<f64> = {
        const MAX_SAMPLES: usize = 510;
        faer_core::Mat::with_dims(MAX_SAMPLES, MAX_SAMPLES, |i, j| {
            if i >= j {
                let diff = i - j;
                if diff < PAD_RESPONSE.len() {
                    // Pad signals are inverted w.r.t simulated response.
                    -1.0 * PAD_RESPONSE[diff]
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
    };
    // LARGEST_SVD[i] corresponds to the largest singular value of the submatrix
    // with size i x i.
    static ref LARGEST_SVD: Vec<f64> = serde_json::from_slice(SVD_BYTES).unwrap();
}
