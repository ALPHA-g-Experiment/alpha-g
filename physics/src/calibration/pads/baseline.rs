use alpha_g_detector::padwing::map::TpcPadPosition;
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/pads/baseline/";
    // All the following files are embedded at compile time.
    // Add new files to the list below.
    BYTES_SIMULATION = "simulation_complete.ron",
    BYTES_9277 = "9277_complete_handwritten_cherry_picked_see_commit.ron",
}

lazy_static! {
    // Whenever a new file is added, generate the appropriate new HashMap.
    // Do not delete any of the existing maps.
    //
    // Adding a new map is as simple as either:
    // complete_from_bytes(BYTES_NUMBER)
    // or
    // update_previous_from_bytes(&PREVIOUS_HASHMAP, BYTES_NUMBER)
    static ref MAP_SIMULATION: HashMap<TpcPadPosition, i16> = complete_from_bytes(BYTES_SIMULATION);
    static ref MAP_9277: HashMap<TpcPadPosition, i16> = complete_from_bytes(BYTES_9277);
}

/// Try to get the baseline for a given pad. Return an error if there is no map
/// available for the given run number or if there is no baseline for the given
/// pad in the map.
pub(crate) fn try_pad_baseline(
    run_number: u32,
    pad: TpcPadPosition,
) -> Result<i16, MapPadBaselineError> {
    // This map should be updated whenever a new file is added.
    let map = match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => &*MAP_SIMULATION,
        // Safe guard in case I die and nobody notices that they haven't
        // calibrated the detector in a very long time.
        10418.. => panic!("bump by another 2000 runs if current calibration is still valid"),
        9277.. => &*MAP_9277,
        _ => return Err(MapPadBaselineError::MissingMap { run_number }),
    };

    map.get(&pad)
        .copied()
        .ok_or(MapPadBaselineError::MissingPad { run_number, pad })
}

// Nothing below this line needs to be changed when adding a new file.

/// The error type returned when the baseline calibration map is not available
/// for a given run number and pad.
#[derive(Debug, Error)]
pub enum MapPadBaselineError {
    #[error("no pad baseline calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
    #[error("no baseline available for pad `{pad:?}` in run number `{run_number}`")]
    MissingPad {
        run_number: u32,
        pad: TpcPadPosition,
    },
}

fn complete_from_bytes(bytes: &[u8]) -> HashMap<TpcPadPosition, i16> {
    // Correctness of the format is checked by unit tests.
    let map: HashMap<TpcPadPosition, (f64, f64, usize)> = ron::de::from_bytes(bytes).unwrap();

    map.into_iter()
        .map(|(pad, (baseline, _, _))| (pad, baseline.round() as i16))
        .collect()
}

// Implement `update_previous_from_bytes` whenever I need it for the first time.
// The implementation would just be a copy-paste of the anode wires calibration.
// I don't need it now, so I just want to implement it whenever I have a use
// case to add unit tests for it.

#[cfg(test)]
mod tests;
