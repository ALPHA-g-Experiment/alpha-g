use thiserror::Error;

pub(crate) fn try_pad_delay(run_number: u32) -> Result<usize, MapPadDelayError> {
    match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => Ok(100),
        // Safe guard in case I die and nobody notices that they haven't
        // calibrated the detector in a very long time.
        10418.. => panic!("bump by another 2000 runs if current calibration is still valid"),
        // This is basically a fixed value, but it is still good to check it
        // every once in a while. This will change if e.g. the main trigger
        // changes from the MLU2+ that has been used since forever.
        7000.. => Ok(115),
        _ => Err(MapPadDelayError::MissingMap { run_number }),
    }
}

/// The error type returned when the ADC delay calibration is not available.
#[derive(Debug, Error)]
pub enum MapPadDelayError {
    #[error("no pad delay calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
}

#[cfg(test)]
mod tests;
