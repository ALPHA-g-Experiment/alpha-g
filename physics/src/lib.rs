use crate::calibration::pads::baseline::try_pad_baseline;
use crate::calibration::pads::gain::try_pad_gain;
use crate::calibration::wires::baseline::try_wire_baseline;
use crate::calibration::wires::gain::try_wire_gain;
use crate::deconvolution::pads::pad_deconvolution;
use crate::deconvolution::wires::{
    contiguous_ranges, remove_noise_after_t, wire_range_deconvolution,
};
use crate::drift::DRIFT_TABLES;
use crate::matching::{match_column_inputs, pad_column_to_wires, t_min, wire_to_pad_column};
use alpha_g_detector::alpha16::aw_map::{
    self, MapTpcWirePositionError, TpcWirePosition, TPC_ANODE_WIRES,
};
use alpha_g_detector::alpha16::{self, AdcPacket, TryAdcPacketFromSliceError};
use alpha_g_detector::midas::{
    Adc32BankName, Alpha16BankName, MainEventBankName, ParseMainEventBankNameError,
};
use alpha_g_detector::padwing::map::{
    MapTpcPadPositionError, TpcPadPosition, TPC_PAD_COLUMNS, TPC_PAD_ROWS,
};
use alpha_g_detector::padwing::{
    self, Chunk, PwbPacket, TryChunkFromSliceError, TryPwbPacketFromChunksError,
};
use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
use alpha_g_detector::trigger::{self, TrgPacket};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uom::si::f64::*;

pub use crate::calibration::pads::baseline::MapPadBaselineError;
pub use crate::calibration::pads::gain::MapPadGainError;
pub use crate::calibration::wires::baseline::MapWireBaselineError;
pub use crate::calibration::wires::gain::MapWireGainError;
pub use crate::drift::TryDriftLookupError;

// Calibration
//
// I have the strong opinion that all calibration is an implementation detail
// and should not be exposed to the user.  This also means that after every
// calibration procedure, all binaries that use the calibration data should be
// recompiled and redistributed. This has the advantage that everyone is
// guaranteed to use the same (and correct) calibration settings.
//
// If there is ever a compelling reason to expose calibration to the user, I
// believe it should be moved to a separate `alpha_g_calibration` crate.
mod calibration;
// Map, as a function of `z` (given that the B field is non-homogeneous through
// the entire rTPC volume), a given drift time to a radius and Lorentz angle
// correction.
mod drift;
// Extract avalanche time and amplitude information from the wire and pad
// signals.
mod deconvolution;
// Match wire and pad signals to obtain Avalanches.
mod matching;
/// Vertex reconstruction.
pub mod reconstruction;

/// Townsend avalanche generated in the multiplying region near an anode wire
/// surface.
///
/// All avalanches happen at the same radius equal to [`ANODE_WIRES_RADIUS`].
#[derive(Clone, Copy, Debug)]
pub struct Avalanche {
    /// Time with respect to the first avalanche in the same event.
    pub t: Time,
    /// Azimuthal angle of the avalanche.
    pub phi: Angle,
    /// Axial position of the avalanche. The center of the detector is at
    /// `z = 0`.
    pub z: Length,
    /// Amplitude of the avalanche in arbitrary units. Useful for relative
    /// comparisons between avalanches in the same event.
    ///
    /// The absolute magnitude of this amplitude is subject to change at any
    /// time without being considered a breaking change.  Do not use this value
    /// to apply e.g. threshold cuts, etc.
    pub amplitude: f64,
}

/// Radial position of the anode wires.
pub const ANODE_WIRES_RADIUS: Length = Length {
    dimension: uom::lib::marker::PhantomData,
    units: uom::lib::marker::PhantomData,
    value: aw_map::ANODE_WIRES_RADIUS,
};

/// Frequency of the internal TRG clock.
pub const TRG_CLOCK_FREQ: Frequency = Frequency {
    dimension: uom::lib::marker::PhantomData,
    units: uom::lib::marker::PhantomData,
    value: trigger::TRG_CLOCK_FREQ,
};

/// Reconstructed ionization position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacePoint {
    /// Radial position of the ionization.
    pub r: Length,
    /// Azimuthal angle of the ionization position.
    pub phi: Angle,
    /// Axial position of the ionization. The center of the detector is at
    /// `z = 0`.
    pub z: Length,
}

impl TryFrom<Avalanche> for SpacePoint {
    type Error = TryDriftLookupError;

    fn try_from(avalanche: Avalanche) -> Result<Self, Self::Error> {
        let (r, lorentz_correction) = DRIFT_TABLES.at(avalanche.z, avalanche.t)?;

        Ok(SpacePoint {
            r,
            phi: avalanche.phi - lorentz_correction,
            z: avalanche.z,
        })
    }
}

impl SpacePoint {
    /// Return the `x` coordinate of the ionization position.
    pub fn x(self) -> Length {
        self.r * self.phi.cos()
    }
    /// Return the `y` coordinate of the ionization position.
    pub fn y(self) -> Length {
        self.r * self.phi.sin()
    }
}

/// The error type returned when conversion from data banks to a [`MainEvent`]
/// fails.
#[derive(Error, Debug)]
pub enum TryMainEventFromDataBanksError {
    /// A given string does not match the pattern of any known bank name.
    #[error("unknown bank name")]
    UnknownBank(#[from] ParseMainEventBankNameError),
    /// The data from an Alpha16 board is invalid.
    #[error("bad alpha16 data")]
    BadAlpha16(#[from] TryAdcPacketFromSliceError),
    /// The `(board_id, channel_id)` pair in an ADC packet does not match the
    /// expected value from the bank name.
    #[error("alpha16 board_id/channel_id mismatch (expected {expected:?}, found {found:?})")]
    Alpha16IdMismatch {
        expected: (alpha16::BoardId, alpha16::Adc32ChannelId),
        found: (alpha16::BoardId, alpha16::Adc32ChannelId),
    },
    /// An anode wire data bank has a BV channel_id.
    #[error("anode wire data bank `{bank_name:?}` has a BV channel_id")]
    WireBankWithBvChannel { bank_name: Adc32BankName },
    /// Duplicate anode wire data banks were found.
    #[error("duplicate anode wire data banks with name `{bank_name:?}`")]
    DuplicateWireBank { bank_name: Adc32BankName },
    /// The chunk data from a Padwing bank is invalid.
    #[error("bad padwing chunk data")]
    BadPadwingChunk(#[from] TryChunkFromSliceError),
    /// The Padwing board_id in a chunk does not match the expected value from
    /// the bank name.
    #[error("padwing board_id mismatch (expected `{expected:?}`, found `{found:?}`)")]
    PadwingBoardIdMismatch {
        expected: padwing::BoardId,
        found: padwing::BoardId,
    },
    /// The chunks from a Padwing board are invalid.
    #[error("bad padwing data")]
    BadPadwing(#[from] TryPwbPacketFromChunksError),
    /// Duplicate cathode pad signal found.
    #[error("duplicate cathode pad signal in position `{position:?}`")]
    DuplicatePadSignal { position: TpcPadPosition },
    /// The data from the TRG board is invalid.
    #[error("bad trg data")]
    BadTrg(#[from] TryTrgPacketFromSliceError),
    /// Duplicate trigger data bank found.
    #[error("duplicate trigger data bank")]
    DuplicateTrgBank,
    /// Missing trigger data bank.
    #[error("missing trigger data bank")]
    MissingTrgBank,
    /// Mapping an anode wire position failed.
    #[error("wire position mapping failed")]
    WirePositionError(#[from] MapTpcWirePositionError),
    /// Mapping a pad position failed.
    #[error("pad position mapping failed")]
    PadPositionError(#[from] MapTpcPadPositionError),
    /// Wire baseline calibration failed.
    #[error("wire baseline calibration failed")]
    WireBaselineError(#[from] MapWireBaselineError),
    /// Wire gain calibration failed.
    #[error("wire gain calibration failed")]
    WireGainError(#[from] MapWireGainError),
    /// Pad baseline calibration failed.
    #[error("pad baseline calibration failed")]
    PadBaselineError(#[from] MapPadBaselineError),
    /// Pad gain calibration failed.
    #[error("pad gain calibration failed")]
    PadGainError(#[from] MapPadGainError),
}

/// ALPHA-g main event.
#[derive(Debug, Clone)]
pub struct MainEvent {
    // These are `Option` given that a channel could simply not have any data
    // for a given event (data suppression, etc.).
    //
    // As explained in the `alpha_g_detector` documentation, the
    // TpcWirePosition::try_from(0) does not necessarily mean `phi = 0`. We
    // should make no assumptions from this `unsigned index` value. Nonetheless,
    // contiguous wire channels are expected to have contiguous indices.
    // It is just easier to work with an array (and their indices) than a map
    // with a `TpcWirePosition` key. (As long as we are careful about the
    // 0th wire channel.)
    wire_signals: [Option<Vec<f64>>; TPC_ANODE_WIRES],
    pad_signals: [[Option<Vec<f64>>; TPC_PAD_ROWS]; TPC_PAD_COLUMNS],
    trigger_timestamp: u32,
}
impl MainEvent {
    /// Given a run number, try to convert data banks to a [`MainEvent`]. The
    /// data banks are provided as an iterator over tuples of bank name and data
    /// slice.
    pub fn try_from_banks<'a, I>(
        run_number: u32,
        banks: I,
    ) -> Result<Self, TryMainEventFromDataBanksError>
    where
        I: IntoIterator<Item = (&'a str, &'a [u8])>,
    {
        // I didn't find another way to initialize such large arrays.
        let mut wire_signals = [(); TPC_ANODE_WIRES].map(|_| None);
        let mut pad_signals = [(); TPC_PAD_COLUMNS].map(|_| [(); TPC_PAD_ROWS].map(|_| None));
        let mut trigger_timestamp = None;
        // Need to group chunks by board and chip.
        let mut pwb_chunks_map: HashMap<_, Vec<_>> = HashMap::new();

        for (bank_name, data_slice) in banks {
            match MainEventBankName::try_from(bank_name)? {
                MainEventBankName::Alpha16(Alpha16BankName::A32(bank_name)) => {
                    let packet = AdcPacket::try_from(data_slice)?;
                    let waveform = packet.waveform();
                    if waveform.is_empty() {
                        continue;
                    }
                    // Given that the waveform is not empty, we can safely
                    // unwrap.
                    let board_id = packet.board_id().unwrap();
                    let alpha16::ChannelId::A32(channel_id) = packet.channel_id() else {
                        return Err(TryMainEventFromDataBanksError::WireBankWithBvChannel {
                            bank_name,
                        });
                    };
                    if (bank_name.board_id(), bank_name.channel_id()) != (board_id, channel_id) {
                        return Err(TryMainEventFromDataBanksError::Alpha16IdMismatch {
                            expected: (bank_name.board_id(), bank_name.channel_id()),
                            found: (board_id, channel_id),
                        });
                    }

                    let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id)?;
                    let wire_index = usize::from(wire_position);
                    if wire_signals[wire_index].is_some() {
                        return Err(TryMainEventFromDataBanksError::DuplicateWireBank {
                            bank_name,
                        });
                    } else {
                        let baseline = try_wire_baseline(run_number, wire_position)?;
                        let gain = try_wire_gain(run_number, wire_position)?;
                        wire_signals[wire_index] = Some(
                            waveform
                                .iter()
                                // Convert to i32 to avoid overflow
                                .map(|&v| f64::from(i32::from(v) - i32::from(baseline)) * gain)
                                .collect(),
                        );
                    }
                }
                MainEventBankName::Padwing(bank_name) => {
                    let chunk = Chunk::try_from(data_slice)?;
                    let key = (chunk.board_id(), chunk.after_id());
                    if key.0 != bank_name.board_id() {
                        return Err(TryMainEventFromDataBanksError::PadwingBoardIdMismatch {
                            expected: bank_name.board_id(),
                            found: key.0,
                        });
                    }

                    pwb_chunks_map.entry(key).or_default().push(chunk);
                }
                MainEventBankName::Trg(_) => {
                    let packet = TrgPacket::try_from(data_slice)?;
                    if trigger_timestamp.is_some() {
                        return Err(TryMainEventFromDataBanksError::DuplicateTrgBank);
                    } else {
                        trigger_timestamp = Some(packet.timestamp());
                    }
                }
                _ => {}
            }
        }

        for chunks in pwb_chunks_map.into_values() {
            let packet = PwbPacket::try_from(chunks)?;
            let board_id = packet.board_id();
            let after_id = packet.after_id();
            for &channel_id in packet.channels_sent() {
                if let padwing::ChannelId::Pad(pad_channel_id) = channel_id {
                    // A waveform is guaranteed to exist and not be empty if the
                    // channel was sent. It is safe to unwrap.
                    let waveform = packet.waveform_at(channel_id).unwrap();

                    let pad_position =
                        TpcPadPosition::try_new(run_number, board_id, after_id, pad_channel_id)?;
                    let pad_index = (
                        usize::from(pad_position.column),
                        usize::from(pad_position.row),
                    );
                    if pad_signals[pad_index.0][pad_index.1].is_some() {
                        return Err(TryMainEventFromDataBanksError::DuplicatePadSignal {
                            position: pad_position,
                        });
                    } else {
                        let baseline = try_pad_baseline(run_number, pad_position)?;
                        let gain = try_pad_gain(run_number, pad_position)?;
                        pad_signals[pad_index.0][pad_index.1] = Some(
                            waveform
                                .iter()
                                // Given the ranges of PWB samples, overflow is
                                // not possible.
                                .map(|&v| f64::from(v.checked_sub(baseline).unwrap()) * gain)
                                .collect(),
                        );
                    }
                }
            }
        }

        Ok(Self {
            wire_signals,
            pad_signals,
            trigger_timestamp: trigger_timestamp
                .ok_or(TryMainEventFromDataBanksError::MissingTrgBank)?,
        })
    }
    /// Return all reconstructed avalanches in the event.
    pub fn avalanches(&self) -> Vec<Avalanche> {
        // Match wire and pad signals only on columns that have both.
        let mut pad_columns = HashSet::new();
        // Deconvolution of wires needs to be done in chunks of contiguous wires.
        let mut wire_inputs = [(); TPC_ANODE_WIRES].map(|_| Vec::new());
        for range in contiguous_ranges(&self.wire_signals) {
            for (i, input) in wire_range_deconvolution(&self.wire_signals, range) {
                wire_inputs[i] = input;
                pad_columns.insert(wire_to_pad_column(i));
            }
        }
        let t_min = t_min(&wire_inputs);
        // Anything before `t_min` is assumed to be noise. Use that to remove
        // the noise in the region of interest, i.e. after `t_min`.
        // 4/5 is just an arbitrary number that leaves some wiggle room in front
        // of `t_min` to account for some jitter in the deconvolution.
        // The typical `tmin` is 100ish or greater. This just leaves about 20
        // or more time bins.
        remove_noise_after_t(&mut wire_inputs, 4 * t_min / 5);
        // We need to iterate over the pad columns in a deterministic order.
        // This is needed for complete deterministic vertex reconstruction
        // because of the `track_fitting`. Floating point arithmetic is not
        // associative, hence having SpacePoints in different orders will lead
        // to some very small differences in the final vertices.
        let pad_columns = {
            let mut temp = pad_columns.into_iter().collect::<Vec<_>>();
            temp.sort_unstable();
            temp
        };

        let mut avalanches = Vec::new();
        for column in pad_columns {
            let mut pad_inputs_column = [(); TPC_PAD_ROWS].map(|_| Vec::new());
            for (row, input) in pad_inputs_column.iter_mut().enumerate() {
                if let Some(signal) = self.pad_signals[column][row].as_ref() {
                    *input = pad_deconvolution(signal);
                }
            }

            let wire_indices = pad_column_to_wires(column);
            avalanches.extend(match_column_inputs(
                wire_indices.clone().collect::<Vec<_>>().try_into().unwrap(),
                wire_inputs[wire_indices].try_into().unwrap(),
                &pad_inputs_column,
                t_min,
            ));
        }

        avalanches
    }
    /// Return the trigger timestamp of the event. This is a counter that
    /// increments at a frequency of [`TRG_CLOCK_FREQ`].
    ///
    /// Note that this counter wraps around after a certain amount of time.
    pub fn timestamp(&self) -> u32 {
        self.trigger_timestamp
    }
}

#[cfg(test)]
mod tests;
