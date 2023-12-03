use crate::Avalanche;
use alpha_g_detector::alpha16::aw_map::{TpcWirePosition, TPC_ANODE_WIRES};
use alpha_g_detector::alpha16::ADC32_RATE;
use alpha_g_detector::padwing::map::{TpcPadRow, PAD_PITCH_Z, TPC_PAD_COLUMNS, TPC_PAD_ROWS};
use std::ops::Range;
use uom::si::angle::radian;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;
use uom::typenum::P2;

const WIRES_PER_COLUMN: usize = TPC_ANODE_WIRES / TPC_PAD_COLUMNS;
// From `alpha_g_detector` internal mapping.
const WIRE_SHIFT: usize = 8;

// Map a wire index to the pad column index that contains it.
//
// In our internal representation, the wire number is an index in the range
// [0, 255].
// The pad column is another index in the range [0, 31].
pub(crate) fn wire_to_pad_column(wire: usize) -> usize {
    // Wire 0 doesn't align with pad column 0. See the documentation of
    // `alpha_g_detector` for more details.
    // The following is checked by unit tests. This is the reason why
    // `alpha_g_detector` strongly suggests to avoid using the indices directly.
    // But we know exactly what we are doing here and have unit tests to
    // guarantee we are not violating any assumptions.
    let shifted_index = wire.wrapping_sub(WIRE_SHIFT) & 0xff;
    // Now, this shifted index does align with the pad columns.
    shifted_index / WIRES_PER_COLUMN
}

// Given a `pad_column` return the [first, last) wire indices that are
// contained in that pad column.
pub(crate) fn pad_column_to_wires(pad_column: usize) -> Range<usize> {
    let first = ((pad_column * WIRES_PER_COLUMN) + WIRE_SHIFT) & 0xff;

    first..first + WIRES_PER_COLUMN
}

#[derive(Clone, Copy, Debug)]
struct WireHit {
    phi: Angle,
    amplitude: f64,
}

fn wire_hits_at_t(
    wire_indices: [usize; WIRES_PER_COLUMN],
    wire_inputs: &[Vec<f64>; WIRES_PER_COLUMN],
    t: usize,
) -> Vec<WireHit> {
    wire_indices
        .iter()
        .zip(wire_inputs)
        .filter_map(|(index, input)| {
            input.get(t).copied().filter(|v| v > &0.0).map(|v| WireHit {
                phi: Angle::new::<radian>(TpcWirePosition::try_from(*index).unwrap().phi()),
                amplitude: v,
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct PadHit {
    z: Length,
    amplitude: f64,
}

fn pad_hits_at_t(pad_column_inputs: &[Vec<f64>; TPC_PAD_ROWS], t: usize) -> Vec<PadHit> {
    let mut pad_hits = Vec::new();

    let mut first = pad_column_inputs[0].get(t).copied().unwrap_or(0.0);
    let mut middle = pad_column_inputs[1].get(t).copied().unwrap_or(0.0);
    for (row, input) in pad_column_inputs.iter().enumerate().skip(2) {
        let last = input.get(t).copied().unwrap_or(0.0);

        if first > 0.0 && last > 0.0 && middle > first && middle > last {
            // See equation 10.3 from "Gaseous Radiation Detectors" by Sauli.
            let width = Length::new::<meter>(PAD_PITCH_Z);
            let sigma_squared = width.powi(P2::new()) / (middle.powi(2) / (first * last)).ln();
            let z = Length::new::<meter>(TpcPadRow::try_from(row - 1).unwrap().z())
                + (sigma_squared / (2.0 * width)) * (last / first).ln();

            let amplitude = middle;
            pad_hits.push(PadHit { z, amplitude });
        }

        first = middle;
        middle = last;
    }

    pad_hits
}

// Match the inputs from all the wires in a pad column to the input from the
// pad column to reconstruct avalanches.
pub(crate) fn match_column_inputs(
    wire_indices: [usize; WIRES_PER_COLUMN],
    wire_inputs: &[Vec<f64>; WIRES_PER_COLUMN],
    pad_column_inputs: &[Vec<f64>; TPC_PAD_ROWS],
) -> Vec<Avalanche> {
    let t_max = wire_inputs.iter().map(|input| input.len()).max().unwrap();

    let mut avalanches = Vec::new();
    for t in 0..t_max {
        let mut wire_hits = wire_hits_at_t(wire_indices, wire_inputs, t);
        if wire_hits.is_empty() {
            continue;
        }
        let mut pad_hits = pad_hits_at_t(pad_column_inputs, t);
        // Sort by amplitude (descending order) before matching. This matches
        // together largest avalanches first and tries to fix the ghosting
        // problem by taking into account the avalanches amplitudes.
        wire_hits.sort_unstable_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap());
        pad_hits.sort_unstable_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap());

        avalanches.extend(
            wire_hits
                .into_iter()
                .zip(pad_hits)
                .map(|(wire_hit, pad_hit)| Avalanche {
                    t: Time::new::<second>(t as f64 / ADC32_RATE),
                    phi: wire_hit.phi,
                    z: pad_hit.z,
                    wire_amplitude: wire_hit.amplitude,
                    pad_amplitude: pad_hit.amplitude,
                }),
        );
    }

    avalanches
}

#[cfg(test)]
mod tests;
