use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/wires/gain/";
    // All the following files are embedded at compile time.
    // Add new files to the list below to include them.
    BYTES_SIMULATION = "simulation_complete.json",
    BYTES_9277 = "9277_complete.json",
}

lazy_static! {
    // Whenever a new file is added, generate the appropriate new HashMap.
    // Do not delete any of the existing maps.
    //
    // Adding a new map is as simple as:
    // complete_from_bytes(BYTES_NUMBER)
    static ref MAP_SIMULATION: HashMap<TpcWirePosition, f64> = complete_from_bytes(BYTES_SIMULATION);
    static ref MAP_9277: HashMap<TpcWirePosition, f64> = complete_from_bytes(BYTES_9277);
}
/// Try to get the gain for a given wire. Return an error if there is no map
/// available for the given run number or if there is no gain for a given
/// wire in the map.
pub(crate) fn try_wire_gain(
    run_number: u32,
    wire: TpcWirePosition,
) -> Result<f64, MapWireGainError> {
    // This map should be updated whenever a new file is added.
    let map = match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => &*MAP_SIMULATION,
        // Safe guard in case I die and nobody notices that they haven't
        // calibrated the detector in a very long time.
        10418.. => panic!("bump by another 2000 runs if current calibration is still valid"),
        9277.. => &*MAP_9277,
        _ => return Err(MapWireGainError::MissingMap { run_number }),
    };

    map.get(&wire)
        .copied()
        .ok_or(MapWireGainError::MissingWire { run_number, wire })
}

// Nothing below this line needs to be changed when new files are added.

/// The error type returned when the gain calibration map is not available.
#[derive(Debug, Error)]
pub enum MapWireGainError {
    #[error("no wire gain calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
    #[error("no wire gain calibration available for wire `{wire:?}` in run number `{run_number}`")]
    MissingWire {
        run_number: u32,
        wire: TpcWirePosition,
    },
}

fn complete_from_bytes(bytes: &[u8]) -> HashMap<TpcWirePosition, f64> {
    // Correctness of the format is checked by unit tests.
    serde_json::from_slice(bytes).unwrap()
}

#[cfg(test)]
mod tests;
