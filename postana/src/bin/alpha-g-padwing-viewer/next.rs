use alpha_g_detector::midas::EventId;
use alpha_g_detector::midas::PadwingBankName;
use alpha_g_detector::padwing::{
    ChannelId, Chunk, PwbPacket, TryChunkFromSliceError, TryPwbPacketFromChunksError,
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
    /// Input file is not a MIDAS file.
    #[error("not a MIDAS file")]
    FailedFileView(#[from] TryFileViewFromSliceError),
    /// Data bank doesn't make a [`Chunk`].
    #[error("bad data bank")]
    BadDataBank(#[from] TryChunkFromSliceError),
    /// [`Chunk`]s don't make a [`PwbPacket`].
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
    /// PWB packet from a single AFTER chip.
    pub pwb_packet: PwbPacket,
    /// Current channel sent, for which to display the waveform.
    // This was done to delegate the "iterating" over sent channels to the
    // worker thread rather than the Cursive user_data.
    // It just makes everything simpler.
    pub channel_id: ChannelId,
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
                sender
                    .send(Err(error.into()))
                    .expect("receiver disconnected on \"failed to open\"");
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(error) => {
                sender
                    .send(Err(error.into()))
                    .expect("receiver disconnected on \"failed to mmap\"");
                continue;
            }
        };
        let file_view = match FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(error) => {
                sender
                    .send(Err(error.into()))
                    .expect("receiver disconnected on \"failed to FileView\"");
                continue;
            }
        };
        let main_events = file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
        for event_view in main_events {
            let mut pwb_chunks_map = HashMap::new();

            let padwing_banks = event_view
                .into_iter()
                .filter(|b| PadwingBankName::try_from(b.name()).is_ok());
            for bank_view in padwing_banks {
                let chunk = match Chunk::try_from(bank_view.data_slice()) {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        sender
                            .send(Err(error.into()))
                            .expect("receiver disconnected on \"BadDataBank\"");
                        continue;
                    }
                };

                let key = (chunk.board_id(), chunk.after_id());
                pwb_chunks_map.entry(key).or_insert(Vec::new()).push(chunk);
            }

            for chunks in pwb_chunks_map.into_values() {
                let pwb_packet = match PwbPacket::try_from(chunks) {
                    Ok(packet) => packet,
                    Err(error) => {
                        sender
                            .send(Err(error.into()))
                            .expect("receiver disconnected on \"BadChunks\"");
                        continue;
                    }
                };
                // The main point of this application is to iterate over
                // waveforms. Delegate the waveform iteration to the worker
                // thread rather than the Cursive user_data. It simplifies all
                // the filter logic, next logic, etc.
                for &channel_id in pwb_packet.channels_sent() {
                    sender
                        .send(Ok(Packet {
                            pwb_packet: pwb_packet.clone(),
                            channel_id,
                        }))
                        .expect("receiver disconnected on \"data packet\"");
                }
            }
        }
    }
    sender
        .send(Err(TryNextPacketError::AllConsumed))
        .expect("receiver disconnected on \"all consumed\"");
}
