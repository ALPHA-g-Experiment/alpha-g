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
    /// sample is [`i16`]. Or the number of waveform samples is shorted than the
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
    pub fn requested_samples(&self) -> usize {
        self.requested_samples
    }
    pub fn event_timestamp(&self) -> u64 {
        self.event_timestamp
    }
    pub fn board_id(&self) -> Option<BoardId> {
        self.board_id
    }
    pub fn trigger_offset(&self) -> Option<i32> {
        self.trigger_offset
    }
    pub fn build_timestamp(&self) -> Option<u32> {
        self.build_timestamp
    }
    pub fn waveform(&self) -> &[i16] {
        &self.waveform
    }
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
    pub fn keep_last(&self) -> usize {
        self.keep_last
    }
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
            if keep_bit {
                return Err(Self::Error::KeepBitMismatch);
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
        if waveform_bytes % 2 != 0 || waveform_bytes < 128 {
            return Err(Self::Error::IncompleteWaveform);
        }
        if !keep_bit {
            return Err(Self::Error::KeepBitMismatch);
        }
        if suppression_enabled && waveform_bytes / 2 > requested_samples {
            return Err(Self::Error::NumberOfSamplesMismatch);
        }
        if !suppression_enabled && waveform_bytes / 2 != requested_samples {
            return Err(Self::Error::NumberOfSamplesMismatch);
        }
        if waveform_bytes / 2 + 3 < 2 * keep_last {
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
            keep_last,
            suppression_enabled,
        })
    }
}

#[cfg(test)]
mod tests;
