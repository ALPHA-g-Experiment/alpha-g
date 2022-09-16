use thiserror::Error;

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to
/// [`TrgPacket`] fails.
#[derive(Error, Debug)]
pub enum TryTrgPacketFromSliceError {
    /// The length of the input slice doesn't match the expected value.
    #[error("slice length mismatch (expected `{expected}`, found `{found}`)")]
    SliceLengthMismatch { found: usize, expected: usize },
    /// The header mask doesn't match the expected `0x80000000`.
    #[error(
        "header mask mismatch (expected `{:0>32b}`, found `{found:0>32b}`)",
        0x80000000u32
    )]
    HeaderMaskMismatch { found: u32 },
    /// The footer mask doesn't match the expected `0xE0000000`.
    #[error(
        "footer mask mismatch (expected `{:0>32b}`, found `{found:0>32b}`)",
        0xE0000000u32
    )]
    FooterMaskMismatch { found: u32 },
    /// The `trig_out` counter doesn't match between the header, footer, and packet value.
    #[error("trig_out mismatch (header `{header}`, footer `{footer}`, value `{value}`)")]
    TrigOutMismatch {
        header: u32,
        footer: u32,
        value: u32,
    },
    /// The `trig_in` counter is less than the `trig_out`.
    #[error("bad trig_in (found `{found}`, expected at least `{min_expected}`)")]
    BadTrigIn { found: u32, min_expected: u32 },
    /// The `drift_counter` is greater than `trig_in` or lower than `trig_out`.
    #[error("bad drift_counter (found `{found}`, expected at least `{min_expected}` and at most `{max_expected}`)")]
    BadDriftCounter {
        found: u32,
        min_expected: u32,
        max_expected: u32,
    },
    /// The `scaledown_counter` is greater than `drift_counter` or lower than `trig_out`.
    #[error("bad scaledown_counter (found `{found}`, expected at least `{min_expected}` and at most `{max_expected}`)")]
    BadScaledownCounter {
        found: u32,
        min_expected: u32,
        max_expected: u32,
    },
    /// Non-zero value found in bytes meant to be fixed to `0`.
    // This could happen in multiple places, but I think that having an
    // individual error for each of these is not worth it.
    #[error("zero-bytes mismatch (found `{found:0>32b}`)")]
    ZeroMismatch { found: u32 },
}

/// Version 3 of a TRG data packet.
///
/// A TRG packet represents the data collected by the trigger DAQ board. The
/// binary representation of a [`TrgV3Packet`] in a data bank is shown below.
/// All multi-byte fields are little-endian:
///
/// <center>
///
/// |Byte(s)|Description|
/// |:-:|:-:|
/// |0-3|UDP packet counter|
/// |4-7|Header|
/// |8-11|Timestamp|
/// |12-15|Output counter|
/// |16-19|Input counter|
/// |20-23|Pulser counter|
/// |24-27|Trigger bitmap|
/// |28-31|NIM bitmap|
/// |32-35|ESATA bitmap|
/// |36-39|MLU AW16 prompt|
/// |40-43|Drift veto counter|
/// |44-47|Scaledown counter|
/// |48-51|Fixed to `0`|
/// |52-55|AW16 bus and multiplicity|
/// |56-63|BSC64 bus|
/// |64-67|BSC64 multiplicity|
/// |68-71|Coincidence latch|
/// |72-75|Firmware revision|
/// |76-79|Footer|
///
/// </center>
#[derive(Clone, Copy, Debug)]
pub struct TrgV3Packet {
    udp_counter: u32,
    timestamp: u32,
    output_counter: u32,
    input_counter: u32,
    pulser_counter: u32,
    trigger_bitmap: u32,
    nim_bitmap: u32,
    esata_bitmap: u32,
    satisfied_mlu: bool,
    aw16_prompt: u16,
    drift_veto_counter: u32,
    scaledown_counter: u32,
    aw16_multiplicity: u8,
    aw16_bus: u16,
    bsc64_bus: u64,
    bsc64_multiplicity: u8,
    coincidence_latch: u8,
    firmware_revision: u32,
}

impl TryFrom<&[u8]> for TrgV3Packet {
    type Error = TryTrgPacketFromSliceError;

    // All fields are little endian
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 80 {
            return Err(Self::Error::SliceLengthMismatch {
                found: slice.len(),
                expected: 80,
            });
        }

        let udp_counter = slice[..4].try_into().unwrap();
        let udp_counter = u32::from_le_bytes(udp_counter);
        if (udp_counter & 0x80000000) != 0 {
            return Err(Self::Error::ZeroMismatch { found: 0x80000000 });
        }
        let header = slice[4..8].try_into().unwrap();
        let header = u32::from_le_bytes(header);
        if (header & 0xF0000000) != 0x80000000 {
            return Err(Self::Error::HeaderMaskMismatch {
                found: header & 0xF0000000,
            });
        }
        let timestamp = slice[8..12].try_into().unwrap();
        let timestamp = u32::from_le_bytes(timestamp);
        let output_counter = slice[12..16].try_into().unwrap();
        let output_counter = u32::from_le_bytes(output_counter);
        let input_counter = slice[16..20].try_into().unwrap();
        let input_counter = u32::from_le_bytes(input_counter);
        if input_counter < output_counter {
            return Err(Self::Error::BadTrigIn {
                found: input_counter,
                min_expected: output_counter,
            });
        }
        let pulser_counter = slice[20..24].try_into().unwrap();
        let pulser_counter = u32::from_le_bytes(pulser_counter);
        let trigger_bitmap = slice[24..28].try_into().unwrap();
        let trigger_bitmap = u32::from_le_bytes(trigger_bitmap);
        let nim_bitmap = slice[28..32].try_into().unwrap();
        let nim_bitmap = u32::from_le_bytes(nim_bitmap);
        let esata_bitmap = slice[32..36].try_into().unwrap();
        let esata_bitmap = u32::from_le_bytes(esata_bitmap);
        let dummy = slice[36..40].try_into().unwrap();
        let dummy = u32::from_le_bytes(dummy);
        if (dummy & 0x7FFF0000) != 0 {
            return Err(Self::Error::ZeroMismatch {
                found: dummy & 0x7FFF0000,
            });
        }
        let satisfied_mlu = (dummy & 0x80000000) != 0;
        let aw16_prompt = (dummy & 0xFFFF).try_into().unwrap();
        let drift_veto_counter = slice[40..44].try_into().unwrap();
        let drift_veto_counter = u32::from_le_bytes(drift_veto_counter);
        if drift_veto_counter > input_counter || drift_veto_counter < output_counter {
            return Err(Self::Error::BadDriftCounter {
                found: drift_veto_counter,
                min_expected: output_counter,
                max_expected: input_counter,
            });
        }
        let scaledown_counter = slice[44..48].try_into().unwrap();
        let scaledown_counter = u32::from_le_bytes(scaledown_counter);
        if scaledown_counter > drift_veto_counter || scaledown_counter < output_counter {
            return Err(Self::Error::BadScaledownCounter {
                found: scaledown_counter,
                min_expected: output_counter,
                max_expected: drift_veto_counter,
            });
        }
        if slice[48..52] != [0, 0, 0, 0] {
            return Err(Self::Error::ZeroMismatch {
                found: {
                    let non_zero = slice[48..52].try_into().unwrap();
                    u32::from_le_bytes(non_zero)
                },
            });
        }
        let dummy = slice[52..56].try_into().unwrap();
        let dummy = u32::from_le_bytes(dummy);
        if (dummy & 0xFF000000) != 0 {
            return Err(Self::Error::ZeroMismatch {
                found: dummy & 0xFF000000,
            });
        }
        let aw16_multiplicity = (dummy >> 16).try_into().unwrap();
        let aw16_bus = (dummy & 0xFFFF).try_into().unwrap();
        let bsc64_bus = slice[56..64].try_into().unwrap();
        let bsc64_bus = u64::from_le_bytes(bsc64_bus);
        let dummy = slice[64..68].try_into().unwrap();
        let dummy = u32::from_le_bytes(dummy);
        if (dummy & 0xFFFFFF00) != 0 {
            return Err(Self::Error::ZeroMismatch {
                found: dummy & 0xFFFFFF00,
            });
        }
        let bsc64_multiplicity = (dummy & 0xFF).try_into().unwrap();
        let dummy = slice[68..72].try_into().unwrap();
        let dummy = u32::from_le_bytes(dummy);
        if (dummy & 0xFFFFFF00) != 0 {
            return Err(Self::Error::ZeroMismatch {
                found: dummy & 0xFFFFFF00,
            });
        }
        let coincidence_latch = (dummy & 0xFF).try_into().unwrap();
        let firmware_revision = slice[72..76].try_into().unwrap();
        let firmware_revision = u32::from_le_bytes(firmware_revision);
        let footer = slice[76..80].try_into().unwrap();
        let footer = u32::from_le_bytes(footer);
        if (footer & 0xF0000000) != 0xE0000000 {
            return Err(Self::Error::FooterMaskMismatch {
                found: footer & 0xF0000000,
            });
        }
        if (header & 0xFFFFFFF) != (footer & 0xFFFFFFF)
            || (header & 0xFFFFFFF != output_counter & 0xFFFFFFF)
        {
            return Err(Self::Error::TrigOutMismatch {
                header: header & 0xFFFFFFF,
                footer: footer & 0xFFFFFFF,
                value: output_counter & 0xFFFFFFF,
            });
        }
        Ok(Self {
            udp_counter,
            timestamp,
            output_counter,
            input_counter,
            pulser_counter,
            trigger_bitmap,
            nim_bitmap,
            esata_bitmap,
            satisfied_mlu,
            aw16_prompt,
            drift_veto_counter,
            scaledown_counter,
            aw16_multiplicity,
            aw16_bus,
            bsc64_bus,
            bsc64_multiplicity,
            coincidence_latch,
            firmware_revision,
        })
    }
}
