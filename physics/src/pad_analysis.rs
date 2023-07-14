// Internal representation of the pad signals is:
// [[Option<Vec<f64>>; TPC_PAD_ROWS]; TPC_PAD_COLUMNS] where an empty channel is
// `None`.

use alpha_g_detector::padwing::PWB_RATE;
use lazy_static::lazy_static;

// Width in nanoseconds of each pad signal bin.
const BIN_WIDTH: usize = (1.0e9 / PWB_RATE) as usize;

const RESPONSE_BYTES: &[u8] = include_bytes!("../data/simulation/tpc_response/pads.json");

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
