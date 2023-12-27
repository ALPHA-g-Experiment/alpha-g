use alpha_g_detector::alpha16::AdcPacket;
use alpha_g_detector::midas::{Adc16BankName, EventId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read("example.mid")?;
    let file_view = midasio::FileView::try_from(&contents[..])?;

    let main_events = file_view
        .into_iter()
        .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
    for event_view in main_events {
        let adc16_banks = event_view
            .into_iter()
            .filter(|b| Adc16BankName::try_from(b.name()).is_ok());
        for bank_view in adc16_banks {
            let packet = AdcPacket::try_from(bank_view.data_slice())?;
            // The waveform can be obtained from the packet.
            // Remember that this can be empty due to data suppression.
            let _waveform = packet.waveform();
        }
    }
    Ok(())
}
