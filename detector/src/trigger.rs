use thiserror::Error;

/// Frequency (Hertz) of the internal clock.
pub const TRG_CLOCK_FREQ: f64 = 62.5e6;

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

impl TrgV3Packet {
    /// Return the value of the UDP packet counter. This counter is reset by
    /// reboot and starts counting from 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.udp_counter(), 255);
    /// # Ok(())
    /// # }
    pub fn udp_counter(&self) -> u32 {
        self.udp_counter
    }
    /// Return the trigger timestamp, which increments at the frequency of
    /// [`TRG_CLOCK_FREQ`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.timestamp(), 254);
    /// # Ok(())
    /// # }
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
    /// Return the counter of signals that come out of the TRG board (these are
    /// distributed to all the other modules as trigger signals).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.output_counter(), 0);
    /// # Ok(())
    /// # }
    pub fn output_counter(&self) -> u32 {
        self.output_counter
    }
    /// Return the counter of trigger input signals.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.input_counter(), 3);
    /// # Ok(())
    /// # }
    pub fn input_counter(&self) -> u32 {
        self.input_counter
    }
    /// Return the counter of pulser triggers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.pulser_counter(), 0);
    /// # Ok(())
    /// # }
    pub fn pulser_counter(&self) -> u32 {
        self.pulser_counter
    }
    /// Return a bitmap with trigger information.
    ///
    /// # Note
    ///
    /// This bitmap changes multiple times within a single TRG packet version.
    /// It is difficult to keep track, but if you really need this information,
    /// you can find more details
    /// [`here`](https://daq00.triumf.ca/AgWiki/index.php/TRG#trigger_bitmap)
    /// about a specific [`firmware_revision`].
    ///
    /// [`firmware_revision`]: TrgV3Packet::firmware_revision
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.trigger_bitmap(), 5);
    /// # Ok(())
    /// # }
    pub fn trigger_bitmap(&self) -> u32 {
        self.trigger_bitmap
    }
    /// Return a bitmap of the ADC NIM inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.nim_bitmap(), 6);
    /// # Ok(())
    /// # }
    pub fn nim_bitmap(&self) -> u32 {
        self.nim_bitmap
    }
    /// Return a bitmap of the ADC eSATA inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.esata_bitmap(), 7);
    /// # Ok(())
    /// # }
    pub fn esata_bitmap(&self) -> u32 {
        self.esata_bitmap
    }
    /// Return [`true`] if the [`aw16_prompt`] bit pattern satisfied the MLU
    /// condition. The name of the specific MLU file in a particular run can be
    /// obtained from the ODB.
    ///
    /// [`aw16_prompt`]: TrgV3Packet::aw16_prompt
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert!(packet.satisfied_mlu());
    /// # Ok(())
    /// # }
    pub fn satisfied_mlu(&self) -> bool {
        self.satisfied_mlu
    }
    /// Return the bit pattern (1 bit per preamp) of the signals collected
    /// during the MLU prompt window.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_prompt(), 8);
    /// # Ok(())
    /// # }
    pub fn aw16_prompt(&self) -> u16 {
        self.aw16_prompt
    }
    /// Return the counter of signals that passed the drift time veto.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.drift_veto_counter(), 2);
    /// # Ok(())
    /// # }
    pub fn drift_veto_counter(&self) -> u32 {
        self.drift_veto_counter
    }
    /// Return the counter of signals that passed the scaledown.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.scaledown_counter(), 1);
    /// # Ok(())
    /// # }
    pub fn scaledown_counter(&self) -> u32 {
        self.scaledown_counter
    }
    /// Return the `aw16_mult`. These are bits `[16..24]` from the word with
    /// index 13 of the data packet. I don't understand what this field means
    /// exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_multiplicity(), 10);
    /// # Ok(())
    /// # }
    pub fn aw16_multiplicity(&self) -> u8 {
        self.aw16_multiplicity
    }
    /// Return the `aw16_bus`. These are bits `[0..16]` from the word with
    /// index 13 of the data packet. I don't understand what this field means
    /// exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_bus(), 9);
    /// # Ok(())
    /// # }
    pub fn aw16_bus(&self) -> u16 {
        self.aw16_bus
    }
    /// Return the `bsc64_bus`. This corresponds to the words with indices 14
    /// and 15 as a little endian u64. I don't understand what this field means
    /// exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.bsc64_bus(), 11);
    /// # Ok(())
    /// # }
    pub fn bsc64_bus(&self) -> u64 {
        self.bsc64_bus
    }
    /// Return the `bsc64_mult`. These are bits `[0..8]` from the word with
    /// index 16 of the data packet. I don't understand what this field means
    /// exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.bsc64_multiplicity(), 12);
    /// # Ok(())
    /// # }
    pub fn bsc64_multiplicity(&self) -> u8 {
        self.bsc64_multiplicity
    }
    /// Return the `coinc_latch`. These are bits `[0..8]` from the word with
    /// index 17 of the data packet. I don't understand what this field means
    /// exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.coincidence_latch(), 13);
    /// # Ok(())
    /// # }
    pub fn coincidence_latch(&self) -> u8 {
        self.coincidence_latch
    }
    /// Return the firmware revision of the TRG board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgV3Packet;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgV3Packet::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.firmware_revision(), 14);
    /// # Ok(())
    /// # }
    pub fn firmware_revision(&self) -> u32 {
        self.firmware_revision
    }
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

/// TRG data packet.
///
/// This enum can currently contain only a [`TrgV3Packet`]. See its
/// documentation for more details.
#[derive(Clone, Debug)]
pub enum TrgPacket {
    /// Version 3 of a TRG packet.
    V3(TrgV3Packet),
}

impl TrgPacket {
    /// Return the value of the UDP packet counter. This counter is reset by
    /// reboot and starts counting from 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.udp_counter(), 255);
    /// # Ok(())
    /// # }
    pub fn udp_counter(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.udp_counter(),
        }
    }
    /// Return the trigger timestamp, which increments at the frequency of
    /// [`TRG_CLOCK_FREQ`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.timestamp(), 254);
    /// # Ok(())
    /// # }
    pub fn timestamp(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.timestamp(),
        }
    }
    /// Return the counter of signals that come out of the TRG board (these are
    /// distributed to all the other modules as trigger signals).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.output_counter(), 0);
    /// # Ok(())
    /// # }
    pub fn output_counter(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.output_counter(),
        }
    }
    /// Return the counter of trigger input signals.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.input_counter(), 3);
    /// # Ok(())
    /// # }
    pub fn input_counter(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.input_counter(),
        }
    }
    /// Return the counter of pulser triggers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.pulser_counter(), 0);
    /// # Ok(())
    /// # }
    pub fn pulser_counter(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.pulser_counter(),
        }
    }
    /// Return a bitmap with trigger information.
    ///
    /// # Note
    ///
    /// This bitmap changes multiple times within a single TRG packet version.
    /// It is difficult to keep track, but if you really need this information,
    /// you can find more details
    /// [`here`](https://daq00.triumf.ca/AgWiki/index.php/TRG#trigger_bitmap)
    /// about a specific [`firmware_revision`].
    ///
    /// [`firmware_revision`]: TrgPacket::firmware_revision
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.trigger_bitmap(), 5);
    /// # Ok(())
    /// # }
    pub fn trigger_bitmap(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.trigger_bitmap(),
        }
    }
    /// Return a bitmap of the ADC NIM inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.nim_bitmap(), 6);
    /// # Ok(())
    /// # }
    pub fn nim_bitmap(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.nim_bitmap(),
        }
    }
    /// Return a bitmap of the ADC eSATA inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.esata_bitmap(), 7);
    /// # Ok(())
    /// # }
    pub fn esata_bitmap(&self) -> u32 {
        match self {
            Self::V3(packet) => packet.esata_bitmap(),
        }
    }
    /// Return [`true`] if the [`aw16_prompt`] bit pattern satisfied the MLU
    /// condition. The name of the specific MLU file in a particular run can be
    /// obtained from the ODB. Return [`None`] if this is a version 1 packet
    /// (these don't contain this field).
    ///
    /// [`aw16_prompt`]: TrgV3Packet::aw16_prompt
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.satisfied_mlu(), Some(true));
    /// # Ok(())
    /// # }
    pub fn satisfied_mlu(&self) -> Option<bool> {
        match self {
            Self::V3(packet) => Some(packet.satisfied_mlu()),
        }
    }
    /// Return the bit pattern (1 bit per preamp) of the signals collected
    /// during the MLU prompt window. Return [`None`] if this is a version 1
    /// packet (these don't contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_prompt(), Some(8));
    /// # Ok(())
    /// # }
    pub fn aw16_prompt(&self) -> Option<u16> {
        match self {
            Self::V3(packet) => Some(packet.aw16_prompt()),
        }
    }
    /// Return the counter of signals that passed the drift time veto. Return
    /// [`None`] if this is a version 1 packet (these don't contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.drift_veto_counter(), Some(2));
    /// # Ok(())
    /// # }
    pub fn drift_veto_counter(&self) -> Option<u32> {
        match self {
            Self::V3(packet) => Some(packet.drift_veto_counter()),
        }
    }
    /// Return the counter of signals that passed the scaledown. Return [`None`]
    /// if this is a version 1 packet (these don't contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.scaledown_counter(), Some(1));
    /// # Ok(())
    /// # }
    pub fn scaledown_counter(&self) -> Option<u32> {
        match self {
            Self::V3(packet) => Some(packet.scaledown_counter()),
        }
    }
    /// Return the `aw16_mult`. These are bits `[16..24]` from the word with
    /// index 13 of the data packet. I don't understand what this field means
    /// exactly. Return [`None`] if this is a version 1 packet (these don't
    /// contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_multiplicity(), Some(10));
    /// # Ok(())
    /// # }
    pub fn aw16_multiplicity(&self) -> Option<u8> {
        match self {
            Self::V3(packet) => Some(packet.aw16_multiplicity()),
        }
    }
    /// Return the `aw16_bus`. These are bits `[0..16]` from the word with
    /// index 13 of the data packet. I don't understand what this field means
    /// exactly. Return [`None`] if this is a version 1 packet (these don't
    /// contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.aw16_bus(), Some(9));
    /// # Ok(())
    /// # }
    pub fn aw16_bus(&self) -> Option<u16> {
        match self {
            Self::V3(packet) => Some(packet.aw16_bus()),
        }
    }
    /// Return the `bsc64_bus`. This corresponds to the words with indices 14
    /// and 15 as a little endian u64. I don't understand what this field means
    /// exactly. Return [`None`] if this is a version 1 packet (these don't
    /// contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.bsc64_bus(), Some(11));
    /// # Ok(())
    /// # }
    pub fn bsc64_bus(&self) -> Option<u64> {
        match self {
            Self::V3(packet) => Some(packet.bsc64_bus()),
        }
    }
    /// Return the `bsc64_mult`. These are bits `[0..8]` from the word with
    /// index 16 of the data packet. I don't understand what this field means
    /// exactly. Return [`None`] if this is a version 1 packet (these don't
    /// contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.bsc64_multiplicity(), Some(12));
    /// # Ok(())
    /// # }
    pub fn bsc64_multiplicity(&self) -> Option<u8> {
        match self {
            Self::V3(packet) => Some(packet.bsc64_multiplicity()),
        }
    }
    /// Return the `coinc_latch`. These are bits `[0..8]` from the word with
    /// index 17 of the data packet. I don't understand what this field means
    /// exactly. Return [`None`] if this is a version 1 packet (these don't
    /// contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.coincidence_latch(), Some(13));
    /// # Ok(())
    /// # }
    pub fn coincidence_latch(&self) -> Option<u8> {
        match self {
            Self::V3(packet) => Some(packet.coincidence_latch()),
        }
    }
    /// Return the firmware revision of the TRG board. Return [`None`] if this
    /// is a version 1 or 2 packet (these don't contain this field).
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert_eq!(packet.firmware_revision(), Some(14));
    /// # Ok(())
    /// # }
    pub fn firmware_revision(&self) -> Option<u32> {
        match self {
            Self::V3(packet) => Some(packet.firmware_revision()),
        }
    }
    /// Return [`true`] if this is a [`TrgV3Packet`], and [`false`] otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
    /// # fn main() -> Result<(), TryTrgPacketFromSliceError> {
    /// use alpha_g_detector::trigger::TrgPacket;
    ///
    /// let buffer = [255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224];
    /// let packet = TrgPacket::try_from(&buffer[..])?;
    ///
    /// assert!(packet.is_v3());
    /// # Ok(())
    /// # }
    pub fn is_v3(&self) -> bool {
        matches!(self, Self::V3(_))
    }
}

impl TryFrom<&[u8]> for TrgPacket {
    type Error = TryTrgPacketFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(TrgPacket::V3(TrgV3Packet::try_from(slice)?))
    }
}

#[cfg(test)]
mod tests;
