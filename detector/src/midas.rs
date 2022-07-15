use std::{error::Error, fmt};

/// The error type returned when conversion from unsigned integer to [`EventId`]
/// fails,
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

#[cfg(test)]
mod tests;
