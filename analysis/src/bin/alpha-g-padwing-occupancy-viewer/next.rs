use alpha_g_detector::midas::{EventId, PadwingBankName};
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
    /// Data bank doesn't make a correct [`Chunk`].
    #[error("bad data bank")]
    BadDataBank(#[from] TryChunkFromSliceError),
    /// [`Chunk`]s don't make a correct [`PwbPacket`].
    #[error("bad chunks")]
    BadChunks(#[from] TryPwbPacketFromChunksError),
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

        let main_events = file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
        for event_view in main_events {
            let mut pwb_chunks_map: HashMap<(BoardId, AfterId), Vec<Chunk>> = HashMap::new();

            let padwing_banks = event_view
                .into_iter()
                .filter(|b| PadwingBankName::try_from(b.name()).is_ok());
            for bank_view in padwing_banks {
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
            if sender
                .send(Ok(Packet {
                    pwb_packets,
                    serial_number: event_view.serial_number(),
                    run_number: file_view.run_number(),
                }))
                .is_err()
            {
                return;
            }
        }
    }
    let _ = sender.send(Err(TryNextPacketError::AllConsumed));
}
