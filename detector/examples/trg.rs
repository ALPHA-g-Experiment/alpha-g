use alpha_g_detector::midas::{EventId, TriggerBankName};
use alpha_g_detector::trigger::TrgPacket;
use midasio::read::file::FileView;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read("example.mid")?;
    let file_view = FileView::try_from(&contents[..])?;

    let main_events = file_view
        .into_iter()
        .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
    for event_view in main_events {
        let trg_banks = event_view
            .into_iter()
            .filter(|b| TriggerBankName::try_from(b.name()).is_ok());
        for bank_view in trg_banks {
            let _packet = TrgPacket::try_from(bank_view.data_slice())?;
            // You can do anything here with the trigger packet.
        }
    }
    Ok(())
}
