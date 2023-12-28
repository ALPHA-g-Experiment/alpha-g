use alpha_g_detector::midas::{EventId, PadwingBankName, PWB_SUPPRESSION_THRESHOLD_JSON_PTR};
use alpha_g_detector::padwing::{AfterId, BoardId, ChannelId, Chunk, PwbPacket};
use anyhow::{anyhow, Context, Result};
use memmap2::Mmap;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::SyncSender;

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
    /// Run number, required to get the appropriate suppression baseline.
    pub run_number: u32,
    /// Data suppression threshold.
    // Value is the same for all channels (reset, FPN, and pads)
    pub suppression_threshold: Option<f64>,
}

/// Worker function that iterates through MIDAS files and tries to send
/// [`Packet`]s to the main thread.
pub fn worker<P>(sender: SyncSender<Result<Packet>>, file_names: impl IntoIterator<Item = P>)
where
    P: AsRef<Path>,
{
    for file_name in file_names {
        let file = match File::open(&file_name) {
            Ok(file) => file,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!("failed to open `{}`", file_name.as_ref().display())
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!("failed to memory map `{}`", file_name.as_ref().display())
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let file_view = match midasio::FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!(
                            "`{}` is not a valid MIDAS file",
                            file_name.as_ref().display()
                        )
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let odb = serde_json::from_slice::<Value>(file_view.initial_odb());
        let suppression_threshold = if let Ok(odb) = odb {
            odb.pointer(PWB_SUPPRESSION_THRESHOLD_JSON_PTR)
                .and_then(|v| v.as_f64())
        } else {
            None
        };

        let main_events = file_view
            .iter()
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
                        if sender
                            .send(Err(error).with_context(|| {
                                format!(
                                    "bad padwing data bank at event `{}`",
                                    event_view.serial_number()
                                )
                            }))
                            .is_err()
                        {
                            return;
                        }
                        continue;
                    }
                };

                let key = (chunk.board_id(), chunk.after_id());
                pwb_chunks_map.entry(key).or_default().push(chunk);
            }

            for chunks in pwb_chunks_map.into_values() {
                let pwb_packet = match PwbPacket::try_from(chunks) {
                    Ok(packet) => packet,
                    Err(error) => {
                        if sender
                            .send(Err(error).with_context(|| {
                                format!("bad chunks at event `{}`", event_view.serial_number())
                            }))
                            .is_err()
                        {
                            return;
                        }
                        continue;
                    }
                };
                // The main point of this application is to iterate over
                // waveforms. Delegate the waveform iteration to the worker
                // thread rather than the Cursive user_data. It simplifies all
                // the filter logic, next logic, etc.
                for &channel_id in pwb_packet.channels_sent() {
                    let packet = Packet {
                        pwb_packet: pwb_packet.clone(),
                        channel_id,
                        run_number: file_view.run_number(),
                        suppression_threshold,
                    };
                    if sender.send(Ok(packet)).is_err() {
                        return;
                    }
                }
            }
        }
    }
    let _ = sender.send(Err(anyhow!("no more files")));
}
