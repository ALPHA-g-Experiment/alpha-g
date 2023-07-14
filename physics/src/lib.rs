use crate::calibration::pads::baseline::try_pad_baseline;
use crate::calibration::pads::gain::try_pad_gain;
use crate::calibration::wires::baseline::try_wire_baseline;
use crate::calibration::wires::gain::try_wire_gain;
use alpha_g_detector::alpha16::aw_map::{
    MapTpcWirePositionError, TpcWirePosition, TPC_ANODE_WIRES,
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
use alpha_g_detector::trigger::TrgPacket;
use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
use std::collections::HashMap;
use thiserror::Error;

pub use crate::calibration::pads::baseline::MapPadBaselineError;
pub use crate::calibration::pads::gain::MapPadGainError;
pub use crate::calibration::wires::baseline::MapWireBaselineError;
pub use crate::calibration::wires::gain::MapWireGainError;

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

// Extract avalanche time and wire from the anode wire signals.
mod anode_analysis;
// Extract induced signal information from the cathode pads.
mod pad_analysis;

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
}
