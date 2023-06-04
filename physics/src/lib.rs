use crate::calibration::wires::baseline::try_wire_baseline;
use crate::calibration::wires::gain::try_wire_gain;
use alpha_g_detector::alpha16::aw_map::{
    MapTpcWirePositionError, TpcWirePosition, TPC_ANODE_WIRES,
};
use alpha_g_detector::alpha16::{self, AdcPacket, TryAdcPacketFromSliceError};
use alpha_g_detector::midas::{
    Adc32BankName, Alpha16BankName, MainEventBankName, ParseMainEventBankNameError,
};
use alpha_g_detector::padwing::map::MapTpcPadPositionError;
use alpha_g_detector::padwing::{TryChunkFromSliceError, TryPwbPacketFromChunksError};
use alpha_g_detector::trigger::TryTrgPacketFromSliceError;
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
    /// An anode wire data bank has a BV channel_id.
    #[error("anode wire data bank `{bank_name:?}` has a BV channel_id")]
    WireBankWithBvChannel { bank_name: Adc32BankName },
    /// Duplicate anode wire data banks were found.
    #[error("duplicate anode wire data banks `{bank_name:?}`")]
    DuplicateWireBank { bank_name: Adc32BankName },
    /// The chunk data from a Padwing bank is invalid.
    #[error("bad padwing chunk data")]
    BadPadwingChunk(#[from] TryChunkFromSliceError),
    /// The chunks from a Padwing board are invalid.
    #[error("bad padwing data")]
    BadPadwing(#[from] TryPwbPacketFromChunksError),
    /// The data from the TRG board is invalid.
    #[error("bad trg data")]
    BadTrg(#[from] TryTrgPacketFromSliceError),
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
    wire_signals: [Option<Vec<f64>>; TPC_ANODE_WIRES],
}
impl MainEvent {
    /// Given a run number, try to convert data banks to a [`MainEvent`]. The
    /// data banks are provided as an iterator over tuples of bank name and data
    /// slice.
    pub fn try_from_data_banks<'a, I>(
        run_number: u32,
        banks: I,
    ) -> Result<Self, TryMainEventFromDataBanksError>
    where
        I: IntoIterator<Item = (&'a str, &'a [u8])>,
    {
        let mut wire_signals = [(); TPC_ANODE_WIRES].map(|_| None);

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
                            bank_name: bank_name,
                        });
                    };
                    let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id)?;
                    let wire_index = usize::from(wire_position);
                    if wire_signals[wire_index].is_some() {
                        return Err(TryMainEventFromDataBanksError::DuplicateWireBank {
                            bank_name: bank_name,
                        });
                    } else {
                        let baseline = try_wire_baseline(run_number, wire_position)?;
                        let gain = try_wire_gain(run_number, wire_position)?;
                        wire_signals[wire_index] = Some(
                            waveform
                                .iter()
                                .map(|&v| f64::from(v - baseline) * gain)
                                .collect(),
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(Self { wire_signals })
    }
}
