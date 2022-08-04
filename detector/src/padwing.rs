use std::fmt;
use thiserror::Error;

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to [`Chunk`]
/// fails.
#[derive(Error, Debug)]
pub enum TryChunkFromSliceError {
    /// The input slice is not long enough to contain a complete chunk.
    #[error("incomplete slice (expected at least `{min_expected}` bytes, found `{found}`)")]
    IncompleteSlice { found: usize, min_expected: usize },
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
    pub fn device_id(&self) -> u32 {
        self.device_id
    }
    pub fn packet_sequence(&self) -> u32 {
        self.packet_sequence
    }
    pub fn channel_sequence(&self) -> u16 {
        self.channel_sequence
    }
    pub fn channel_id(&self) -> u8 {
        self.channel_id
    }
    pub fn is_end_of_message(&self) -> bool {
        self.flags & 1 == 1
    }
    pub fn chunk_id(&self) -> u16 {
        self.chunk_id
    }
    pub fn header_crc32c(&self) -> u32 {
        1
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn payload_crc32c(&self) -> u32 {
        1
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
        let packet_sequence = slice[4..8].try_into().unwrap();
        let packet_sequence = u32::from_le_bytes(packet_sequence);
        let channel_sequence = slice[8..10].try_into().unwrap();
        let channel_sequence = u16::from_le_bytes(channel_sequence);
        let channel_id = slice[10];
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
