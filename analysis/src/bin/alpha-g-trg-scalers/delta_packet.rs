use alpha_g_detector::trigger::TrgPacket;
use anyhow::{anyhow, ensure, Result};

/// Represents the difference between two TRG packets.
// To make things simpler:
// delta = current - previous
// where `current` is a packet that happens AFTER `previous`
// That way I don't need to care about signed differences (they should all be
// treated as errors).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeltaPacket {
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
    pub(crate) fn try_from(current: &TrgPacket, previous: &TrgPacket) -> Result<DeltaPacket> {
        let timestamp = current.timestamp().wrapping_sub(previous.timestamp());
        let output_counter = current
            .output_counter()
            .checked_sub(previous.output_counter())
            .ok_or_else(|| anyhow!("decreasing output counter"))?;
        let input_counter = current
            .input_counter()
            .checked_sub(previous.input_counter())
            .ok_or_else(|| anyhow!("decreasing input counter"))?;
        let pulser_counter = current
            .pulser_counter()
            .checked_sub(previous.pulser_counter())
            .ok_or_else(|| anyhow!("decreasing pulser counter"))?;
        let drift_veto_counter = match (current.drift_veto_counter(), previous.drift_veto_counter())
        {
            (Some(current), Some(previous)) => current
                .checked_sub(previous)
                .ok_or_else(|| anyhow!("decreasing drift veto counter"))?,
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
                .ok_or_else(|| anyhow!("decreasing scaledown counter"))?,
            (None, None) => 0,
            // Same as drift_veto above
            _ => unreachable!(),
        };

        ensure!(
            output_counter != 0 && input_counter != 0,
            "non-incrementing counter"
        );
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
