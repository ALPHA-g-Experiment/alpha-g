use thiserror::Error;

pub(crate) fn try_wire_delay(run_number: u32) -> Result<usize, MapWireDelayError> {
    match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => Ok(100),
        // This is basically a fixed value, but it is still good to check it
        // every once in a while. This will change if e.g. the main trigger
        // changes from the MLU2+ that has been used since forever.
        7000.. => Ok(129),
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
