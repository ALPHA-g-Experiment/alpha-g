use alpha_g_detector::midas::EventId;
use alpha_g_physics::MainEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read("example.mid")?;
    let file_view = midasio::FileView::try_from(&contents[..])?;
    let run_number = file_view.run_number();

    let main_events = file_view
        .into_iter()
        .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
    for event in main_events {
        let banks = event
            .into_iter()
            .map(|bank| (bank.name(), bank.data_slice()));
        let main_event = MainEvent::try_from_banks(run_number, banks)?;
        // If all you need is the primary vertex coordinates there is a
        // convenience method for that:
        let _vertex = main_event.vertex();
    }

    Ok(())
}
