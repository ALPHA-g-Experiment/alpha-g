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

// There are too many pad signals to treat them in the same way as the anode
// wires (too slow). We can take advantage of the fact that the response is
// front-heavy, hence we can do a greedy deconvolution by just looking at the
// few samples ahead of each time bin.
// Given the details of our use-case add a non-negative constraint on the
// deconvolved input (same as the anode wires).
fn nn_greedy_deconvolution(
    signal: &[f64],
    response: &[f64],
    offset: usize,
    look_ahead: usize,
    // Return the sum of residuals squared together with the reconstructed input
) -> (f64, Vec<f64>) {
    let mut residual = signal.to_vec();
    let mut input = vec![0.0; signal.len()];

    for i in 0..(residual.len() - offset - look_ahead) {
        let val = residual[i + offset..][..look_ahead]
            .iter()
            .zip(response[offset..][..look_ahead].iter())
            .map(|(s, r)| s / r)
            .reduce(f64::min)
            .unwrap();

        if val > 0.0 {
            input[i] = val;
            residual[i..]
                .iter_mut()
                .zip(response)
                .for_each(|(s, r)| *s -= val * r);
        }
    }
    let residual = residual.iter().map(|x| x.powi(2)).sum();

    (residual, input)
}
