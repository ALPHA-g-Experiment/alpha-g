use std::{error::Error, fmt};

// Sampling rate (samples per second) of the ADC channels that receive the
// Barrel Veto SiPM signals.
const ADC16RATE: f64 = 100e6;

// Sampling rate (samples per second) of the ADC channels that receive the
// radial Time Projection Chamber anode wire signals.
const ADC32RATE: f64 = 62.5e6;

/// The error type returned when conversion from unsigned integer to
/// [`ChannelId`] fails.
#[derive(Clone, Copy, Debug)]
pub struct TryChannelIdFromUnsignedError;
impl fmt::Display for TryChannelIdFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown channel number attempted")
    }
}
impl Error for TryChannelIdFromUnsignedError {}

/// Channel ID that corresponds to SiPMs of the Barrel Veto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc16ChannelId(u8);
impl TryFrom<u8> for Adc16ChannelId {
    type Error = TryChannelIdFromUnsignedError;

    /// There are 16 valid channel ids. Perform the conversion from an integer
    /// in range `0..=15`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num > 15 {
            Err(TryChannelIdFromUnsignedError)
        } else {
            Ok(Adc16ChannelId(num))
        }
    }
}
impl Adc16ChannelId {
    /// Sampling rate of the ADC channel in samples per second.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use detector::alpha16::Adc16ChannelId;
    ///
    /// let channel = Adc16ChannelId::try_from(0)?;
    /// assert_eq!(channel.sampling_rate(), 100e6);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sampling_rate(&self) -> f64 {
        ADC16RATE
    }
}

/// Channel ID that corresponds to anode wires in the radial Time Projection
/// Chamber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc32ChannelId(u8);
impl TryFrom<u8> for Adc32ChannelId {
    type Error = TryChannelIdFromUnsignedError;

    /// There are 32 valid channel ids. Perform the conversion from an integer
    /// in range `0..=31`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num > 31 {
            Err(TryChannelIdFromUnsignedError)
        } else {
            Ok(Adc32ChannelId(num))
        }
    }
}
impl Adc32ChannelId {
    /// Sampling rate of the ADC channel in samples per second.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use detector::alpha16::Adc32ChannelId;
    ///
    /// let channel = Adc32ChannelId::try_from(0)?;
    /// assert_eq!(channel.sampling_rate(), 62.5e6);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sampling_rate(&self) -> f64 {
        ADC32RATE
    }
}

/// ADC channel ID in an Alpha16 board.
#[derive(Clone, Copy, Debug)]
pub enum ChannelId {
    /// Barrel Veto SiPM channel.
    A16(Adc16ChannelId),
    /// Radial Time Projection Chamber anode wire channel.
    A32(Adc32ChannelId),
}
impl ChannelId {
    /// Sampling rate of the ADC channel in samples per second.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use detector::alpha16::{Adc32ChannelId, ChannelId};
    ///
    /// let aw_channel = Adc32ChannelId::try_from(0)?;
    /// let channel = ChannelId::A32(aw_channel);
    ///
    /// assert_eq!(channel.sampling_rate(), 62.5e6);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sampling_rate(&self) -> f64 {
        match self {
            ChannelId::A16(channel) => channel.sampling_rate(),
            ChannelId::A32(channel) => channel.sampling_rate(),
        }
    }
}
// There is not TryFrom implementation because there is not an unambiguous
// integer representation for both channels at the same time.
// Agana uses some times [0-47] with [0-15] BV and [16-47] TPC. In other places
// it uses [0-15] BV and [128-159] TPC. Avoid that mess here.

/// The error type returned when conversion from unsigned integer to
/// [`ModuleId`] fails.
#[derive(Clone, Copy, Debug)]
pub struct TryModuleIdFromUnsignedError;
impl fmt::Display for TryModuleIdFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown module number attempted")
    }
}
impl Error for TryModuleIdFromUnsignedError {}

/// Module ID of an Alpha16 board.
///
/// It is important to notice that a [`BoardId`] is different to a [`ModuleId`].
/// The former identifies a physical Alpha16 board, while the latter is a fixed
/// ID that maps a module to BV and TPC channels. The mapping between
/// [`BoardId`] and [`ModuleId`] depends on the run number e.g. we switch an old
/// board for a new board. You can see the [`ModuleId`] as the slot in which a
/// board is plugged, which always maps to the same BV and TPC channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleId(u8);
impl TryFrom<u8> for ModuleId {
    type Error = TryModuleIdFromUnsignedError;

    /// There are 8 valid module ids. Perform the conversion from an integer
    /// in range `0..=7`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num > 7 {
            Err(TryModuleIdFromUnsignedError)
        } else {
            Ok(ModuleId(num))
        }
    }
}

/// The error type returned when conversion from mac address to [`BoardId`]
/// fails.
#[derive(Clone, Copy, Debug)]
pub struct TryBoardIdFromMacAddressError;
impl fmt::Display for TryBoardIdFromMacAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown mac address attempted")
    }
}
impl Error for TryBoardIdFromMacAddressError {}

// Known Alpha16 board names and mac addresses
// Just add new boards to this list
// ("name", [mac address])
// "name" is 2 ASCII characters that also appear in the data bank name
const ALPHA16BOARDS: [(&str, [u8; 6]); 8] = [
    ("09", [216, 128, 57, 104, 55, 76]),
    ("10", [216, 128, 57, 104, 170, 37]),
    ("11", [216, 128, 57, 104, 172, 127]),
    ("12", [216, 128, 57, 104, 79, 167]),
    ("13", [216, 128, 57, 104, 202, 166]),
    ("14", [216, 128, 57, 104, 142, 130]),
    ("16", [216, 128, 57, 104, 111, 162]),
    ("18", [216, 128, 57, 104, 142, 82]),
];

/// Identity of a physical Alpha16 board.
///
/// It is important to notice that a [`BoardId`] is different to a [`ModuleId`].
/// The former identifies a physical Alpha16 board, while the latter is a fixed
/// ID that maps a module to BV and TPC channels. The mapping between
/// [`BoardId`] and [`ModuleId`] depends on the run number e.g. we switch an old
/// board for a new board. You can see the [`ModuleId`] as the slot in which a
/// board is plugged, which always maps to the same BV and TPC channels.
#[derive(Clone, Copy, Debug)]
pub struct BoardId {
    name: &'static str,
    mac_address: [u8; 6],
}
impl TryFrom<[u8; 6]> for BoardId {
    type Error = TryBoardIdFromMacAddressError;

    fn try_from(mac: [u8; 6]) -> Result<Self, Self::Error> {
        for pair in ALPHA16BOARDS {
            if mac == pair.1 {
                return Ok(BoardId {
                    name: pair.0,
                    mac_address: pair.1,
                });
            }
        }
        Err(TryBoardIdFromMacAddressError)
    }
}
impl BoardId {
    /// Return the name of a physical Alpha16 board. This is a human readable
    /// name used to identify a board instead of the mac address.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryBoardIdFromMacAddressError;
    /// # fn main() -> Result<(), TryBoardIdFromMacAddressError> {
    /// use detector::alpha16::BoardId;
    ///
    /// let board_id = BoardId::try_from([216, 128, 57, 104, 142, 82])?;
    /// assert_eq!(board_id.name(), "18");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        self.name
    }
    /// Return the mac address of a physical Alpha16 board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryBoardIdFromMacAddressError;
    /// # fn main() -> Result<(), TryBoardIdFromMacAddressError> {
    /// use detector::alpha16::BoardId;
    ///
    /// let board_id = BoardId::try_from([216, 128, 57, 104, 142, 82])?;
    /// assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 142, 82]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }
}

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to
/// [`AdcPacket`] fails.
#[derive(Clone, Copy, Debug)]
pub enum TryAdcPacketFromSliceError {
    /// Unknown packet type. This value should currently be fixed to `1`.
    UnknownType,
    /// Unknown packet version. This value should currently be fixed to `3`.
    UnknownVersion,
    /// Integer representation of Module ID doesn't match any known Alpha16.
    /// board.
    UnknownModuleId,
    /// Unknown channel in the Alpha16 board.
    UnknownChannelId,
    /// Non-zero value found in bytes meant to be fixed to 0.
    ZeroMismatch,
    /// MAC address doesn't match any known Alpha16 board.
    UnknownMac,
    /// Suppression baseline in the footer doesn't match waveform samples.
    BaselineMismatch,
    /// The number of waveform samples is less than the number indicated by
    /// `keep_last` in the footer.
    BadKeepLast,
    /// The `keep_bit` in the footer is set, but there are no waveform samples.
    /// Or `keep_bit` is not set, but there are waveform samples.
    KeepBitMismatch,
    /// Data suppression is not enabled, and the number of waveform samples
    /// doesn't match the number of requested samples.
    RequestedSamplesMismatch,
    // Is there a failure mode from number of samples requested? e.g. there are
    // more samples than the requested number.
}
impl fmt::Display for TryAdcPacketFromSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryAdcPacketFromSliceError::UnknownType => write!(f, "unknown packet type"),
            TryAdcPacketFromSliceError::UnknownVersion => write!(f, "unknown packet version"),
            TryAdcPacketFromSliceError::UnknownModuleId => write!(f, "unknown module id"),
            TryAdcPacketFromSliceError::UnknownChannelId => write!(f, "unknown channel number"),
            TryAdcPacketFromSliceError::ZeroMismatch => write!(f, "zero-bytes mismatch"),
            TryAdcPacketFromSliceError::UnknownMac => write!(f, "unknown mac address"),
            TryAdcPacketFromSliceError::BaselineMismatch => write!(f, "baseline mismatch"),
            TryAdcPacketFromSliceError::BadKeepLast => write!(f, "bad keep_last"),
            TryAdcPacketFromSliceError::KeepBitMismatch => write!(f, "keep_bit mismatch"),
            TryAdcPacketFromSliceError::RequestedSamplesMismatch => {
                write!(f, "requested number of samples mismatch")
            }
        }
    }
}
impl Error for TryAdcPacketFromSliceError {}
impl From<TryChannelIdFromUnsignedError> for TryAdcPacketFromSliceError {
    fn from(_: TryChannelIdFromUnsignedError) -> Self {
        Self::UnknownChannelId
    }
}

#[derive(Clone, Debug)]
pub struct AdcV3Packet {
    accepted_trigger: u16,
    module_id: ModuleId,
    channel_id: ChannelId,
    samples_requested: usize,
    event_timestamp: u64,
    board_id: Option<BoardId>,
    trigger_offset: Option<u32>,
    build_timestamp: Option<u32>,
    waveform: Vec<i16>,
    keep_last: usize,
    suppression_enabled: bool,
}

impl AdcV3Packet {
    pub fn packet_type(&self) -> u8 {
        1
    }
    pub fn packet_version(&self) -> u8 {
        3
    }
    pub fn accepted_trigger(&self) -> u16 {
        self.accepted_trigger
    }
    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
    pub fn samples_requested(&self) -> usize {
        self.samples_requested
    }
    pub fn event_timestamp(&self) -> u64 {
        self.event_timestamp
    }
    pub fn mac_address(&self) -> Option<[u8; 6]> {
        self.board_id.map(|board| board.mac_address)
    }
    pub fn trigger_offset(&self) -> Option<u32> {
        self.trigger_offset
    }
    pub fn build_timestamp(&self) -> Option<u32> {
        self.build_timestamp
    }
    pub fn waveform(&self) -> &[i16] {
        &self.waveform
    }
    pub fn baseline(&self) -> i16 {
        todo!()
    }
    pub fn is_suppression_enabled(&self) -> bool {
        self.suppression_enabled
    }
    pub fn keep_last(&self) -> usize {
        self.keep_last
    }
}

#[cfg(test)]
mod tests;
