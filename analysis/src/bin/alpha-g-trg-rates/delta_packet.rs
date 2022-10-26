use alpha_g_detector::trigger::TrgPacket;

/// Represents the difference between two TRG packets.
// To make things simpler:
// delta = current - previous
// where `current` is a packet that happens AFTER `previous`
// That way I don't need to care about signed differences (they should all be
// treated as errors).
#[derive(Clone, Copy, Debug)]
pub struct DeltaPacket {
    pub timestamp: u32,
    pub output_counter: u32,
    pub input_counter: u32,
    pub pulser_counter: u32,
    pub drift_veto_counter: u32,
    pub scaledown_counter: u32,
}

impl DeltaPacket {
    // Return an error if `previous` happens AFTER `current` or if they are
    // equal. DeltaPacket is only meant to represent differences against a
    // previous packet (from BEFORE).
    pub fn try_from(current: &TrgPacket, previous: &TrgPacket) -> Result<DeltaPacket, String> {
        let timestamp = current.timestamp().wrapping_sub(previous.timestamp());
        let output_counter = current
            .output_counter()
            .checked_sub(previous.output_counter())
            .ok_or_else(|| "corrupted output counter".to_string())?;
        let input_counter = current
            .input_counter()
            .checked_sub(previous.input_counter())
            .ok_or_else(|| "corrupted input counter".to_string())?;
        let pulser_counter = current
            .pulser_counter()
            .checked_sub(previous.pulser_counter())
            .ok_or_else(|| "corrupted pulser counter".to_string())?;
        let drift_veto_counter = match (current.drift_veto_counter(), previous.drift_veto_counter())
        {
            (Some(current), Some(previous)) => current
                .checked_sub(previous)
                .ok_or_else(|| "corrupted drift veto counter".to_string())?,
            (None, None) => 0,
            // Currently the field is either always Some, or always
            // None. It is not possible to have mixed states within
            // the same version (and they are same version because
            // they come form the same file).
            _ => unreachable!(),
        };
        let scaledown_counter = match (current.scaledown_counter(), previous.scaledown_counter()) {
            (Some(current), Some(previous)) => current
                .checked_sub(previous)
                .ok_or_else(|| "corrupted scaledown counter".to_string())?,
            (None, None) => 0,
            // Same as drift_veto above
            _ => unreachable!(),
        };
        if output_counter == 0 || input_counter == 0 {
            return Err("non-incrementing counter".to_string());
        }
        Ok(DeltaPacket {
            timestamp,
            output_counter,
            input_counter,
            pulser_counter,
            drift_veto_counter,
            scaledown_counter,
        })
    }
}

#[cfg(test)]
mod tests;
