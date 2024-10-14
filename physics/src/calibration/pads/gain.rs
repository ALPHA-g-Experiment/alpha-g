use alpha_g_detector::padwing::map::TpcPadPosition;
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/pads/gain/";
    // All the following files are embedded at compile time.
    // Add new files to the list below.
    BYTES_SIMULATION = "simulation_complete.ron",
    BYTES_9277 = "9277_complete.ron",
    BYTES_11186 = "11186_complete.ron",
}

lazy_static! {
    // Whenever a new file is added, generate the appropriate new HashMap.
    // Do not delete any of the existing maps.
    //
    // Adding a new map is as simple as:
    // complete_from_bytes(BYTES_NUMBER)
    static ref MAP_SIMULATION: HashMap<TpcPadPosition, f64> = complete_from_bytes(BYTES_SIMULATION);
    static ref MAP_9277: HashMap<TpcPadPosition, f64> = complete_from_bytes(BYTES_9277);
    static ref MAP_11186: HashMap<TpcPadPosition, f64> = complete_from_bytes(BYTES_11186);
}
/// Try to get the gain for a given pad. Return an error if there is no map
/// available for the given run number or if there is no gain for a given pad in
/// the map.
pub(crate) fn try_pad_gain(run_number: u32, pad: TpcPadPosition) -> Result<f64, MapPadGainError> {
    // This map should be updated whenever a new file is added.
    let map = match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => &*MAP_SIMULATION,
        // The calibration was done on run 11186. But the detector was in this
        // configuration since run 11084 when it was turned on.
        11084.. => &*MAP_11186,
        9277.. => &*MAP_9277,
        _ => return Err(MapPadGainError::MissingMap { run_number }),
    };

    map.get(&pad)
        .copied()
        .ok_or(MapPadGainError::MissingPad { run_number, pad })
}

// Nothing below this line needs to be changed when adding a new file.

/// The error type returned when the gain calibration map is not available.
#[derive(Debug, Error)]
pub enum MapPadGainError {
    #[error("no pad gain calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
    #[error("no pad gain calibration available for pad `{pad:?}` in run number `{run_number}`")]
    MissingPad {
        run_number: u32,
        pad: TpcPadPosition,
    },
}

fn complete_from_bytes(bytes: &[u8]) -> HashMap<TpcPadPosition, f64> {
    // Correctness of the file is checked by unit tests.
    ron::de::from_bytes(bytes).unwrap()
}

#[cfg(test)]
mod tests;
