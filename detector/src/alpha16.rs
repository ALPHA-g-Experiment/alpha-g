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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// The input slice is not long enough to contain a complete packet.
    IncompleteSlice,
    /// Unknown packet type. This value should currently be fixed to `1`.
    UnknownType,
    /// Unknown packet version. This value should currently be fixed to `3`.
    UnknownVersion,
    /// Integer representation of Module ID doesn't match any known
    /// [`ModuleId`].
    UnknownModuleId,
    /// Integer representation of channel ID doesn't match any known
    /// [`ChannelId`].
    UnknownChannelId,
    /// Non-zero value found in bytes meant to be fixed to `0`.
    ZeroMismatch,
    /// MAC address doesn't map to any known [`BoardId`].
    UnknownMac,
    /// Suppression baseline in the footer doesn't match waveform samples.
    BaselineMismatch,
    /// The number of waveform samples is less than the number indicated by
    /// `keep_last` in the footer.
    // The `keep_more` and `threshold` values are not known here, so a more
    // specific error than this is not possible.
    BadKeepLast,
    /// The `keep_bit` in the footer is set, but there are no waveform samples.
    /// Or `keep_bit` is not set, but there are waveform samples.
    // The `threshold` is not known here, so a more specific error than this is
    // not possible.
    KeepBitMismatch,
    /// Data suppression is not enabled, and the number of waveform samples
    /// doesn't match the number of requested samples. Or data suppression is
    /// enabled, and the number of waveform samples is greater than the number
    /// of requested samples.
    // The `keep_more` is not known here, so a more specific error than this is
    // not possible.
    NumberOfSamplesMismatch,
    /// The number of waveform bytes is not even; recall that each waveform
    /// sample is [`i16`]. Or the number of waveform samples is shorter than the
    /// minimum required to reproduce the suppression baseline.
    IncompleteWaveform,
}
impl fmt::Display for TryAdcPacketFromSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryAdcPacketFromSliceError::IncompleteSlice => write!(f, "short input slice"),
            TryAdcPacketFromSliceError::UnknownType => write!(f, "unknown packet type"),
            TryAdcPacketFromSliceError::UnknownVersion => write!(f, "unknown packet version"),
            TryAdcPacketFromSliceError::UnknownModuleId => write!(f, "unknown module id"),
            TryAdcPacketFromSliceError::UnknownChannelId => write!(f, "unknown channel number"),
            TryAdcPacketFromSliceError::ZeroMismatch => write!(f, "zero-bytes mismatch"),
            TryAdcPacketFromSliceError::UnknownMac => write!(f, "unknown mac address"),
            TryAdcPacketFromSliceError::BaselineMismatch => {
                write!(f, "suppression baseline mismatch")
            }
            TryAdcPacketFromSliceError::BadKeepLast => write!(f, "bad keep_last"),
            TryAdcPacketFromSliceError::KeepBitMismatch => write!(f, "keep_bit mismatch"),
            TryAdcPacketFromSliceError::NumberOfSamplesMismatch => {
                write!(f, "number of samples mismatch")
            }
            TryAdcPacketFromSliceError::IncompleteWaveform => write!(f, "incomplete waveform"),
        }
    }
}
impl Error for TryAdcPacketFromSliceError {}
impl From<TryModuleIdFromUnsignedError> for TryAdcPacketFromSliceError {
    fn from(_: TryModuleIdFromUnsignedError) -> Self {
        Self::UnknownModuleId
    }
}
impl From<TryChannelIdFromUnsignedError> for TryAdcPacketFromSliceError {
    fn from(_: TryChannelIdFromUnsignedError) -> Self {
        Self::UnknownChannelId
    }
}
impl From<TryBoardIdFromMacAddressError> for TryAdcPacketFromSliceError {
    fn from(_: TryBoardIdFromMacAddressError) -> Self {
        Self::UnknownMac
    }
}

/// Version 3 of an ADC data packet.
///
/// An ADC packet represents the data collected from an individual channel in an
/// Alpha16 board. The binary representation of an [`AdcV3Packet`] in a data
/// bank is shown below. All multi-byte fields are big-endian:
///
/// <center>
///
/// |Byte(s)|Description|
/// |:-:|:-:|
/// |0|Fixed to 1|
/// |1|Fixed to 3|
/// |2-3|Accepted trigger|
/// |4|Module ID|
/// |5|Channel ID|
/// |6-7|Requested samples|
/// |8-11|Event timestamp (LSW)|
/// |12-13|Fixed to 0|
/// |14-19|MAC address|
/// |20-23|Event timestamp (MSW)|
/// |24-27|Trigger offset|
/// |28-31|Build timestamp|
/// |32-33|First waveform sample|
/// |...|Waveform samples|
/// |Last 4 bytes|Data suppression info|
///
/// </center>
///
/// Bytes `[12..size - 4]` are only included in the packet if the `keep_bit` is
/// set after data suppression.
#[derive(Clone, Debug)]
pub struct AdcV3Packet {
    accepted_trigger: u16,
    module_id: ModuleId,
    channel_id: ChannelId,
    requested_samples: usize,
    event_timestamp: u64,
    board_id: Option<BoardId>,
    trigger_offset: Option<i32>,
    build_timestamp: Option<u32>,
    waveform: Vec<i16>,
    // Only `keep_bit` and `baseline` can be deduced by logic from the other
    // fields. Both `keep_last` and `suppression_enabled` need to be stored.
    keep_last: Option<usize>,
    suppression_enabled: bool,
}

impl AdcV3Packet {
    /// Return the packet type. Currently fixed to `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.packet_type(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn packet_type(&self) -> u8 {
        1
    }
    /// Return the packet version. For [`AdcV3Packet`] it is fixed to `3`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.packet_version(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn packet_version(&self) -> u8 {
        3
    }
    /// In the firmware logic, `accepted_trigger` is a 32-bits unsigned integer.
    /// Return the 16 LSB as [`u16`].
    ///
    /// This is a counter that indicates the number of trigger signals received
    /// from the TRG board. All packets from the same event must have the same
    /// `accepted_trigger` counter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.accepted_trigger(), 4);
    /// # Ok(())
    /// # }
    /// ```
    pub fn accepted_trigger(&self) -> u16 {
        self.accepted_trigger
    }
    /// Return the [`ModuleId`] of the Alpha16 board from which the packet was
    /// generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::{AdcV3Packet, ModuleId};
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.module_id(), ModuleId::try_from(5)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
    /// Return the [`ChannelId`] in an Alpha16 board from which the packet was
    /// generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::{AdcV3Packet, ChannelId};
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(matches!(packet.channel_id(), ChannelId::A16(_)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
    /// Return the number of requested waveform samples. The actual number of
    /// samples in the packet should be obtained from [`waveform`]; due to data
    /// suppression these two are most likely not equal.
    ///
    /// [`waveform`]: AdcV3Packet::waveform.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.requested_samples(), 699);
    /// # Ok(())
    /// # }
    /// ```
    pub fn requested_samples(&self) -> usize {
        self.requested_samples
    }
    /// I do not know what this field means. It never matches the event
    /// timestamp in the MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.event_timestamp(), 7);
    /// # Ok(())
    /// # }
    /// ```
    pub fn event_timestamp(&self) -> u64 {
        self.event_timestamp
    }
    /// Return the [`BoardId`] of the Alpha16 board from which the packet was
    /// generated. Return [`None`] if the `keep_bit` is not set in the data
    /// suppression footer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.board_id().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> Option<BoardId> {
        self.board_id
    }
    /// I do not understand what this field means exactly. I know that it
    /// matches `adcXX_trig_delay - adcXX_trig_start` in the ODB (with `XX`
    /// equal to `16` or `32`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.trigger_offset().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn trigger_offset(&self) -> Option<i32> {
        self.trigger_offset
    }
    /// Return the SOF file build timestamp; this acts as firmware version.
    /// Return [`None`] if the `keep_bit` is not set in the data suppression
    /// footer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.build_timestamp().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_timestamp(&self) -> Option<u32> {
        self.build_timestamp
    }
    /// Return the digitized waveform samples received by an ADC channel in an
    /// Alpha16 board. Return an empty slice if the `keep_bit` is not set in the
    /// data suppression footer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.waveform().len(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn waveform(&self) -> &[i16] {
        &self.waveform
    }
    /// Return the waveform baseline used in data suppression. Return [`None`]
    /// if there are no waveform samples i.e. the `keep_bit` is not set in the
    /// data suppression footer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.suppression_baseline().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn suppression_baseline(&self) -> Option<i16> {
        if self.waveform.is_empty() {
            None
        } else {
            // Add over i32 because adding 64 i16s can overflow.
            Some(
                i16::try_from(
                    self.waveform[..64]
                        .iter()
                        .map(|n| i32::from(*n))
                        .sum::<i32>()
                        / 64,
                )
                .unwrap(),
            )
        }
    }
    /// Return a counter of the last data word with a sample over the data
    /// suppression threshold. The `keep_last` value is computed from the last
    /// waveform `index` as `keep_last = (index + 1) / 2 + 1`. Hence the `index`
    /// of the last waveform sample above the suppression threshold is either
    /// `(keep_last - 1) * 2` or `(keep_last - 1) * 2 - 1`. Return [`None`] if
    /// the `keep_bit` is not set in the data suppression footer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.keep_last().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn keep_last(&self) -> Option<usize> {
        self.keep_last
    }
    /// Return [`true`] if data suppression is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.is_suppression_enabled());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_suppression_enabled(&self) -> bool {
        self.suppression_enabled
    }
}

impl TryFrom<&[u8]> for AdcV3Packet {
    type Error = TryAdcPacketFromSliceError;

    // All fields are big endian
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < 16 {
            return Err(Self::Error::IncompleteSlice);
        }

        if slice[0] != 1 {
            return Err(Self::Error::UnknownType);
        }
        if slice[1] != 3 {
            return Err(Self::Error::UnknownVersion);
        }
        let accepted_trigger = slice[2..4].try_into().unwrap();
        let accepted_trigger = u16::from_be_bytes(accepted_trigger);
        let module_id = ModuleId::try_from(slice[4])?;
        // A value of [0-15] is BV, and a value of [128-159] is rTPC
        let channel_id = slice[5];
        let channel_id = if channel_id < 128 {
            ChannelId::A16(channel_id.try_into()?)
        } else {
            ChannelId::A32((channel_id - 128).try_into()?)
        };
        let requested_samples = slice[6..8].try_into().unwrap();
        let requested_samples = u16::from_be_bytes(requested_samples).into();
        let lsw_event_timestamp = slice[8..12].try_into().unwrap();

        let baseline = slice[slice.len() - 2..].try_into().unwrap();
        let baseline = i16::from_be_bytes(baseline);
        let footer = slice[slice.len() - 4..][..2].try_into().unwrap();
        let footer = u16::from_be_bytes(footer);
        let keep_last = usize::from(footer & 0xFFF);
        let keep_bit = (footer >> 12) & 1 == 1;
        let suppression_enabled = (footer >> 13) & 1 == 1;

        if slice.len() == 16 {
            if !suppression_enabled {
                return Err(Self::Error::NumberOfSamplesMismatch);
            }
            if keep_bit {
                return Err(Self::Error::KeepBitMismatch);
            }
            if keep_last != 0 {
                return Err(Self::Error::BadKeepLast);
            }
            return Ok(AdcV3Packet {
                accepted_trigger,
                module_id,
                channel_id,
                requested_samples,
                event_timestamp: u32::from_be_bytes(lsw_event_timestamp).into(),
                board_id: None,
                trigger_offset: None,
                build_timestamp: None,
                waveform: Vec::new(),
                keep_last: None,
                suppression_enabled,
            });
        }

        if slice.len() < 36 {
            return Err(Self::Error::IncompleteSlice);
        }

        if slice[12..14] != [0, 0] {
            return Err(Self::Error::ZeroMismatch);
        }
        let board_id: [u8; 6] = slice[14..20].try_into().unwrap();
        let board_id = BoardId::try_from(board_id)?;
        let msw_event_timestamp = slice[20..24].try_into().unwrap();
        let event_timestamp = [msw_event_timestamp, lsw_event_timestamp].concat();
        let event_timestamp = event_timestamp.try_into().unwrap();
        let event_timestamp = u64::from_be_bytes(event_timestamp);
        let trigger_offset = slice[24..28].try_into().unwrap();
        let trigger_offset = i32::from_be_bytes(trigger_offset);
        let build_timestamp = slice[28..32].try_into().unwrap();
        let build_timestamp = u32::from_be_bytes(build_timestamp);

        let waveform_bytes = slice.len() - 36;
        // We need 64 waveform samples i.e. 128 bytes to reconstruct and check
        // that the baseline is correct.
        if waveform_bytes % 2 != 0 || waveform_bytes < 128 {
            return Err(Self::Error::IncompleteWaveform);
        }
        if !keep_bit {
            return Err(Self::Error::KeepBitMismatch);
        }
        let waveform_samples = waveform_bytes / 2;
        if suppression_enabled && waveform_samples > requested_samples {
            return Err(Self::Error::NumberOfSamplesMismatch);
        }
        if !suppression_enabled && waveform_samples != requested_samples {
            return Err(Self::Error::NumberOfSamplesMismatch);
        }
        // The keep_bit is computed from the last waveform index over the
        // threshold as: keep_last = (index + 1) / 2 + 1.
        // Then the index of the last sample over the threshold is:
        // (keep_last - 1) * 2
        // or
        // (keep_last - 1) * 2 - 1
        // It can be any of the above (due to integer division rounding). Then,
        // this implies that the number of waveform samples has to be at least
        // (keep_last - 1) * 2 - 1 or bigger. We rearrange the inequality to
        // avoid unsigned integer underflow.
        // This also removes the trivially wrong keep_last = {0,1} from which
        // you cant reconstruct the index without risking underflow.
        if waveform_samples + 3 < 2 * keep_last {
            return Err(Self::Error::BadKeepLast);
        }

        let waveform: Vec<i16> = slice[32..][..waveform_bytes]
            .chunks_exact(2)
            .map(|b| i16::from_be_bytes(b.try_into().unwrap()))
            .collect();

        if i32::from(baseline) != waveform[..64].iter().map(|n| i32::from(*n)).sum::<i32>() / 64 {
            return Err(Self::Error::BaselineMismatch);
        }

        Ok(AdcV3Packet {
            accepted_trigger,
            module_id,
            channel_id,
            requested_samples,
            event_timestamp,
            board_id: Some(board_id),
            trigger_offset: Some(trigger_offset),
            build_timestamp: Some(build_timestamp),
            waveform,
            keep_last: Some(keep_last),
            suppression_enabled,
        })
    }
}

#[cfg(test)]
mod tests;
