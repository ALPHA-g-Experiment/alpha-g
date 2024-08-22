use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
use lazy_static::{__Deref, lazy_static};
use std::collections::HashMap;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/wires/baseline/";
    // All the following files are embedded at compile time.
    // Add new files to the list below to include them.
    BYTES_SIMULATION = "simulation_complete.json",
    BYTES_7026 = "7026_complete.json",
}

lazy_static! {
    // Whenever a new file is added, generate the appropriate new HashMap.
    // Do not delete any of the existing maps.
    //
    // Adding a new map is as simple as either:
    // complete_from_bytes(BYTES_NUMBER)
    // or
    // update_previous_from_bytes(&PREVIOUS_HASHMAP, BYTES_NUMBER)
    static ref MAP_SIMULATION: HashMap<TpcWirePosition, i16> = complete_from_bytes(BYTES_SIMULATION);
    static ref MAP_7026: HashMap<TpcWirePosition, i16> = complete_from_bytes(BYTES_7026);
}
/// Try to get the baseline for a given wire. Return an error if there is no map
/// available for the given run number or if there is no baseline for a given
/// wire in the map.
pub(crate) fn try_wire_baseline(
    run_number: u32,
    wire: TpcWirePosition,
) -> Result<i16, MapWireBaselineError> {
    // This map should be updated whenever a new file is added.
    let map = match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => MAP_SIMULATION.deref(),
        7026.. => MAP_7026.deref(),
        _ => return Err(MapWireBaselineError::MissingMap { run_number }),
    };

    map.get(&wire)
        .copied()
        .ok_or(MapWireBaselineError::MissingWire { run_number, wire })
}

// Nothing below this line needs to be changed when new files are added.

/// The error type returned when the baseline calibration map is not available.
#[derive(Debug, Error)]
pub enum MapWireBaselineError {
    #[error("no wire baseline calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
    #[error("no baseline calibration available for wire `{wire:?}` in run number `{run_number}`")]
    MissingWire {
        run_number: u32,
        wire: TpcWirePosition,
    },
}

fn complete_from_bytes(bytes: &[u8]) -> HashMap<TpcWirePosition, i16> {
    // Correctness of the format is checked by unit tests.
    let map: HashMap<TpcWirePosition, (f64, f64, usize)> = serde_json::from_slice(bytes).unwrap();
    map.into_iter()
        .map(|(wire, (baseline, _, _))| (wire, baseline.round() as i16))
        .collect()
}

fn _update_previous_from_bytes(
    previous: &HashMap<TpcWirePosition, i16>,
    bytes: &[u8],
) -> HashMap<TpcWirePosition, i16> {
    // Correctness of the format is checked by unit tests.
    let update: HashMap<TpcWirePosition, Option<(f64, f64, usize)>> =
        serde_json::from_slice(bytes).unwrap();
    let update: HashMap<_, _> = update
        .into_iter()
        .map(|(k, v)| (k, v.map(|(baseline, _, _)| baseline.round() as i16)))
        .collect();

    let mut map = previous.clone();
    for (wire, value) in update {
        match value {
            Some(value) => map.insert(wire, value),
            None => map.remove(&wire),
        };
    }
    map
}

#[cfg(test)]
mod tests;
