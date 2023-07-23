use crate::alpha16::{Adc16ChannelId, Adc32ChannelId, ChannelId};
use std::num::ParseIntError;
use thiserror::Error;

/// JSON pointer that identifies the ADC16 data suppression threshold in the
/// ODB.
pub const ADC16_SUPPRESSION_THRESHOLD_JSON_PTR: &str =
    "/Equipment/CTRL/Settings/ADC/adc16_sthreshold";
/// JSON pointer that identifies the ADC32 data suppression enable flag in the
/// ODB.
pub const ADC32_SUPPRESSION_ENABLE_JSON_PTR: &str =
    "/Equipment/CTRL/Settings/ADC/adc32_ch_suppress";
/// JSON pointer that identifies the ADC32 data suppression threshold in the
/// ODB.
pub const ADC32_SUPPRESSION_THRESHOLD_JSON_PTR: &str =
    "/Equipment/CTRL/Settings/ADC/adc32_sthreshold";
/// JSON pointer that identifies the Barrel Veto pulser enable flag in the ODB.
pub const BSC_PULSER_ENABLE_JSON_PTR: &str = "/Equipment/CTRL/Settings/BscPulserEnable";
/// JSON pointer that identifies the field wire pulser enable flag in the ODB.
pub const FIELD_WIRE_PULSER_ENABLE_JSON_PTR: &str = "/Equipment/CTRL/Settings/FwPulserEnable";
/// JSON pointer that identifies the pulser enable flag in the ODB.
pub const PULSER_ENABLE_JSON_PTR: &str = "/Equipment/CTRL/Settings/Pulser/Enable";
/// JSON pointer that identifies the PWB force channels flag in the ODB (i.e.
/// disable data suppression).
pub const PWB_FORCE_CHANNELS_JSON_PTR: &str = "/Equipment/CTRL/Settings/PWB/ch_force";
/// JSON pointer that identifies the PWB data suppression threshold in the ODB.
// This is the same for all Reset, FPN, and Pad channels.
pub const PWB_SUPPRESSION_THRESHOLD_JSON_PTR: &str = "/Equipment/CTRL/Settings/PWB/ch_threshold";
/// JSON pointer that identifies the pulser trigger flag in the ODB.
pub const TRIGGER_PULSER_JSON_PTR: &str = "/Equipment/CTRL/Settings/TrigSrc/TrigPulser";
/// JSON pointer that identifies the trigger sources in the ODB.
pub const TRIGGER_SOURCES_JSON_PTR: &str = "/Equipment/CTRL/Settings/TrigSrc";

/// The error type returned when conversion from unsigned integer to [`EventId`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to EventId")]
pub struct TryEventIdFromUnsignedError {
    input: u16,
}

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
            _ => Err(TryEventIdFromUnsignedError { input: num }),
        }
    }
}

/// The error type returned when parsing an Alpha16 bank name fails.
#[derive(Error, Debug)]
pub enum ParseAlpha16BankNameError {
    /// Input string pattern doesn't match expected Alpha16 bank name pattern.
    #[error("input string `{input}` doesn't match Alpha16BankName pattern")]
    PatternMismatch { input: String },
    /// Board name doesn't match any known [`BoardId`].
    ///
    /// [`BoardId`]: crate::alpha16::BoardId
    #[error("unknown board id")]
    UnknownBoardId(#[from] crate::alpha16::ParseBoardIdError),
    /// The representation of the Channel ID doesn't match any known
    /// [`ChannelId`].
    #[error("unknown channel id")]
    UnknownChannelId(#[from] ParseIntError),
}

/// Name of a MIDAS bank with data from SiPMs of the Barrel Veto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc16BankName {
    board_id: crate::alpha16::BoardId,
    channel_id: Adc16ChannelId,
}
impl Adc16BankName {
    /// Return the [`BoardId`] associated with the bank name.
    ///
    /// [`BoardId`]: crate::alpha16::BoardId
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # fn main() -> Result<(), ParseAlpha16BankNameError> {
    /// use alpha_g_detector::midas::Adc16BankName;
    /// use alpha_g_detector::alpha16::BoardId;
    ///
    /// let bank_name = Adc16BankName::try_from("B09F")?;
    /// let board_id = BoardId::try_from("09")?;
    ///
    /// assert_eq!(bank_name.board_id(), board_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> crate::alpha16::BoardId {
        self.board_id
    }
    /// Return the [`ChannelId`] associated with a bank name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use alpha_g_detector::midas::Adc16BankName;
    /// use alpha_g_detector::alpha16::Adc16ChannelId;
    ///
    /// let bank_name = Adc16BankName::try_from("B09F")?;
    /// let channel_id = Adc16ChannelId::try_from(15)?;
    ///
    /// assert_eq!(bank_name.channel_id(), channel_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_id(&self) -> Adc16ChannelId {
        self.channel_id
    }
}
impl TryFrom<&str> for Adc16BankName {
    type Error = ParseAlpha16BankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if !name.starts_with('B')
            || name.len() != 4
            || !name.chars().all(|c| c.is_ascii_alphanumeric())
            || name.chars().any(|c| c.is_ascii_lowercase())
        {
            return Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            });
        }
        let board_id = crate::alpha16::BoardId::try_from(&name[1..][..2])?;
        let channel_id = Adc16ChannelId::try_from(u8::from_str_radix(&name[3..], 16)?).unwrap();
        Ok(Adc16BankName {
            board_id,
            channel_id,
        })
    }
}

/// Name of a MIDAS bank with data from anode wires in the radial Time
/// Projection Chamber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc32BankName {
    board_id: crate::alpha16::BoardId,
    channel_id: Adc32ChannelId,
}
impl Adc32BankName {
    /// Return the [`BoardId`] associated with the bank name.
    ///
    /// [`BoardId`]: crate::alpha16::BoardId
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # fn main() -> Result<(), ParseAlpha16BankNameError> {
    /// use alpha_g_detector::midas::Adc32BankName;
    /// use alpha_g_detector::alpha16::BoardId;
    ///
    /// let bank_name = Adc32BankName::try_from("C09F")?;
    /// let board_id = BoardId::try_from("09")?;
    ///
    /// assert_eq!(bank_name.board_id(), board_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> crate::alpha16::BoardId {
        self.board_id
    }
    /// Return the [`ChannelId`] associated with a bank name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use alpha_g_detector::midas::Adc32BankName;
    /// use alpha_g_detector::alpha16::Adc32ChannelId;
    ///
    /// let bank_name = Adc32BankName::try_from("C09F")?;
    /// let channel_id = Adc32ChannelId::try_from(15)?;
    ///
    /// assert_eq!(bank_name.channel_id(), channel_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_id(&self) -> Adc32ChannelId {
        self.channel_id
    }
}
impl TryFrom<&str> for Adc32BankName {
    type Error = ParseAlpha16BankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if !name.starts_with('C')
            || name.len() != 4
            || !name.chars().all(|c| c.is_ascii_alphanumeric())
            || name.chars().any(|c| c.is_ascii_lowercase())
        {
            return Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            });
        }
        let board_id = crate::alpha16::BoardId::try_from(&name[1..][..2])?;
        let channel_id = Adc32ChannelId::try_from(u8::from_str_radix(&name[3..], 32)?).unwrap();
        Ok(Adc32BankName {
            board_id,
            channel_id,
        })
    }
}

/// Name of a MIDAS bank with data from an Alpha16 DAQ board.
#[derive(Clone, Copy, Debug)]
pub enum Alpha16BankName {
    /// Barrel Veto SiPM bank name.
    A16(Adc16BankName),
    /// Radial Time Projection anode wire bank name.
    A32(Adc32BankName),
}
impl TryFrom<&str> for Alpha16BankName {
    type Error = ParseAlpha16BankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match name.chars().next() {
            Some('C') => Ok(Self::A32(Adc32BankName::try_from(name)?)),
            Some('B') => Ok(Self::A16(Adc16BankName::try_from(name)?)),
            _ => Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            }),
        }
    }
}
impl Alpha16BankName {
    /// Return the [`BoardId`] associated with the bank name.
    ///
    /// [`BoardId`]: crate::alpha16::BoardId
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # fn main() -> Result<(), ParseAlpha16BankNameError> {
    /// use alpha_g_detector::midas::Alpha16BankName;
    /// use alpha_g_detector::alpha16::BoardId;
    ///
    /// let bank_name = Alpha16BankName::try_from("B09F")?;
    /// let board_id = BoardId::try_from("09")?;
    ///
    /// assert_eq!(bank_name.board_id(), board_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> crate::alpha16::BoardId {
        match self {
            Self::A16(name) => name.board_id,
            Self::A32(name) => name.board_id,
        }
    }
    /// Return the [`ChannelId`] associated with a bank name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParseAlpha16BankNameError;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use alpha_g_detector::midas::Alpha16BankName;
    /// use alpha_g_detector::alpha16::{ChannelId, Adc16ChannelId};
    ///
    /// let bank_name = Alpha16BankName::try_from("B09F")?;
    /// if let ChannelId::A16(channel) = bank_name.channel_id() {
    ///     assert_eq!(channel, Adc16ChannelId::try_from(15)?);
    /// };
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_id(&self) -> ChannelId {
        match self {
            Self::A16(name) => ChannelId::A16(name.channel_id),
            Self::A32(name) => ChannelId::A32(name.channel_id),
        }
    }
}

/// The error type returned when parsing a PadWing bank name fails.
#[derive(Error, Debug)]
pub enum ParsePadwingBankNameError {
    /// Input string pattern doesn't match the expected PadWing bank name
    /// pattern.
    #[error("input string `{input}` doesn't match PadwingBankName pattern")]
    PatternMismatch { input: String },
    /// Board name doesn't match any known [`BoardId`].
    ///
    /// [`BoardId`]: crate::padwing::BoardId
    #[error("unknown board id")]
    UnknownBoardId(#[from] crate::padwing::ParseBoardIdError),
}

/// Name of a MIDAS bank with data from cathode pads of the radial Time
/// Projection Chamber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadwingBankName {
    board_id: crate::padwing::BoardId,
}
impl PadwingBankName {
    /// Return the [`BoardId`] associated with the bank name.
    ///
    /// [`BoardId`]: crate::padwing::BoardId
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::midas::ParsePadwingBankNameError;
    /// # fn main() -> Result<(), ParsePadwingBankNameError> {
    /// use alpha_g_detector::midas::PadwingBankName;
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let bank_name = PadwingBankName::try_from("PC00")?;
    /// let board_id = BoardId::try_from("00")?;
    ///
    /// assert_eq!(bank_name.board_id(), board_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> crate::padwing::BoardId {
        self.board_id
    }
}
impl TryFrom<&str> for PadwingBankName {
    type Error = ParsePadwingBankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if !name.starts_with("PC")
            || name.len() != 4
            || !name[2..].chars().all(|c| c.is_ascii_digit())
        {
            return Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            });
        }
        let board_id = crate::padwing::BoardId::try_from(&name[2..])?;
        Ok(PadwingBankName { board_id })
    }
}

/// The error type returned when parsing a Trigger bank name fails.
#[derive(Error, Debug)]
pub enum ParseTriggerBankNameError {
    /// Input string pattern doesn't match the expected Trigger bank name
    /// pattern.
    #[error("input string `{input}` doesn't match TriggerBankName pattern")]
    PatternMismatch { input: String },
}

/// Name of a MIDAS bank with data from the Trigger board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerBankName;
impl TryFrom<&str> for TriggerBankName {
    type Error = ParseTriggerBankNameError;
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if name != "ATAT" {
            return Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            });
        }
        Ok(TriggerBankName)
    }
}

/// The error type returned when parsing a TRB3 bank name fails.
#[derive(Error, Debug)]
pub enum ParseTrb3BankNameError {
    /// Input string pattern doesn't match the expected TRB3 bank name
    /// pattern.
    #[error("input string `{input}` doesn't match Trb3BankName pattern")]
    PatternMismatch { input: String },
}

/// Name of a MIDAS bank with data from the TRB3 board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trb3BankName;
impl TryFrom<&str> for Trb3BankName {
    type Error = ParseTrb3BankNameError;
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if name != "TRBA" {
            return Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            });
        }
        Ok(Trb3BankName)
    }
}

/// The error type returned when parsing a main event bank name fails.
#[derive(Error, Debug)]
pub enum ParseMainEventBankNameError {
    /// Input string pattern doesn't match any main event bank name pattern.
    #[error("input string `{input}` doesn't match any main event bank name pattern")]
    PatternMismatch { input: String },
    /// Input string partially matches an [`Alpha16BankName`] pattern but
    /// doesn't match the full pattern.
    #[error("bad alpha16 bank name")]
    BadAlpha16(#[from] ParseAlpha16BankNameError),
    /// Input string partially matches a [`PadwingBankName`] pattern but doesn't
    /// match the full pattern.
    #[error("bad padwing bank name")]
    BadPadwing(#[from] ParsePadwingBankNameError),
    /// Input string partially matches a [`TriggerBankName`] pattern but doesn't
    /// match the full pattern.
    #[error("bad trigger bank name")]
    BadTrigger(#[from] ParseTriggerBankNameError),
    /// Input string partially matches a [`Trb3BankName`] pattern but doesn't
    /// match the full pattern.
    #[error("bad trb3 bank name")]
    BadTrb3(#[from] ParseTrb3BankNameError),
}

/// Name of a MIDAS bank in a main event (i.e. with an event id [`EventId::Main`]).
#[derive(Clone, Copy, Debug)]
pub enum MainEventBankName {
    /// Name of a MIDAS bank with data from the Alpha16 board.
    Alpha16(Alpha16BankName),
    /// Name of a MIDAS bank with data from the cathode pads of the radial Time
    /// Projection Chamber.
    Padwing(PadwingBankName),
    /// Name of a MIDAS bank with data from the Trigger board.
    Trg(TriggerBankName),
    /// Name of a MIDAS bank with data from the TRB3 board.
    Trb3(Trb3BankName),
}
impl TryFrom<&str> for MainEventBankName {
    type Error = ParseMainEventBankNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match name.chars().next() {
            Some('A') => Ok(Self::Trg(TriggerBankName::try_from(name)?)),
            Some('B' | 'C') => Ok(Self::Alpha16(Alpha16BankName::try_from(name)?)),
            Some('P') => Ok(Self::Padwing(PadwingBankName::try_from(name)?)),
            Some('T') => Ok(Self::Trb3(Trb3BankName::try_from(name)?)),
            _ => Err(Self::Error::PatternMismatch {
                input: name.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests;
