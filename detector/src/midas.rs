use crate::alpha16::{Adc16ChannelId, BoardId, ParseBoardIdError, TryChannelIdFromUnsignedError};
use std::num::ParseIntError;
use std::{error::Error, fmt};

/// The error type returned when conversion from unsigned integer to [`EventId`]
/// fails.
#[derive(Clone, Copy, Debug)]
pub struct TryEventIdFromUnsignedError;
impl fmt::Display for TryEventIdFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown event id number attempted")
    }
}
impl Error for TryEventIdFromUnsignedError {}

/// Possible ID of an event in an ALPHA-g MIDAS file.
#[derive(Clone, Copy, Debug)]
pub enum EventId {
    /// Main ALPHA-g event. These events include data from the rTPC and BV
    /// detectors.
    Main,
}

impl TryFrom<u16> for EventId {
    type Error = TryEventIdFromUnsignedError;

    fn try_from(num: u16) -> Result<Self, Self::Error> {
        match num {
            1 => Ok(EventId::Main),
            _ => Err(TryEventIdFromUnsignedError),
        }
    }
}

/// The error type returned when parsing an Alpha16 bank name fails.
#[derive(Clone, Copy, Debug)]
pub enum ParseAlpha16BankNameError {
    /// Input string pattern doesn't match expected Alpha16 bank name pattern.
    PatternMismatch,
    /// Board name doesn't match any known [`BoardId`].
    UnknownBoardId,
    /// Channel ID doesn't match any known [`ChannelId`].
    UnknownChannelId,
}
impl fmt::Display for ParseAlpha16BankNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PatternMismatch => write!(f, "pattern mismatch"),
            Self::UnknownBoardId => write!(f, "unknown board id"),
            Self::UnknownChannelId => write!(f, "unknown channel id"),
        }
    }
}
impl Error for ParseAlpha16BankNameError {}
impl From<ParseBoardIdError> for ParseAlpha16BankNameError {
    fn from(_: ParseBoardIdError) -> Self {
        Self::UnknownBoardId
    }
}
impl From<TryChannelIdFromUnsignedError> for ParseAlpha16BankNameError {
    fn from(_: TryChannelIdFromUnsignedError) -> Self {
        Self::UnknownChannelId
    }
}
impl From<ParseIntError> for ParseAlpha16BankNameError {
    fn from(_: ParseIntError) -> Self {
        Self::UnknownChannelId
    }
}

/// Bank name that corresponds to SiPMs of the Barrel Veto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc16BankName {
    pub board_id: BoardId,
    pub channel_id: Adc16ChannelId,
}
impl TryFrom<&str> for Adc16BankName {
    type Error = ParseAlpha16BankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if !name.starts_with('B')
            || name.len() != 4
            || !name.chars().all(|c| c.is_ascii_alphanumeric())
            || name.chars().any(|c| c.is_ascii_lowercase())
        {
            return Err(Self::Error::PatternMismatch);
        }
        let board_id = BoardId::try_from(&name[1..][..2])?;
        let channel_id = Adc16ChannelId::try_from(u8::from_str_radix(&name[3..], 16)?)?;
        Ok(Adc16BankName {
            board_id,
            channel_id,
        })
    }
}

#[cfg(test)]
mod tests;
