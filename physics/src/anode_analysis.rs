// This internal module contains all the analysis of the anode wire signals.
// Our internal representation of the anode wires is:
// [Option<Vec<f64>>; TPC_ANODE_WIRES] where an empty channel is `None`.
// The index of a Vector in the array `i` maps to a wire position in the
// detector as `TpcWirePosition::try_from(i).unwrap()`.
// IMPORTANT to keep in mind that index 0 does not correspond to `phi = 0`.

use alpha_g_detector::alpha16::{aw_map::TPC_ANODE_WIRES, ADC32_RATE};
use lazy_static::lazy_static;

// Width in nanoseconds of each wire signal bin.
const BIN_WIDTH: usize = (1.0e9 / ADC32_RATE) as usize;
const RESPONSE_BYTES: &[u8] = include_bytes!("../data/simulation/tpc_response/wires.json");
lazy_static! {
    // The format of the file is a serialized vector with the response every
    // nanosecond. Need to re-bin (by accumulating) by `BIN_WIDTH`.
    static ref WIRE_RESPONSE: Vec<f64> = {
        let raw: Vec<f64> = serde_json::from_slice(RESPONSE_BYTES).unwrap();

        let mut rebinned = Vec::with_capacity(raw.len() / BIN_WIDTH);
        for i in 0..raw.len() / BIN_WIDTH {
            let mut sum = 0.0;
            for j in 0..BIN_WIDTH {
                sum += raw[i * BIN_WIDTH + j];
            }
            rebinned.push(sum);
        }

        rebinned
    };
}

// Identify all the contiguous `Some` signals.
// Return, in an arbitrary order, a vector with the (half-open) intervals of the
// first (inclusive) and last (exclusive) indices in each contiguous block
// i.e. [first, last).
//
// Note that the detector is a cylinder, so the last signal is contiguous with
// the first one. In this case, a block could be e.g. (220, 5) i.e. from wire
// 220 to wire 4. For this reason I return [first, last) instead of a Range.
//
// Each of these blocks can be treated independently e.g. analyzed in parallel,
// etc.
fn find_contiguous(wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let mut in_range = false;
    for (i, signal) in wire_signals.iter().enumerate() {
        if signal.is_some() {
            if !in_range {
                start = i;
                in_range = true;
            }
            end = i + 1;
        } else if in_range {
            ranges.push((start, end));
            in_range = false;
        }
    }
    if in_range {
        ranges.push((start, end));
    }
    // The array is actually a ring, hence the last signal can be contiguous
    // with the first one.
    // Merge these into a single block.
    if ranges.len() > 1 {
        if let Some((0, _)) = ranges.first() {
            if let Some((_, TPC_ANODE_WIRES)) = ranges.last() {
                let (start_f, _) = ranges.pop().unwrap();
                let (_, end_i) = ranges.swap_remove(0);
                ranges.push((start_f, end_i));
            }
        }
    }

    ranges
}
