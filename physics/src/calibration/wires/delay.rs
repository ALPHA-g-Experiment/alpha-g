use thiserror::Error;

pub(crate) fn try_wire_delay(run_number: u32) -> Result<usize, MapWireDelayError> {
    match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => Ok(100),
        // Safe guard in case I die and nobody notices that they haven't
        // calibrated the detector in a very long time.
        10418.. => panic!("bump by another 2000 runs if current calibration is still valid"),
        9567.. => Ok(129),
        _ => Err(MapWireDelayError::MissingMap { run_number }),
    }
}

/// The error type returned when the ADC delay calibration is not available.
#[derive(Debug, Error)]
pub enum MapWireDelayError {
    #[error("no wire delay calibration available for run number `{run_number}`")]
    MissingMap { run_number: u32 },
}

#[cfg(test)]
mod tests;
