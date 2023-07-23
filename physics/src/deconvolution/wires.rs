// Our internal representation of the anode wires is:
// [Option<Vec<f64>>; TPC_ANODE_WIRES] where an empty channel is `None`.

use crate::deconvolution::ls_deconvolution;
use alpha_g_detector::alpha16::{aw_map::TPC_ANODE_WIRES, ADC32_RATE};
use dyn_stack::ReborrowMut;
use lazy_static::lazy_static;

// Width in nanoseconds of each wire signal bin.
const BIN_WIDTH: usize = (1.0e9 / ADC32_RATE) as usize;

const RESPONSE_BYTES: &[u8] = include_bytes!("../../data/simulation/tpc_response/wires.json");

lazy_static! {
    // The format of the file is a serialized vector with the response every
    // nanosecond. Need to re-bin (by accumulating) by `BIN_WIDTH`.
    static ref WIRE_RESPONSE: Vec<f64> = {
        let raw: Vec<f64> = serde_json::from_slice(RESPONSE_BYTES).unwrap();

        raw
            .chunks_exact(BIN_WIDTH)
            .map(|chunk| chunk.iter().sum())
            .collect()
    };
}
// "Strength" of the signal induced on a neighboring wire.
const NEIGHBOR_FACTORS: [f64; 5] = [1.0, -0.1275, -0.0365, -0.012, -0.0042];

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
pub(crate) fn contiguous_ranges(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut end = 0;
    while start < TPC_ANODE_WIRES {
        while end < TPC_ANODE_WIRES && wire_signals[end].is_some() {
            end += 1;
        }
        if start < end {
            ranges.push((start, end));
        }
        start = end + 1;
        end = start;
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

// The range should be one of the ones returned by `contiguous_ranges`.
// Basically this assumes:
//     1. The range is not empty.
//     2. All signals in the range [first, last) are `Some`.
//
//  The returned vector has the same length as the input range. Vector `0`
//  corresponds to `first`, element `1` to `first.next()`, etc.
pub(crate) fn wire_range_deconvolution(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> Vec<(usize, Vec<f64>)> {
    let (i, j) = problem_dimensions(wire_signals, range);
    let mut a = a_matrix(j);
    let mut y = y_matrix(wire_signals, range);
    // I can remove the cross-talk between channels as:
    // Y * A^-1 = Y' = R * X
    let mut mem = dyn_stack::GlobalMemBuffer::new(
        faer_cholesky::llt::compute::cholesky_in_place_req::<f64>(
            j,
            faer_core::Parallelism::None,
            Default::default(),
        )
        .unwrap(),
    );
    let mut stack = dyn_stack::DynStack::new(&mut mem);
    faer_cholesky::llt::compute::cholesky_in_place(
        a.as_mut(),
        faer_core::Parallelism::None,
        stack.rb_mut(),
        Default::default(),
    )
    .unwrap();

    let mut mem = dyn_stack::GlobalMemBuffer::new(
        faer_cholesky::llt::solve::solve_transpose_in_place_req::<f64>(
            j,
            i,
            faer_core::Parallelism::None,
        )
        .unwrap(),
    );
    let mut stack = dyn_stack::DynStack::new(&mut mem);
    faer_cholesky::llt::solve::solve_transpose_in_place_with_conj(
        a.as_ref(),
        faer_core::Conj::No,
        y.as_mut().transpose(),
        faer_core::Parallelism::None,
        stack.rb_mut(),
    );
    // At this point I have the system:
    // Y' = R * X
    // Each channel is just an independent system.
    let mut sol = Vec::with_capacity(j);
    for column in 0..j {
        let signal = (0..i).map(|row| y.read(row, column)).collect::<Vec<_>>();
        // The best `offset` and `look_ahead` are highly concentrated in the
        // following ranges chosen. To reproduce just make a 2D histogram; there
        // is barely anything outside these ranges.
        sol.push(ls_deconvolution(&signal, &WIRE_RESPONSE, 0..=1, 3..=12));
    }

    range_to_indices(range).zip(sol).collect()
}

// Given a time bin `t`, remove the noise of all channels from `t` onwards.
// For each channel, the noise threshold is the maximum value in the range
// [0, t).
pub(crate) fn remove_noise_after_t(wire_inputs: &mut [Vec<f64>; TPC_ANODE_WIRES], t: usize) {
    for input in wire_inputs {
        let noise_threshold = input.iter().take(t).copied().fold(0.0, f64::max);
        for value in input.iter_mut().skip(t) {
            if *value <= noise_threshold {
                *value = 0.0;
            }
        }
    }
}

// Given a range [first, last), return an iterator over the indices.
fn range_to_indices(range: (usize, usize)) -> Box<dyn Iterator<Item = usize>> {
    let (first, last) = range;
    if first < last {
        Box::new(first..last)
    } else {
        Box::new((first..TPC_ANODE_WIRES).chain(0..last))
    }
}

// Given a range [first, last), return the number of wires in the range.
fn range_to_len(range: (usize, usize)) -> usize {
    let (first, last) = range;
    if first < last {
        last - first
    } else {
        TPC_ANODE_WIRES - first + last
    }
}

// I can express a set of wire signals as the matrix equation:
// Y = R * X * A
// where:
// - Y is the observed signals.
// - R is the response matrix.
// - X is the unknown/wanted avalanches.
// - A is an induction coefficients matrix.
//
// Each column of X (and Y) is a channel. Each row of X (and Y) is a time bin.
// I just need to create all the matrices and solve for X.

// Create the A matrix for a given size.
fn a_matrix(n: usize) -> faer_core::Mat<f64> {
    faer_core::Mat::with_dims(n, n, |i, j| {
        let diff = if i > j { i - j } else { j - i };
        NEIGHBOR_FACTORS.get(diff).copied().unwrap_or(0.0)
    })
}

// Find the dimensions of our problem given the range of contiguous signals.
// The first index is the size of the largest signal in the block.
// The second index is the number of signals in the block.
fn problem_dimensions(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> (usize, usize) {
    let max_len = range_to_indices(range)
        // Guaranteed to be Some. Safe to unwrap
        .map(|i| wire_signals[i].as_ref().unwrap().len())
        .max()
        // Guaranteed to not be empty. Safe to unwrap
        .unwrap();

    (max_len, range_to_len(range))
}

// Create the Y matrix for a given range of signals.
fn y_matrix(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> faer_core::Mat<f64> {
    let (i, j) = problem_dimensions(wire_signals, range);

    faer_core::Mat::with_dims(i, j, |i, j| {
        let wire = range_to_indices(range).nth(j).unwrap();
        wire_signals[wire]
            .as_ref()
            .unwrap()
            .get(i)
            .copied()
            .unwrap_or(0.0)
    })
}

#[cfg(test)]
mod tests;
