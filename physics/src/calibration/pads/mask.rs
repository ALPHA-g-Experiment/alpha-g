use alpha_g_detector::padwing::map::TpcPadPosition;
use lazy_static::lazy_static;
use thiserror::Error;

includes! {
    DATA_PATH = "../../../data/calibration/pads/mask/";
    // All the following files are embedded at compile time.
    // Add new files to the list below.
    BYTES_9277 = "9277_complete.ron",
    BYTES_11186 = "11186_complete.ron",
}

lazy_static! {
    // Do not delete any of the existing maps.
    //
    // Adding a new map is just:
    // complete_from_bytes(BYTES_NUMBER)
    static ref MASK_SIMULATION: Vec<TpcPadPosition> = Vec::new();
    static ref MASK_9277: Vec<TpcPadPosition> = complete_from_bytes(BYTES_9277);
    static ref MASK_11186: Vec<TpcPadPosition> = complete_from_bytes(BYTES_11186);
}

pub(crate) fn try_is_pad_masked(
    run_number: u32,
    pos: TpcPadPosition,
) -> Result<bool, PadMaskError> {
    let mask = match run_number {
        // u32::MAX corresponds to a simulation run.
        u32::MAX => &*MASK_SIMULATION,
        // The calibration was done on run 11186. But the detector was in this
        // configuration since run 11084 when it was turned on.
        11084.. => &*MASK_11186,
        9277.. => &*MASK_9277,
        _ => return Err(PadMaskError::MissingMask { run_number }),
    };

    Ok(mask.contains(&pos))
}

// Nothing below this line needs to be changed when adding a new file.

/// The error type returned when the pad mask is not available.
#[derive(Debug, Error)]
pub enum PadMaskError {
    #[error("no pad mask available for run number `{run_number}`")]
    MissingMask { run_number: u32 },
}

fn complete_from_bytes(bytes: &[u8]) -> Vec<TpcPadPosition> {
    // Correctness of the file is checked by unit tests.
    ron::de::from_bytes(bytes).unwrap()
}

#[cfg(test)]
mod tests;
