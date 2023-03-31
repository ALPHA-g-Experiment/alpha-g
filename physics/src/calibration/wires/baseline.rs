use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/wires/baseline/";
    // All the following files are embedded at compile time.
    // Add new files to the list below to include them.
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
    static ref MAP_7026: HashMap<TpcWirePosition, i16> = complete_from_bytes(BYTES_7026);
}
/// Try to get the baseline for a given wire. Return an error if there is no map
/// available for the given run number.
///
/// If there is no baseline for a given wire return None. This is not an error
/// because a channel could simply be disconnected.
pub(crate) fn try_wire_baseline(
    run_number: u32,
    wire: TpcWirePosition,
) -> Result<Option<i16>, WireBaselineMapError> {
    // This map changes whenever a new file is added.
    let map = match run_number {
        7026.. => &MAP_7026,
        _ => return Err(WireBaselineMapError { run_number }),
    };

    Ok(map.get(&wire).copied())
}

// Nothing below this line needs to be changed when new files are added.

/// The error type returned when the baseline calibration map does not exist for
/// a given run number.
#[derive(Debug, Error)]
#[error("no wire baseline calibration available for run number `{run_number}`")]
pub struct WireBaselineMapError {
    run_number: u32,
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
