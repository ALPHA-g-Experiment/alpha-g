use thiserror::Error;

/// The error type returned when conversion from unsigned integer to
/// [`ChannelId`] fails.
#[derive(Debug, Error)]
#[error("unknown conversion from unsigned `{input}` to ChannelId")]
pub struct TryChannelIdFromUnsignedError {
    input: u8,
}

const NUM_INPUT_CHANNELS: usize = 59;
/// Input channel in a ChronoBox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelId(u8);
impl TryFrom<u8> for ChannelId {
    type Error = TryChannelIdFromUnsignedError;

    /// There are 59 valid input channel IDs. Perform the conversion from an
    /// integer in range `0..=58`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num < u8::try_from(NUM_INPUT_CHANNELS).unwrap() {
            Ok(ChannelId(num))
        } else {
            Err(TryChannelIdFromUnsignedError { input: num })
        }
    }
}

/// The [`EdgeType`] represents the leading or trailing edge of an input signal.
#[derive(Clone, Copy, Debug)]
pub enum EdgeType {
    Leading,
    Trailing,
}

/// Timestamp counter.
#[derive(Clone, Copy, Debug)]
pub struct TimestampCounter {
    pub channel: ChannelId,
    // The timestamp is only 24 bits wide. Make it private to ensure that the
    // value is always within the valid range.
    timestamp: u32,
    pub edge: EdgeType,
}

impl TimestampCounter {
    /// Returns the timestamp value. This counter is 24 bits wide, hence the
    /// most significant 8 bits are always zero.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

/// Wrap around marker.
///
/// The wrap around marker is used to keep track of the overflow of the
/// timestamp counter. To avoid ambiguity, additional markers are written half
/// way through the wrap around.
#[derive(Clone, Copy, Debug)]
pub struct WrapAroundMarker {
    /// If the top bit is not yet set, the wrap around is at the half way point.
    pub timestamp_top_bit: bool,
    // The wrap around marker is only 23 bits wide. Make it private to ensure
    // that the value is always within the valid range.
    counter: u32,
}

impl WrapAroundMarker {
    /// Returns the wrap around counter. This counter increments with every wrap
    /// around marker written to the Chronobox FIFO (i.e. 2 times per timestamp
    /// counter overflow).
    ///
    /// This counter is 23 bits wide, hence the most significant 9 bits are
    /// always zero.
    pub fn wrap_around_counter(&self) -> u32 {
        self.counter
    }
}

/// Entry in the ChronoBox FIFO.
#[derive(Clone, Copy, Debug)]
pub enum FifoEntry {
    TimestampCounter(TimestampCounter),
    WrapAroundMarker(WrapAroundMarker),
}

/// Frequency (Hertz) of the system clock.
pub const SYS_CLOCK_FREQ: f64 = 100e6;

/// Chronobox data packet.
///
/// A [`ChronoPacket`] represents the data collected from a single Chronobox.
#[derive(Clone, Debug)]
pub struct ChronoPacket {
    pub fifo: Vec<FifoEntry>,
    scalers: [u32; NUM_INPUT_CHANNELS],
    /// System clock counter which increments at a frequency of
    /// [`SYS_CLOCK_FREQ`].
    pub sys_clock: u32,
}

impl ChronoPacket {
    /// Returns the latched scaler at [`ChronoPacket::sys_clock`] for the given
    /// channel.
    pub fn scaler(&self, channel: ChannelId) -> u32 {
        self.scalers[usize::from(channel.0)]
    }
}

#[cfg(test)]
mod tests;
