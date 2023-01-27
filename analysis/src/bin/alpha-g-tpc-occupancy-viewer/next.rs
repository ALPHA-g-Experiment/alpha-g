use alpha_g_detector::alpha16::{AdcPacket, ChannelId, TryAdcPacketFromSliceError};
use alpha_g_detector::midas::{Adc32BankName, EventId, PadwingBankName};
use alpha_g_detector::padwing::{
    AfterId, BoardId, Chunk, PwbPacket, TryChunkFromSliceError, TryPwbPacketFromChunksError,
};
use memmap2::Mmap;
use midasio::read::file::{FileView, TryFileViewFromSliceError};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::SyncSender;
use thiserror::Error;

/// The error type returned when obtaining the next [`Packet`] fails.
#[derive(Error, Debug)]
pub enum TryNextPacketError {
    /// Error opening an input file.
    #[error("failed to open input file")]
    FailedOpen(#[from] std::io::Error),
    /// Input file is not a valid MIDAS file.
    #[error("not a valid MIDAS file")]
    FailedFileView(#[from] TryFileViewFromSliceError),
    /// PWB data bank doesn't make a correct [`Chunk`].
    #[error("bad pwb data bank")]
    BadPwbDataBank(#[from] TryChunkFromSliceError),
    /// [`Chunk`]s don't make a correct [`PwbPacket`].
    #[error("bad chunks")]
    BadChunks(#[from] TryPwbPacketFromChunksError),
    /// ADC data bank doesn't make a correct [`AdcPacket`].
    #[error("bad adc data bank")]
    BadAdcDataBank(#[from] TryAdcPacketFromSliceError),
    /// An [`AdcPacket`] (presumably from the anode wires) contains a BV (A16)
    /// channel id.
    #[error("aw packet contains a BV channel id")]
    BadAdcPacket,
    /// All input files have been consumed.
    #[error("consumed all input files")]
    AllConsumed,
}

/// Data that the worker thread is trying to collect from the MIDAS file with
/// every iteration of "next".
// All these have to be owned to avoid lifetime restrictions of Cursive's
// user_data.
#[derive(Clone, Debug)]
pub struct Packet {
    /// All the [`PwbPacket`]s from the event.
    pub pwb_packets: Vec<PwbPacket>,
    /// All the [`AdcPacket`]s from the event.
    pub adc_packets: Vec<AdcPacket>,
    /// Serial number of the MIDAS event.
    pub serial_number: u32,
    /// Run number, required to map the pads.
    pub run_number: u32,
}

/// Worker function that iterates through MIDAS files and tries to send
/// [`Packet`]s to the main thread.
pub fn worker<P>(
    sender: SyncSender<Result<Packet, TryNextPacketError>>,
    file_names: impl IntoIterator<Item = P>,
) where
    P: AsRef<Path>,
{
    for file_name in file_names {
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
        };
        let file_view = match FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
        };

        for event_view in file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)))
        {
            let mut adc_packets = Vec::new();
            let mut pwb_chunks_map: HashMap<(BoardId, AfterId), Vec<Chunk>> = HashMap::new();

            for bank_view in event_view {
                if Adc32BankName::try_from(bank_view.name()).is_ok() {
                    let adc_packet = match AdcPacket::try_from(bank_view.data_slice()) {
                        Ok(adc_packet) => adc_packet,
                        Err(error) => {
                            if sender.send(Err(error.into())).is_err() {
                                return;
                            }
                            continue;
                        }
                    };
                    // Checked here before sending to allow the main thread to
                    // assume all packets are from the anode wires (it unwraps
                    // the channel id).
                    if let ChannelId::A16(_) = adc_packet.channel_id() {
                        if sender.send(Err(TryNextPacketError::BadAdcPacket)).is_err() {
                            return;
                        }
                        continue;
                    }

                    adc_packets.push(adc_packet);
                } else if PadwingBankName::try_from(bank_view.name()).is_ok() {
                    let chunk = match Chunk::try_from(bank_view.data_slice()) {
                        Ok(chunk) => chunk,
                        Err(error) => {
                            if sender.send(Err(error.into())).is_err() {
                                return;
                            }
                            continue;
                        }
                    };

                    let key = (chunk.board_id(), chunk.after_id());
                    pwb_chunks_map.entry(key).or_default().push(chunk);
                }
            }

            let mut pwb_packets = Vec::new();
            for chunks in pwb_chunks_map.into_values() {
                let pwb_packet = match PwbPacket::try_from(chunks) {
                    Ok(packet) => packet,
                    Err(error) => {
                        if sender.send(Err(error.into())).is_err() {
                            return;
                        }
                        continue;
                    }
                };
                pwb_packets.push(pwb_packet);
            }

            let packet = Packet {
                pwb_packets,
                adc_packets,
                serial_number: event_view.serial_number(),
                run_number: file_view.run_number(),
            };
            if sender.send(Ok(packet)).is_err() {
                return;
            }
        }
    }
    let _ = sender.send(Err(TryNextPacketError::AllConsumed));
}
