use alpha_g_detector::midas::{EventId, PadwingBankName};
use alpha_g_detector::padwing::{AfterId, BoardId, Chunk, PwbPacket};
use midasio::read::file::FileView;
use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read("example.mid")?;
    let file_view = FileView::try_from(&contents[..])?;

    let main_events = file_view
        .into_iter()
        .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
    for event_view in main_events {
        // Need to group chunks by board and chip.
        let mut pwb_chunks_map: HashMap<(BoardId, AfterId), Vec<Chunk>> = HashMap::new();

        let padwing_banks = event_view
            .into_iter()
            .filter(|b| PadwingBankName::try_from(b.name()).is_ok());
        for bank_view in padwing_banks {
            let chunk = Chunk::try_from(bank_view.data_slice())?;
            let key = (chunk.board_id(), chunk.after_id());
            pwb_chunks_map.entry(key).or_default().push(chunk);
        }

        for chunks in pwb_chunks_map.into_values() {
            let packet = PwbPacket::try_from(chunks)?;
            for &channel_id in packet.channels_sent() {
                // A waveform is guaranteed to exist and not be empty if the
                // channel was sent. It is safe to unwrap.
                let _waveform = packet.waveform_at(channel_id).unwrap();
            }
        }
    }
    Ok(())
}
