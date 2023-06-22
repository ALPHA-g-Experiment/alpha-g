// This internal module contains all the analysis of the anode wire signals.
// Our internal representation of the anode wires is:
// [Option<Vec<f64>>; TPC_ANODE_WIRES] where an empty channel is `None`.
// The index of a Vector in the array `i` maps to a wire position in the
// detector as `TpcWirePosition::try_from(i).unwrap()`.
// IMPORTANT to keep in mind that index 0 does not correspond to `phi = 0`.

use alpha_g_detector::alpha16::{aw_map::TPC_ANODE_WIRES, ADC32_RATE};
use dyn_stack::ReborrowMut;
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
fn find_contiguous_intervals(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
) -> Vec<(usize, usize)> {
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

// I can express my problem as the matrix equation:
// Y = R * X * A
// where:
// - Y is the observed signals.
// - R is the response matrix.
// - X is the unknown/wanted avalanches.
// - A is an induction coefficients matrix.
//
// I just need to create all the matrices and solve for X.

// Create the R matrix for a given size.
fn r_matrix(n: usize) -> faer_core::Mat<f64> {
    faer_core::Mat::with_dims(n, n, |i, j| {
        if i >= j {
            let diff = i - j;
            if diff < WIRE_RESPONSE.len() {
                WIRE_RESPONSE[diff]
            } else {
                0.0
            }
        } else {
            0.0
        }
    })
}

// Create the A matrix for a given size.
fn a_matrix(n: usize) -> faer_core::Mat<f64> {
    faer_core::Mat::with_dims(n, n, |i, j| {
        let diff = if i > j { i - j } else { j - i };
        if diff < NEIGHBOR_FACTORS.len() {
            NEIGHBOR_FACTORS[diff]
        } else {
            0.0
        }
    })
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

// Find the dimensions of our problem given the range of contiguous signals.
// The first index is the size of the largest signal in the block.
// The second index is the number of signals in the block.
fn problem_dimensions(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> (usize, usize) {
    let mut max_signal_len = 0;
    for i in range_to_indices(range) {
        // It is guaranteed that the signal is `Some`. Safe to unwrap.
        let signal = wire_signals[i].as_ref().unwrap();
        if signal.len() > max_signal_len {
            max_signal_len = signal.len();
        }
    }

    (max_signal_len, range_to_len(range))
}

// Create the Y matrix for a given range of signals.
fn y_matrix(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> faer_core::Mat<f64> {
    let (i, j) = problem_dimensions(wire_signals, range);

    faer_core::Mat::with_dims(i, j, |i, j| {
        let wire = range_to_indices(range).nth(j).unwrap();
        let signal = wire_signals[wire].as_ref().unwrap();
        if i < signal.len() {
            signal[i]
        } else {
            0.0
        }
    })
}

// Solve for X given a range of signals.
// Y = R * X * A
fn solve_x(
    wire_signals: &[Option<Vec<f64>>; TPC_ANODE_WIRES],
    range: (usize, usize),
) -> faer_core::Mat<f64> {
    let (i, j) = problem_dimensions(wire_signals, range);
    let r = r_matrix(i);
    let mut a = a_matrix(j);
    let mut y = y_matrix(wire_signals, range);

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

    faer_core::solve::solve_lower_triangular_in_place_with_conj(
        r.as_ref(),
        faer_core::Conj::No,
        y.as_mut(),
        faer_core::Parallelism::None,
    );

    y
}
