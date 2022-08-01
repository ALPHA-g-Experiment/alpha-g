use std::fmt;
use thiserror::Error;

// Sampling rate (samples per second) of the ADC channels that receive the
// Barrel Veto SiPM signals.
const ADC16RATE: f64 = 100e6;

// Sampling rate (samples per second) of the ADC channels that receive the
// radial Time Projection Chamber anode wire signals.
const ADC32RATE: f64 = 62.5e6;

/// The error type returned when conversion from unsigned integer to
/// [`ChannelId`] fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to ChannelId")]
pub struct TryChannelIdFromUnsignedError {
    input: u8,
}

/// Channel ID that corresponds to SiPMs of the Barrel Veto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adc16ChannelId(u8);
impl TryFrom<u8> for Adc16ChannelId {
    type Error = TryChannelIdFromUnsignedError;

    /// There are 16 valid channel ids. Perform the conversion from an integer
    /// in range `0..=15`.
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num > 15 {
            Err(TryChannelIdFromUnsignedError { input: num })
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
    /// # use alpha_g_detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use alpha_g_detector::alpha16::Adc16ChannelId;
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
            Err(TryChannelIdFromUnsignedError { input: num })
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
    /// # use alpha_g_detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use alpha_g_detector::alpha16::Adc32ChannelId;
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
    /// # use alpha_g_detector::alpha16::TryChannelIdFromUnsignedError;
    /// # fn main() -> Result<(), TryChannelIdFromUnsignedError> {
    /// use alpha_g_detector::alpha16::{Adc32ChannelId, ChannelId};
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
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to ModuleId")]
pub struct TryModuleIdFromUnsignedError {
    input: u8,
}

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
            Err(TryModuleIdFromUnsignedError { input: num })
        } else {
            Ok(ModuleId(num))
        }
    }
}

/// The error type returned when conversion from mac address to [`BoardId`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from mac address `{input:?}` to BoardId")]
pub struct TryBoardIdFromMacAddressError {
    input: [u8; 6],
}

/// The error type returned when parsing a [`BoardId`] fails.
#[derive(Error, Debug)]
#[error("unknown parsing from board name `{input}` to BoardId")]
pub struct ParseBoardIdError {
    input: String,
}

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
impl TryFrom<&str> for BoardId {
    type Error = ParseBoardIdError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        for pair in ALPHA16BOARDS {
            if name == pair.0 {
                return Ok(BoardId {
                    name: pair.0,
                    mac_address: pair.1,
                });
            }
        }
        Err(ParseBoardIdError {
            input: name.to_string(),
        })
    }
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
        Err(TryBoardIdFromMacAddressError { input: mac })
    }
}
impl BoardId {
    /// Return the name of a physical Alpha16 board. This is a human readable
    /// name used to identify a board instead of the mac address.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryBoardIdFromMacAddressError;
    /// # fn main() -> Result<(), TryBoardIdFromMacAddressError> {
    /// use alpha_g_detector::alpha16::BoardId;
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
    /// # use alpha_g_detector::alpha16::TryBoardIdFromMacAddressError;
    /// # fn main() -> Result<(), TryBoardIdFromMacAddressError> {
    /// use alpha_g_detector::alpha16::BoardId;
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
#[derive(Error, Debug)]
pub enum TryAdcPacketFromSliceError {
    /// The input slice is not long enough to contain a complete packet.
    #[error("incomplete slice (expected at least `{min_expected}` bytes, found `{found}`)")]
    IncompleteSlice { found: usize, min_expected: usize },
    /// Unknown packet type.
    #[error("unknown packet type `{found}`")]
    UnknownType { found: u8 },
    /// Unknown packet version.
    #[error("unknown packet version `{found}`")]
    UnknownVersion { found: u8 },
    /// Integer representation of Module ID doesn't match any known
    /// [`ModuleId`].
    #[error("unknown module id")]
    UnknownModuleId(#[from] TryModuleIdFromUnsignedError),
    /// Integer representation of channel ID doesn't match any known
    /// [`ChannelId`].
    #[error("unknown channel number")]
    UnknownChannelId(#[from] TryChannelIdFromUnsignedError),
    /// Non-zero value found in bytes meant to be fixed to `0`.
    #[error("zero-bytes mismatch (found `{found:?}`)")]
    ZeroMismatch { found: [u8; 2] },
    /// MAC address doesn't map to any known [`BoardId`].
    #[error("unknown mac address")]
    UnknownMac(#[from] TryBoardIdFromMacAddressError),
    /// Suppression baseline in the footer doesn't match waveform samples.
    #[error("suppression baseline mismatch (expected `{expected}`, found `{found}`)")]
    BaselineMismatch { found: i16, expected: i16 },
    /// The value of `keep_last` is inconsistent with the `keep_bit`, or its
    /// value is less than the minimum required by the suppression baseline.
    // The `keep_more` and `threshold` values are not known here, so a more
    // specific error than this is not possible.
    // If limit == 0, then it is an inconsistency with the `keep_bit`
    // If limit != 0, then value is less than the limit imposed by the
    // suppression baseline.
    #[error("bad keep_last `{found}` (limit was `{limit}`)")]
    BadKeepLast { found: usize, limit: usize },
    /// The `keep_bit` in the footer is inconsistent with the packet size and
    /// data suppression status.
    // The `threshold` is not known here, so a more specific error than this is
    // not possible.
    #[error("keep_bit mismatch (found `{found}`)")]
    KeepBitMismatch { found: bool },
    /// The number of waveform samples is less/more than the minimum/maximum
    /// required by the suppression baseline, `keep_last`, or requested number
    /// of samples.
    #[error("bad number of samples `{found}` (expected at least `{min}` and at most `{max}`)")]
    BadNumberOfSamples {
        found: usize,
        min: usize,
        max: usize,
    },
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
    suppression_baseline: i16,
    keep_last: usize,
    keep_bit: bool,
    suppression_enabled: bool,
}

impl fmt::Display for AdcV3Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Packet type: {}", self.packet_type())?;
        writeln!(f, "Packet version: {}", self.packet_version())?;
        writeln!(f, "Accepted trigger: {}", self.accepted_trigger)?;
        writeln!(f, "Module ID: {:?}", self.module_id)?;
        let channel_id = match self.channel_id {
            ChannelId::A16(channel) => format!("{:?}", channel),
            ChannelId::A32(channel) => format!("{:?}", channel),
        };
        writeln!(f, "Channel ID: {channel_id}")?;
        writeln!(f, "Requested samples: {}", self.requested_samples)?;
        writeln!(f, "Event timestamp: {}", self.event_timestamp)?;
        let mac_address = self
            .board_id
            .map_or("None".to_string(), |b| format!("{:?}", b.mac_address()));
        writeln!(f, "MAC address: {mac_address}")?;
        let trigger_offset = self
            .trigger_offset
            .map_or("None".to_string(), |v| v.to_string());
        writeln!(f, "Trigger offset: {trigger_offset}",)?;
        let build_timestamp = self
            .build_timestamp
            .map_or("None".to_string(), |v| v.to_string());
        writeln!(f, "Build timestamp: {build_timestamp}",)?;
        writeln!(f, "Waveform samples: {}", self.waveform.len())?;
        writeln!(f, "Suppression baseline: {}", self.suppression_baseline)?;
        writeln!(f, "Keep last: {}", self.keep_last)?;
        writeln!(f, "Keep bit: {}", self.keep_bit)?;
        write!(f, "Suppression enabled: {}", self.suppression_enabled)?;

        Ok(())
    }
}

impl AdcV3Packet {
    /// Return the packet type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::{AdcV3Packet, ModuleId};
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::{AdcV3Packet, ChannelId};
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// generated. Return [`None`] if data suppression is enabled and the
    /// `keep_bit` is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// equal to `16` or `32`). Return [`None`] if data suppression is enabled
    /// and the `keep_bit` is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// Return [`None`] if data suppression is enabled and the `keep_bit` is not
    /// set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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
    /// Alpha16 board. Return an empty slice if data suppression is enabled and
    /// the `keep_bit` is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.waveform().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn waveform(&self) -> &[i16] {
        &self.waveform
    }
    /// Return the data suppression waveform baseline.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.suppression_baseline(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn suppression_baseline(&self) -> i16 {
        self.suppression_baseline
    }
    /// This is a counter in the firmware side on how many data words are being
    /// kept due to data suppression. If the `keep_bit` is not set, then
    /// `keep_last` is equal to 0. This counter increases by the index of the
    /// last waveform sample over threshold as `keep_last = (index + 2) / 2 + 1`.
    ///
    /// Recall that data suppression doesn't "see" the last 6(?) samples, hence
    /// `keep_last` is not a reliable way to obtain the last waveform sample
    /// over the data suppression threshold. This `keep_last` value is only
    /// really useful in validating/checking the data suppression on the
    /// firmware side. If you are using this for anything else, you are most
    /// likely making a mistake.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.keep_last(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn keep_last(&self) -> usize {
        self.keep_last
    }
    /// Return [`true`] if at least one [`waveform`] sample is over the data
    /// suppression threshold.
    ///
    /// [`waveform`]: AdcV3Packet::waveform.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(!packet.keep_bit());
    /// # Ok(())
    /// # }
    /// ```
    pub fn keep_bit(&self) -> bool {
        self.keep_bit
    }
    /// Return [`true`] if data suppression is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcV3Packet;
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

// The minimum number of samples required to reconstruct the data suppression
// baseline.
const BASELINE_SAMPLES: usize = 64;
// Minimum valid value of keep_last different to 0.
// keep_last = (index + 2) / 2 + 1
// And the minimum index is one after the baseline.
const MIN_KEEP_LAST: usize = (BASELINE_SAMPLES + 2) / 2 + 1;

impl TryFrom<&[u8]> for AdcV3Packet {
    type Error = TryAdcPacketFromSliceError;

    // All fields are big endian
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < 16 {
            return Err(Self::Error::IncompleteSlice {
                found: slice.len(),
                min_expected: 16,
            });
        }

        if slice[0] != 1 {
            return Err(Self::Error::UnknownType { found: slice[0] });
        }
        if slice[1] != 3 {
            return Err(Self::Error::UnknownVersion { found: slice[1] });
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

        let suppression_baseline = slice[slice.len() - 2..].try_into().unwrap();
        let suppression_baseline = i16::from_be_bytes(suppression_baseline);
        let footer = slice[slice.len() - 4..][..2].try_into().unwrap();
        let footer = u16::from_be_bytes(footer);
        let keep_last = usize::from(footer & 0xFFF);
        let keep_bit = (footer >> 12) & 1 == 1;
        let suppression_enabled = (footer >> 13) & 1 == 1;

        if slice.len() == 16 {
            if !suppression_enabled {
                return Err(Self::Error::IncompleteSlice {
                    found: 16,
                    min_expected: 36,
                });
            }
            if keep_bit {
                return Err(Self::Error::KeepBitMismatch { found: keep_bit });
            }
            if keep_last != 0 {
                return Err(Self::Error::BadKeepLast {
                    found: keep_last,
                    limit: 0,
                });
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
                keep_last,
                suppression_baseline,
                keep_bit,
                suppression_enabled,
            });
        }

        if slice.len() < 36 {
            return Err(Self::Error::IncompleteSlice {
                found: slice.len(),
                min_expected: 36,
            });
        }

        if slice[12..14] != [0, 0] {
            return Err(Self::Error::ZeroMismatch {
                found: slice[12..14].try_into().unwrap(),
            });
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
        if waveform_bytes % 2 != 0 {
            return Err(Self::Error::IncompleteSlice {
                // waveform bytes + header + footer
                found: waveform_bytes + 36,
                min_expected: waveform_bytes + 37,
            });
        }
        let waveform: Vec<i16> = slice[32..][..waveform_bytes]
            .chunks_exact(2)
            .map(|b| i16::from_be_bytes(b.try_into().unwrap()))
            .collect();

        if waveform.len() < BASELINE_SAMPLES {
            return Err(Self::Error::BadNumberOfSamples {
                found: waveform.len(),
                min: BASELINE_SAMPLES,
                max: requested_samples - 2,
            });
        }
        let data_baseline = {
            // Add over i32 to avoid overflow
            let num = waveform[..BASELINE_SAMPLES]
                .iter()
                .map(|n| i32::from(*n))
                .sum::<i32>();
            let d = num / 64;
            if num % 64 < 0 {
                d - 1
            } else {
                d
            }
        };
        if data_baseline != suppression_baseline.into() {
            return Err(Self::Error::BaselineMismatch {
                found: suppression_baseline,
                expected: data_baseline.try_into().unwrap(),
            });
        }

        if suppression_enabled {
            if !keep_bit {
                return Err(Self::Error::KeepBitMismatch { found: keep_bit });
            }
            if keep_last < MIN_KEEP_LAST {
                return Err(Self::Error::BadKeepLast {
                    found: keep_last,
                    limit: MIN_KEEP_LAST,
                });
            }
            let last_index = (keep_last - 1) * 2 - 2;
            if waveform.len() <= last_index {
                return Err(Self::Error::BadNumberOfSamples {
                    found: waveform.len(),
                    min: last_index + 1,
                    max: requested_samples - 2,
                });
            }
            if waveform.len() > requested_samples - 2 {
                return Err(Self::Error::BadNumberOfSamples {
                    found: waveform.len(),
                    min: last_index + 1,
                    max: requested_samples - 2,
                });
            }
        } else {
            if keep_bit {
                if keep_last < MIN_KEEP_LAST {
                    return Err(Self::Error::BadKeepLast {
                        found: keep_last,
                        limit: MIN_KEEP_LAST,
                    });
                }
                let last_index = (keep_last - 1) * 2 - 2;
                if waveform.len() <= last_index {
                    return Err(Self::Error::BadNumberOfSamples {
                        found: waveform.len(),
                        min: last_index + 1,
                        max: requested_samples - 2,
                    });
                }
            } else if keep_last != 0 {
                return Err(Self::Error::BadKeepLast {
                    found: keep_last,
                    limit: 0,
                });
            }
            if waveform.len() != requested_samples - 2 {
                return Err(Self::Error::BadNumberOfSamples {
                    found: waveform.len(),
                    min: requested_samples - 2,
                    max: requested_samples - 2,
                });
            }
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
            keep_last,
            suppression_baseline,
            keep_bit,
            suppression_enabled,
        })
    }
}

/// ADC data packet.
///
/// This enum can currently contain only an [`AdcV3Packet`]. See its
/// documentation for more details.
#[derive(Clone, Debug)]
pub enum AdcPacket {
    /// Version 3 of an ADC packet.
    V3(AdcV3Packet),
}

impl fmt::Display for AdcPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V3(packet) => write!(f, "{packet}"),
        }
    }
}

impl AdcPacket {
    /// Return the packet type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.packet_type(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn packet_type(&self) -> u8 {
        match self {
            Self::V3(packet) => packet.packet_type(),
        }
    }
    /// Return the packet version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.packet_version(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn packet_version(&self) -> u8 {
        match self {
            Self::V3(packet) => packet.packet_version(),
        }
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.accepted_trigger(), 4);
    /// # Ok(())
    /// # }
    /// ```
    pub fn accepted_trigger(&self) -> u16 {
        match self {
            Self::V3(packet) => packet.accepted_trigger(),
        }
    }
    /// Return the [`ModuleId`] of the Alpha16 board from which the packet was
    /// generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::{AdcPacket, ModuleId};
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.module_id(), ModuleId::try_from(5)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn module_id(&self) -> ModuleId {
        match self {
            Self::V3(packet) => packet.module_id(),
        }
    }
    /// Return the [`ChannelId`] in an Alpha16 board from which the packet was
    /// generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::{AdcPacket, ChannelId};
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(matches!(packet.channel_id(), ChannelId::A16(_)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_id(&self) -> ChannelId {
        match self {
            Self::V3(packet) => packet.channel_id(),
        }
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
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.requested_samples(), 699);
    /// # Ok(())
    /// # }
    /// ```
    pub fn requested_samples(&self) -> usize {
        match self {
            Self::V3(packet) => packet.requested_samples(),
        }
    }
    /// I do not know what this field means. It never matches the event
    /// timestamp in the MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.event_timestamp(), 7);
    /// # Ok(())
    /// # }
    /// ```
    pub fn event_timestamp(&self) -> u64 {
        match self {
            Self::V3(packet) => packet.event_timestamp(),
        }
    }
    /// Return the [`BoardId`] of the Alpha16 board from which the packet was
    /// generated. Return [`None`] if data suppression is enabled and the
    /// `keep_bit` is not set in a version 3 packet.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::{AdcPacket, BoardId};
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.board_id().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> Option<BoardId> {
        match self {
            Self::V3(packet) => packet.board_id(),
        }
    }
    /// I do not understand what this field means exactly. I know that it
    /// matches `adcXX_trig_delay - adcXX_trig_start` in the ODB (with `XX`
    /// equal to `16` or `32`). Return [`None`] if data suppression is enabled
    /// and the `keep_bit` is not set in a version 3 packet.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.trigger_offset().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn trigger_offset(&self) -> Option<i32> {
        match self {
            Self::V3(packet) => packet.trigger_offset(),
        }
    }
    /// Return the SOF file build timestamp; this acts as firmware version.
    /// Return [`None`] if data suppression is enabled and the `keep_bit` is not
    /// set in a version 3 packet.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.build_timestamp().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_timestamp(&self) -> Option<u32> {
        match self {
            Self::V3(packet) => packet.build_timestamp(),
        }
    }
    /// Return the digitized waveform samples received by an ADC channel in an
    /// Alpha16 board. Return an empty slice if data suppression is enabled and
    /// the `keep_bit` is not set in a version 3 packet.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.waveform().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn waveform(&self) -> &[i16] {
        match self {
            Self::V3(packet) => packet.waveform(),
        }
    }
    /// Return the data suppression waveform baseline. Return [`None`] if this
    /// is a version 1 packet (these don't have any data suppression
    /// implemented).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.suppression_baseline(), Some(0));
    /// # Ok(())
    /// # }
    /// ```
    pub fn suppression_baseline(&self) -> Option<i16> {
        match self {
            Self::V3(packet) => Some(packet.suppression_baseline()),
        }
    }
    /// This is a counter in the firmware side on how many data words are being
    /// kept due to data suppression. If the `keep_bit` is not set, then
    /// `keep_last` is equal to 0. This counter increases by the index of the
    /// last waveform sample over threshold as `keep_last = (index + 2) / 2 + 1`.
    ///
    /// Recall that data suppression doesn't "see" the last 6(?) samples, hence
    /// `keep_last` is not a reliable way to obtain the last waveform sample
    /// over the data suppression threshold. This `keep_last` value is only
    /// really useful in validating/checking the data suppression on the
    /// firmware side. If you are using this for anything else, you are most
    /// likely making a mistake.
    ///
    ///  Return [`None`] if this is a version 1 packet (these don't have any
    ///  data suppression implemented).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.keep_last(), Some(0));
    /// # Ok(())
    /// # }
    /// ```
    pub fn keep_last(&self) -> Option<usize> {
        match self {
            Self::V3(packet) => Some(packet.keep_last()),
        }
    }
    /// Return [`true`] if at least one [`waveform`] sample is over the data
    /// suppression threshold. Return [`None`] if this is a version 1 packet
    /// (these don't have any data suppression implemented).
    ///
    /// [`waveform`]: AdcV3Packet::waveform.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.keep_bit(), Some(false));
    /// # Ok(())
    /// # }
    /// ```
    pub fn keep_bit(&self) -> Option<bool> {
        match self {
            Self::V3(packet) => Some(packet.keep_bit()),
        }
    }
    /// Return [`true`] if data suppression is enabled. Return [`None`] if this
    /// is a version 1 packet (these don't have any data suppression
    /// implemented).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.is_suppression_enabled(), Some(true));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_suppression_enabled(&self) -> Option<bool> {
        match self {
            Self::V3(packet) => Some(packet.is_suppression_enabled()),
        }
    }
    /// Return [`true`] if this adc packet is an [`AdcV3Packet`], and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::alpha16::TryAdcPacketFromSliceError;
    /// # fn main() -> Result<(), TryAdcPacketFromSliceError> {
    /// use alpha_g_detector::alpha16::AdcPacket;
    ///
    /// let buffer = [1, 3, 0, 4, 5, 6, 2, 187, 0, 0, 0, 7, 224, 0, 0, 0];
    /// let packet = AdcPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.is_v3());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_v3(&self) -> bool {
        matches!(self, Self::V3(_))
    }
}

impl TryFrom<&[u8]> for AdcPacket {
    type Error = TryAdcPacketFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(AdcPacket::V3(AdcV3Packet::try_from(slice)?))
    }
}

#[cfg(test)]
mod tests;
