use std::fmt;
use thiserror::Error;

/// The error type returned when parsing a [`BoardId`] fails.
#[derive(Error, Debug)]
#[error("unknown parsing from board name `{input}` to BoardId")]
pub struct ParseBoardIdError {
    input: String,
}

/// The error type returned when conversion from mac address to [`BoardId`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from mac address `{input:?}` to BoardId")]
pub struct TryBoardIdFromMacAddressError {
    input: [u8; 6],
}

/// The error type returned when conversion from unsigned integer to [`BoardId`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to BoardId")]
pub struct TryBoardIdFromUnsignedError {
    input: u32,
}

// Known PadWing board names, mac addresses, and device IDs
// Just add new boards to this list
// ("name", [mac address], device_id)
// "name" is 2 ASCII characters that also appear in the data bank name
// Note: The device ID is just the first 4 bytes of the MAC address as
// little endian u32. Maybe remove the last u32 in the future, and just get it
// from the MAC address.
const PADWINGBOARDS: [(&str, [u8; 6], u32); 64] = [
    ("00", [236, 40, 255, 135, 84, 2], 2281646316),
    ("01", [236, 40, 250, 162, 84, 2], 2734303468),
    ("02", [236, 40, 136, 108, 84, 2], 1820862700),
    ("03", [236, 40, 226, 49, 84, 2], 836905196),
    ("04", [236, 41, 12, 121, 84, 2], 2030840300),
    ("05", [236, 40, 211, 69, 84, 2], 1171466476),
    ("06", [236, 40, 218, 6, 84, 2], 114960620),
    ("07", [236, 40, 116, 164, 84, 2], 2759076076),
    ("08", [236, 40, 253, 139, 84, 2], 2348624108),
    ("10", [236, 40, 248, 75, 84, 2], 1274554604),
    ("11", [236, 40, 197, 187, 84, 2], 3150260460),
    ("12", [236, 41, 34, 206, 84, 2], 3458345452),
    ("13", [236, 40, 159, 252, 84, 2], 4238289132),
    ("14", [236, 41, 44, 52, 84, 2], 875309548),
    ("15", [236, 40, 219, 60, 84, 2], 1020995820),
    ("17", [236, 40, 153, 39, 84, 2], 664348908),
    ("18", [236, 40, 228, 87, 84, 2], 1474570476),
    ("19", [236, 40, 116, 173, 84, 2], 2910071020),
    ("20", [236, 40, 219, 80, 84, 2], 1356540140),
    ("21", [236, 40, 221, 26, 84, 2], 450701548),
    ("22", [236, 40, 113, 70, 84, 2], 1181821164),
    ("23", [236, 41, 39, 253, 84, 2], 4247202284),
    ("24", [236, 40, 226, 191, 84, 2], 3219269868),
    ("25", [236, 40, 212, 176, 84, 2], 2966694124),
    ("26", [236, 40, 188, 31, 84, 2], 532424940),
    ("27", [236, 40, 252, 239, 84, 2], 4026280172),
    ("29", [236, 40, 108, 189, 84, 2], 3177982188),
    ("33", [236, 40, 255, 150, 84, 2], 2533304556),
    ("34", [236, 40, 226, 52, 84, 2], 887236844),
    ("35", [236, 40, 137, 30, 84, 2], 512305388),
    ("36", [236, 40, 165, 153, 84, 2], 2577737964),
    ("37", [236, 41, 43, 61, 84, 2], 1026238956),
    ("39", [236, 41, 43, 253, 84, 2], 4247464428),
    ("40", [236, 40, 198, 81, 84, 2], 1371941100),
    ("41", [236, 40, 187, 198, 84, 2], 3334154476),
    ("42", [236, 41, 41, 188, 84, 2], 3156814316),
    ("44", [236, 40, 218, 198, 84, 2], 3336186092),
    ("45", [236, 41, 24, 143, 84, 2], 2400725484),
    ("46", [236, 40, 160, 64, 84, 2], 1084238060),
    ("49", [236, 40, 156, 87, 84, 2], 1469851884),
    ("52", [236, 41, 24, 28, 84, 2], 471345644),
    ("53", [236, 40, 183, 208, 84, 2], 3501664492),
    ("54", [236, 40, 113, 62, 84, 2], 1047603436),
    ("55", [236, 40, 255, 172, 84, 2], 2902403308),
    ("56", [236, 40, 135, 152, 84, 2], 2558994668),
    ("57", [236, 40, 128, 45, 84, 2], 763373804),
    ("58", [236, 41, 42, 70, 84, 2], 1177168364),
    ("60", [236, 40, 243, 36, 84, 2], 619915500),
    ("63", [236, 40, 108, 234, 84, 2], 3932956908),
    ("64", [236, 40, 110, 20, 84, 2], 342763756),
    ("65", [236, 40, 215, 15, 84, 2], 265758956),
    ("66", [236, 40, 197, 199, 84, 2], 3351587052),
    ("67", [236, 40, 183, 38, 84, 2], 649537772),
    ("68", [236, 40, 211, 91, 84, 2], 1540565228),
    ("69", [236, 40, 224, 249, 84, 2], 4192217324),
    ("70", [236, 40, 248, 99, 84, 2], 1677207788),
    ("71", [236, 40, 129, 16, 84, 2], 276900076),
    ("72", [236, 40, 241, 249, 84, 2], 4193331436),
    ("73", [236, 40, 113, 64, 84, 2], 1081157868),
    ("74", [236, 40, 252, 14, 84, 2], 251406572),
    ("75", [236, 41, 39, 26, 84, 2], 438774252),
    ("76", [236, 40, 244, 136, 84, 2], 2297702636),
    ("77", [236, 41, 17, 29, 84, 2], 487664108),
    ("78", [236, 41, 37, 14, 84, 2], 237316588),
];

/// Identity of a physical PadWing board.
///
/// It is important to notice that a [`BoardId`] is different to a [`ModuleId`].
/// The former identifies a physical PadWing board, while the latter is a fixed
/// ID that maps a module to cathode pads. The mapping between
/// [`BoardId`] and [`ModuleId`] depends on the run number e.g. we switch an old
/// board for a new board. You can see the [`ModuleId`] as the slot in which a
/// board is plugged, which always maps to the same cathode pads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardId {
    name: &'static str,
    mac_address: [u8; 6],
    device_id: u32,
}
impl TryFrom<&str> for BoardId {
    type Error = ParseBoardIdError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        for triplet in PADWINGBOARDS {
            if name == triplet.0 {
                return Ok(BoardId {
                    name: triplet.0,
                    mac_address: triplet.1,
                    device_id: triplet.2,
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
        for triplet in PADWINGBOARDS {
            if mac == triplet.1 {
                return Ok(BoardId {
                    name: triplet.0,
                    mac_address: triplet.1,
                    device_id: triplet.2,
                });
            }
        }
        Err(TryBoardIdFromMacAddressError { input: mac })
    }
}
impl TryFrom<u32> for BoardId {
    type Error = TryBoardIdFromUnsignedError;

    fn try_from(device_id: u32) -> Result<Self, Self::Error> {
        for triplet in PADWINGBOARDS {
            if device_id == triplet.2 {
                return Ok(BoardId {
                    name: triplet.0,
                    mac_address: triplet.1,
                    device_id: triplet.2,
                });
            }
        }
        Err(TryBoardIdFromUnsignedError { input: device_id })
    }
}
impl BoardId {
    /// Return the name of a physical PadWing board. This is a human readable
    /// name used to identify a board instead of the mac address or device ID.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryBoardIdFromMacAddressError;
    /// # fn main() -> Result<(), TryBoardIdFromMacAddressError> {
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let board_id = BoardId::try_from([236, 40, 255, 135, 84, 2])?;
    /// assert_eq!(board_id.name(), "00");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        self.name
    }
    /// Return the mac address of a physical PadWing board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::ParseBoardIdError;
    /// # fn main() -> Result<(), ParseBoardIdError> {
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let board_id = BoardId::try_from("00")?;
    /// assert_eq!(board_id.mac_address(), [236, 40, 255, 135, 84, 2]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }
    /// Return the device ID of a physical PadWing board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::ParseBoardIdError;
    /// # fn main() -> Result<(), ParseBoardIdError> {
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let board_id = BoardId::try_from("00")?;
    /// assert_eq!(board_id.device_id(), 2281646316);
    /// # Ok(())
    /// # }
    /// ```
    pub fn device_id(&self) -> u32 {
        self.device_id
    }
}

/// The error type returned when conversion from unsigned integer to [`AfterId`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to AfterId")]
pub struct TryAfterIdFromUnsignedError {
    input: u8,
}

/// The error type returned when parsing an [`AfterId`] fails.
#[derive(Error, Debug)]
#[error("unknown parsing from char `{input}` to AfterId")]
pub struct ParseAfterIdError {
    input: char,
}

/// AFTER chip in a PadWing board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterId {
    A,
    B,
    C,
    D,
}
impl TryFrom<u8> for AfterId {
    type Error = TryAfterIdFromUnsignedError;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            _ => Err(Self::Error { input: num }),
        }
    }
}
impl TryFrom<char> for AfterId {
    type Error = ParseAfterIdError;

    fn try_from(character: char) -> Result<Self, Self::Error> {
        match character {
            'A' | 'a' => Ok(Self::A),
            'B' | 'b' => Ok(Self::B),
            'C' | 'c' => Ok(Self::C),
            'D' | 'd' => Ok(Self::D),
            _ => Err(Self::Error { input: character }),
        }
    }
}

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to [`Chunk`]
/// fails.
#[derive(Error, Debug)]
pub enum TryChunkFromSliceError {
    /// The input slice is not long enough to contain a complete chunk.
    #[error("incomplete slice (expected at least `{min_expected}` bytes, found `{found}`)")]
    IncompleteSlice { found: usize, min_expected: usize },
    /// Integer representation of device ID doesn't match any known [`BoardId`].
    #[error("unknown device id")]
    UnknownDeviceId(#[from] TryBoardIdFromUnsignedError),
    /// Integer representation of ASIC ID doesn't match any known [`AfterId`].
    #[error("unknown channel id")]
    UnknownChannelId(#[from] TryAfterIdFromUnsignedError),
    /// Integer representation of flags doesn't match any known set of flags.
    #[error("unknown flags `{found:0>8b}`")]
    UnknownFlags { found: u8 },
    /// The number of unpadded payload bytes is below/above the expected
    /// minimum/maximum.
    #[error("bad chunk length `{found}` (expected at least `{min}` and at most `{max}`)")]
    BadChunkLength {
        found: usize,
        min: usize,
        max: usize,
    },
    /// Non-zero value found in bytes meant to be fixed to `0`.
    #[error("zero-bytes mismatch (found `{found:?}`)")]
    ZeroMismatch { found: Vec<u8> },
    /// The CRC-32C value calculated form the first four words of the header
    /// doesn't match the expected value.
    #[error("header CRC-32C mismatch (expected `{expected}`, found `{found}`)")]
    HeaderCRC32CMismatch { found: u32, expected: u32 },
    /// The CRC-32C value calculated form the payload (including padding bytes)
    /// doesn't match the expected value.
    #[error("payload CRC-32C mismatch (expected `{expected}`, found `{found}`)")]
    PayloadCRC32CMismatch { found: u32, expected: u32 },
}

/// MCP Chunk.
///
/// A Chunk in the Message Chunk Protocol. Here, a packet (message) of data is
/// broken into chunks. Each chunk contains a header and a payload. The binary
/// representation of a [`Chunk`] in a data bank is shown below. All multi-byte
/// fields are little-endian:
///
/// <center>
///
/// |Byte(s)|Description|
/// |:-:|:-:|
/// |0-3|Device ID|
/// |4-7|Packet sequence|
/// |8-9|Channel sequence|
/// |10|Channel ID|
/// |11|Flags|
/// |12-13|Chunk ID|
/// |14-15|Chunk length|
/// |16-19|Header CRC-32C|
/// |...|Payload, 32-bit aligned|
/// |Last 4 bytes| Payload CRC-32C|
///
/// </center>
#[derive(Clone, Debug)]
pub struct Chunk {
    // Even though device_id and channel_id represent the BoardId and AfterId,
    // keep them as integers internally to simplify the CRC-32C and printing
    // without having to implemento Into u32/u8.
    // The constructor should ensure that BoardId::try_from and
    // AfterId::try_from cannot fail.
    device_id: u32,
    packet_sequence: u32,
    channel_sequence: u16,
    channel_id: u8,
    flags: u8,
    chunk_id: u16,
    payload: Vec<u8>,
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Device ID: {}", self.device_id)?;
        writeln!(f, "Packet sequence: {}", self.packet_sequence)?;
        writeln!(f, "Channel sequence: {}", self.channel_sequence)?;
        writeln!(f, "Channel ID: {}", self.channel_id)?;
        writeln!(f, "Flags: {:0>8b}", self.flags)?;
        writeln!(f, "Chunk ID: {}", self.chunk_id)?;
        writeln!(f, "Chunk length: {}", self.payload.len())?;
        writeln!(f, "Header CRC-32C: {}", self.header_crc32c())?;
        write!(f, "Payload CRC-32C: {}", self.payload_crc32c())?;

        Ok(())
    }
}

impl Chunk {
    /// Return the board ID of the PWB from where the [`Chunk`] is sent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use alpha_g_detector::padwing::{BoardId, Chunk};
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.board_id(), BoardId::try_from("00")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn board_id(&self) -> BoardId {
        BoardId::try_from(self.device_id).unwrap()
    }
    /// Return the packet sequence. This is a counter associated to a device
    /// which increments every time a [`Chunk`] is sent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.packet_sequence(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn packet_sequence(&self) -> u32 {
        self.packet_sequence
    }
    /// Return the channel sequence. This is a counter associated to a channel
    /// ID which increments every time a [`Chunk`] is sent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.channel_sequence(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_sequence(&self) -> u16 {
        self.channel_sequence
    }
    /// Return the AFTER chip ID for which this [`Chunk`] corresponds to.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::{AfterId, Chunk};
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.after_id(), AfterId::A);
    /// # Ok(())
    /// # }
    /// ```
    pub fn after_id(&self) -> AfterId {
        AfterId::try_from(self.channel_id).unwrap()
    }
    /// Return [`true`] if this is the last [`Chunk`] in a message (by actual
    /// position, not sent sequence).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert!(chunk.is_end_of_message());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_end_of_message(&self) -> bool {
        self.flags & 1 == 1
    }
    /// Return the chunk ID. This is a counter that indicates the position of
    /// the [`Chunk`] within a message. The first chunk has an ID equal to `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.chunk_id(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn chunk_id(&self) -> u16 {
        self.chunk_id
    }
    /// Return the CRC-32C value calculated from the first 16 bytes of the
    /// header.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.header_crc32c(), 1367591823);
    /// # Ok(())
    /// # }
    /// ```
    pub fn header_crc32c(&self) -> u32 {
        let slice: Vec<u8> = self
            .device_id
            .to_le_bytes()
            .into_iter()
            .chain(self.packet_sequence.to_le_bytes().into_iter())
            .chain(self.channel_sequence.to_le_bytes().into_iter())
            .chain(self.channel_id.to_le_bytes().into_iter())
            .chain(self.flags.to_le_bytes().into_iter())
            .chain(self.chunk_id.to_le_bytes().into_iter())
            .chain(
                u16::try_from(self.payload.len())
                    .unwrap()
                    .to_le_bytes()
                    .into_iter(),
            )
            .collect();
        !crc32c::crc32c(&slice[..])
    }
    /// Return the payload without padding bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.payload(), [255]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    /// Return the CRC-32C value calculated from the payload and the padding
    /// bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::TryChunkFromSliceError;
    /// # fn main() -> Result<(), TryChunkFromSliceError> {
    /// use alpha_g_detector::padwing::Chunk;
    ///
    /// let buffer = [236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122, 92, 155, 159];
    /// let chunk = Chunk::try_from(&buffer[..])?;
    ///
    /// assert_eq!(chunk.payload_crc32c(), 2677759098);
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_crc32c(&self) -> u32 {
        let padding = match self.payload.len() % 4 {
            0 => 0,
            r => 4 - r,
        };
        let slice: Vec<u8> = self
            .payload
            .clone()
            .into_iter()
            .chain(std::iter::repeat(0).take(padding))
            .collect();
        !crc32c::crc32c(&slice[..])
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = TryChunkFromSliceError;

    // All fields are little endian
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        // 20 -> Header
        // 1 -> Payload
        // 3 -> Padding 32 bit aligned
        // 4 -> Payload CRC-32C
        if slice.len() < 28 {
            return Err(Self::Error::IncompleteSlice {
                found: slice.len(),
                min_expected: 28,
            });
        }
        // payload has to be 32-bit aligned
        if slice.len() % 4 != 0 {
            return Err(Self::Error::IncompleteSlice {
                found: slice.len(),
                min_expected: slice.len() + 4 - slice.len() % 4,
            });
        }
        let device_id = slice[..4].try_into().unwrap();
        let device_id = u32::from_le_bytes(device_id);
        BoardId::try_from(device_id)?;
        let packet_sequence = slice[4..8].try_into().unwrap();
        let packet_sequence = u32::from_le_bytes(packet_sequence);
        let channel_sequence = slice[8..10].try_into().unwrap();
        let channel_sequence = u16::from_le_bytes(channel_sequence);
        let channel_id = slice[10];
        AfterId::try_from(channel_id)?;
        let flags = slice[11];
        if flags != 0 && flags != 1 {
            return Err(Self::Error::UnknownFlags { found: flags });
        }
        let chunk_id = slice[12..14].try_into().unwrap();
        let chunk_id = u16::from_le_bytes(chunk_id);
        let chunk_length = slice[14..16].try_into().unwrap();
        let chunk_length = u16::from_le_bytes(chunk_length).into();
        let max = slice.len() - 24;
        let min = max - 3;
        if chunk_length < min || chunk_length > max {
            return Err(Self::Error::BadChunkLength {
                found: chunk_length,
                min,
                max,
            });
        }
        let header_crc = slice[16..20].try_into().unwrap();
        let header_crc = u32::from_le_bytes(header_crc);
        let expected_crc = !crc32c::crc32c(&slice[0..16]);
        if header_crc != expected_crc {
            return Err(Self::Error::HeaderCRC32CMismatch {
                found: header_crc,
                expected: expected_crc,
            });
        }
        let payload = slice[20..][..chunk_length].to_vec();
        let padding = slice[20 + chunk_length..slice.len() - 4].to_vec();
        if padding.iter().any(|&x| x != 0) {
            return Err(Self::Error::ZeroMismatch { found: padding });
        }
        let payload_crc = slice[slice.len() - 4..].try_into().unwrap();
        let payload_crc = u32::from_le_bytes(payload_crc);
        let expected_crc = !crc32c::crc32c(&slice[20..slice.len() - 4]);
        if payload_crc != expected_crc {
            return Err(Self::Error::PayloadCRC32CMismatch {
                found: payload_crc,
                expected: expected_crc,
            });
        }

        Ok(Self {
            device_id,
            packet_sequence,
            channel_sequence,
            channel_id,
            flags,
            chunk_id,
            payload,
        })
    }
}

/// The error type returned when conversion from unsigned integer to
/// [`Compression`] fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to Compression")]
pub struct TryCompressionFromUnsignedError {
    input: u8,
}

/// Compression types available for the PadWing boards event data.
#[derive(Clone, Copy, Debug)]
pub enum Compression {
    /// Uncompressed raw data. Any SCA channel data is sent without compression,
    /// in 16-bit signed format.
    Raw,
}
impl TryFrom<u8> for Compression {
    type Error = TryCompressionFromUnsignedError;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(Self::Raw),
            _ => Err(Self::Error { input: num }),
        }
    }
}

/// The error type returned when conversion from unsigned integer to [`Trigger`]
/// fails.
#[derive(Error, Debug)]
#[error("unknown conversion from unsigned `{input}` to Trigger")]
pub struct TryTriggerFromUnsignedError {
    input: u8,
}

/// Trigger sources available that cause an event to be captured.
#[derive(Clone, Copy, Debug)]
pub enum Trigger {
    /// Trigger came from the external pin on the PadWing board.
    External,
    /// Trigger came from the NIOS via user request.
    Manual,
    /// Trigger came from the internal pulser.
    InternalPulse,
}
impl TryFrom<u8> for Trigger {
    type Error = TryTriggerFromUnsignedError;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(Self::External),
            1 => Ok(Self::Manual),
            3 => Ok(Self::InternalPulse),
            _ => Err(Self::Error { input: num }),
        }
    }
}

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to
/// [`PwbPacket`] fails.
#[derive(Error, Debug)]
pub enum TryPwbPacketFromSliceError {
    /// The input slice is not long enough to contain a complete packet.
    #[error("incomplete slice (expected at least `{min_expected}` bytes, found `{found}`)")]
    IncompleteSlice { found: usize, min_expected: usize },
    /// Unknown packet version
    #[error("unknown packet version `{found}`")]
    UnknownVersion { found: u8 },
    /// ASCII representation of AFTER chip doesn't match any known [`AfterId`].
    #[error("unknown AFTER id")]
    UnknownAfterId(#[from] ParseAfterIdError),
    /// Integer representation of compression doesn't match any known
    /// [`Compression`].
    #[error("unknown compression")]
    UnknownCompression(#[from] TryCompressionFromUnsignedError),
    /// Integer representation of compression doesn't match any known
    /// [`Trigger`].
    #[error("unknown trigger source")]
    UnknownTrigger(#[from] TryTriggerFromUnsignedError),
    /// MAC address doesn't map to any known [`BoardId`].
    #[error("unknown mac address")]
    UnknownMac(#[from] TryBoardIdFromMacAddressError),
    /// Non-zero value found in bytes meant to be fixed to `0`.
    #[error("zero-bytes mismatch (found `{found:?}`)")]
    ZeroMismatch { found: [u8; 2] },
    /// The value of `last_sca_cell` is larger than `511`. There are only `511`
    /// SCA cells per channel.
    #[error("bad last_sca_cell `{found}`")]
    BadLastScaCell { found: u16 },
    /// The value of `requested_samples` is larger than `511`. There are
    /// only `511` SCA cells per channel.
    #[error("bad requested_samples `{found}`")]
    BadScaSamples { found: usize },
    /// The 79th bit in set channels_sent bit mask is set. There are only 79
    /// channels i.e. 78th is maximum possible bit.
    #[error("bad channels sent bit mask")]
    BadScaChannelsSent,
    /// The 79th bit in set channels_threshold bit mask is set. There are only
    /// 79 channels i.e. 78th is maximum possible bit.
    #[error("bad channels over threshold bit mask")]
    BadScaChannelsThreshold,
    // Still missing errors from the waveforms themselves, but first need to
    // understand them.
}

#[cfg(test)]
mod tests;
