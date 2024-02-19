use thiserror::Error;
use winnow::binary::{le_u24, le_u32, length_repeat, u8};
use winnow::combinator::{alt, empty, repeat_till, seq};
use winnow::error::{ContextError, StrContext};
use winnow::{PResult, Parser};

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

fn timestamp_counter(input: &mut &[u8]) -> PResult<TimestampCounter> {
    let ts = le_u24.parse_next(input)?;
    let timestamp = ts & 0x00FFFFFE;
    let edge = if ts & 1 == 1 {
        EdgeType::Trailing
    } else {
        EdgeType::Leading
    };

    let channel = u8
        .verify(|&n| n & 0x80 == 0x80)
        .try_map(|n| ChannelId::try_from(n & 0x7F))
        .parse_next(input)?;

    Ok(TimestampCounter {
        channel,
        timestamp,
        edge,
    })
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

fn wrap_around_marker(input: &mut &[u8]) -> PResult<WrapAroundMarker> {
    let (timestamp_top_bit, counter) = le_u32
        .context(StrContext::Label("wrap around counter word"))
        .verify(|&word| word & 0xFF000000 == 0xFF000000)
        .context(StrContext::Label("top 8 bits"))
        .map(|word| {
            let timestamp_top_bit = word & 0x00800000 != 0;
            let counter = word & 0x007FFFFF;

            (timestamp_top_bit, counter)
        })
        .parse_next(input)?;

    Ok(WrapAroundMarker {
        timestamp_top_bit,
        counter,
    })
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

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to
/// [`ChronoPacket`] fails.
#[derive(Debug)]
pub struct TryChronoPacketFromBytesError {
    offset: usize,
    inner: ContextError,
}

impl std::fmt::Display for TryChronoPacketFromBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing stopped at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for TryChronoPacketFromBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
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

fn chrono_packet(input: &mut &[u8]) -> PResult<ChronoPacket> {
    seq! {ChronoPacket{
        fifo: repeat_till(0.., alt((
            timestamp_counter.map(FifoEntry::TimestampCounter),
            wrap_around_marker.map(FifoEntry::WrapAroundMarker)
        )), b"\x3C\x00\x00\xFE").map(|(fifo, _)| fifo),
        scalers: length_repeat(empty.value(NUM_INPUT_CHANNELS), le_u32)
            .map(|v: Vec<_>| v.try_into().unwrap()),
        sys_clock: le_u32,
    }}
    .parse_next(input)
}

impl TryFrom<&[u8]> for ChronoPacket {
    type Error = TryChronoPacketFromBytesError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        chrono_packet
            .parse(bytes)
            .map_err(|e| TryChronoPacketFromBytesError {
                offset: e.offset(),
                inner: e.into_inner(),
            })
    }
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
