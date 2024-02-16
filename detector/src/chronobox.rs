use thiserror::Error;

/// The error type returned when conversion from unsigned integer to
/// [`ChannelId`] fails.
#[derive(Debug, Error)]
#[error("unknown conversion from unsigned `{input}` to ChannelId")]
pub struct TryChannelIdFromUnsignedError {
    input: u8,
}

/// Input channel in a ChronoBox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelId(u8);
impl TryFrom<u8> for ChannelId {
    type Error = TryChannelIdFromUnsignedError;

    /// There are 60 valid channel IDs. Perform the conversion from an integer
    /// in range `0..=59`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num > 59 {
            Err(TryChannelIdFromUnsignedError { input: num })
        } else {
            Ok(ChannelId(num))
        }
    }
}

#[cfg(test)]
mod tests;
