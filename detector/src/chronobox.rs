use thiserror::Error;
use winnow::binary::{le_u24, le_u32, u8};
use winnow::combinator::{alt, empty, repeat, separated_foldl1, seq};
use winnow::token::take;
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
impl From<ChannelId> for u8 {
    /// Convert to the `u: u8` such that
    /// `ChannelId::try_from(u).unwrap() == self`.
    fn from(channel: ChannelId) -> Self {
        channel.0
    }
}

/// The [`EdgeType`] represents the leading or trailing edge of an input signal.
#[derive(Clone, Copy, Debug)]
pub enum EdgeType {
    Leading,
    Trailing,
}

/// The size of the timestamp counter in bits.
pub const TIMESTAMP_BITS: u32 = 24;
/// Frequency (Hertz) of the timestamp clock.
pub const TIMESTAMP_CLOCK_FREQ: f64 = 10e6;

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
    let temp = le_u24.parse_next(input)?;

    seq! {TimestampCounter{
        timestamp: empty.value(temp & 0x00FFFFFE),
        edge: empty.value(if temp & 1 == 1 {EdgeType::Trailing} else {EdgeType::Leading}),
        channel: u8
            .verify(|&n| n & 0x80 == 0x80)
            .try_map(|n| ChannelId::try_from(n & 0x7F))
    }}
    .parse_next(input)
}

impl TimestampCounter {
    /// Returns the timestamp value. This counter is [`TIMESTAMP_BITS`] bits
    /// wide, hence the remaining most significant bits are always zero.
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
    let temp = le_u24.parse_next(input)?;

    seq! {WrapAroundMarker{
        timestamp_top_bit: empty.value(temp & 0x00800000 == 0x00800000),
        counter: empty.value(temp & 0x007FFFFF),
        _: 0xFF,
    }}
    .parse_next(input)
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

fn fifo_entry(input: &mut &[u8]) -> PResult<FifoEntry> {
    alt((
        timestamp_counter.map(FifoEntry::TimestampCounter),
        wrap_around_marker.map(FifoEntry::WrapAroundMarker),
    ))
    .parse_next(input)
}

fn scalers_block(input: &mut &[u8]) -> PResult<()> {
    (
        b"\x3C\x00\x00\xFE",
        take(NUM_INPUT_CHANNELS * std::mem::size_of::<u32>()),
        le_u32,
    )
        .void()
        .parse_next(input)
}

// The chronobox data banks are arbitrary sub-slices of a complete/correct
// data stream. This means that parsing correctly may need information from
// previous data banks (e.g. a scaler block may be split between two banks).
// Then, instead of having a "Packet" structure (like all other data banks),
// we have a partial parser that stops when it needs more data (so that the user
// can append it and resume parsing).
/// Parse Chronobox FIFO data from a slice of bytes (skipping scalers blocks).
///
/// The input slice is advanced up until no more FIFO entries can be parsed.
/// Note that the slice may stop before consuming all the data. This could mean
/// that:
/// - More data is required (e.g. the input slice stops in the middle of a
///  scalers block). In this case, the user should append more data and resume
///  parsing.
/// - The data is not correctly formatted. Even after appending more data, the
/// input slice is still stuck.
pub fn chronobox_fifo(input: &mut &[u8]) -> Vec<FifoEntry> {
    separated_foldl1(
        repeat(0.., fifo_entry),
        scalers_block,
        |mut l: Vec<_>, _, mut r| {
            l.append(&mut r);
            l
        },
    )
    .parse_next(input)
    // It is OK to unwrap because this parser always succeeds. Worst case it
    // returns an empty vector, which is a valid result.
    .unwrap()
}

// Known Chronobox names.
const CHRONOBOX_NAMES: [&str; 4] = ["cb01", "cb02", "cb03", "cb04"];

/// The error type returned when parsing a [`BoardId`] from a string fails.
#[derive(Debug, Error)]
#[error("unknown parsing from board name `{input}` to BoardId")]
pub struct ParseBoardIdError {
    input: String,
}

/// Identity of a physical Chronobox.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoardId(&'static str);

impl TryFrom<&str> for BoardId {
    type Error = ParseBoardIdError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match CHRONOBOX_NAMES.iter().find(|&&n| n == name) {
            Some(&n) => Ok(BoardId(n)),
            None => Err(ParseBoardIdError {
                input: name.to_string(),
            }),
        }
    }
}

impl BoardId {
    /// Returns the name of a physical Chronobox. This is a human readable
    /// name used to identify the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::chronobox::BoardId;
    ///
    /// let board_id = BoardId::try_from("cb01")?;
    /// assert_eq!(board_id.name(), "cb01");
    /// # Ok(())
    /// # }
    pub fn name(&self) -> &str {
        self.0
    }
}

#[cfg(test)]
mod tests;
