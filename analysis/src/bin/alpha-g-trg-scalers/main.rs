use alpha_g_detector::midas::{EventId, TriggerBankName};
use alpha_g_detector::trigger::TrgPacket;
use alpha_g_physics::TRG_CLOCK_FREQ;
use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::path::PathBuf;
use uom::si::time::second;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Extract the TRG scalers for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the TRG scalers to `OUTPUT.csv`
    #[arg(short, long)]
    output: PathBuf,
    /// Ignore the first `SKIP` number of events
    /// The first event not skipped sets `t=0`
    // The default here is used to skip the initial 10 synchronization software
    // triggers.
    #[arg(long, default_value = "10", verbatim_doc_comment)]
    skip: usize,
    /// Print detailed information about errors (if any)
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default, serde::Serialize)]
struct Row {
    serial_number: u32,
    trg_time: Option<f64>,
    input: Option<u32>,
    drift_veto: Option<u32>,
    scaledown: Option<u32>,
    pulser: Option<u32>,
    output: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (_, files) =
        alpha_g_analysis::sort_run_files(args.files).context("failed to sort input files")?;

    let bar = ProgressBar::new(files.len().try_into().unwrap()).with_style(
        ProgressStyle::with_template("  Processing [{bar:25}] {percent}%,  ETA: {eta}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar.tick();

    let mut rows = Vec::new();
    let mut previous_final_timestamp = None;
    for file in files {
        let contents = alpha_g_analysis::read(&file)
            .with_context(|| format!("failed to read `{}`", file.display()))?;
        let file_view = midasio::FileView::try_from(&contents[..])
            .with_context(|| format!("failed to parse `{}`", file.display()))?;
        if let Some(previous_final_timestamp) = previous_final_timestamp {
            ensure!(
                file_view.initial_timestamp() - previous_final_timestamp <= 1,
                "missing file before `{}`",
                file.display()
            );
        }
        previous_final_timestamp = Some(file_view.final_timestamp());

        rows.extend(
            file_view
                .into_iter()
                .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
                .map(|event| {
                    let serial_number = event.serial_number();

                    let [trg_bank] = event
                        .into_iter()
                        .filter(|bank| TriggerBankName::try_from(bank.name()).is_ok())
                        .collect::<Vec<_>>()[..]
                    else {
                        if args.verbose {
                            bar.println(format!(
                                "Error in event `{serial_number}`: bad number of trg data banks"
                            ));
                        }
                        return (serial_number, None);
                    };

                    match TrgPacket::try_from(trg_bank.data_slice()) {
                        Ok(trg_packet) => (serial_number, Some(trg_packet)),
                        Err(error) => {
                            if args.verbose {
                                bar.println(format!("Error in event `{serial_number}`: {error}"));
                            }
                            (serial_number, None)
                        }
                    }
                }),
        );
        bar.inc(1);
    }
    bar.finish_and_clear();

    let rows = rows.into_iter().skip(args.skip).scan(
        (None, 0),
        |(previous, cumulative), (serial_number, trg_packet)| {
            let timestamp = trg_packet.map(|p| p.timestamp());
            // If we can't get a timestamp, it is OK to use the previous one
            // because this counter overflows every 70ish seconds.
            // This will only be problematic if we go a full 70 seconds
            // without an event, which is already impossible because DAQ has
            // a 10 seconds timeout before stopping the run.
            let current = timestamp.unwrap_or(previous.unwrap_or(0));
            let delta = current.wrapping_sub(previous.unwrap_or(current));
            *previous = Some(current);
            *cumulative += u64::from(delta);

            if let Some(trg_packet) = trg_packet {
                Some(Row {
                    serial_number,
                    trg_time: Some((*cumulative as f64 / TRG_CLOCK_FREQ).get::<second>()),
                    input: Some(trg_packet.input_counter()),
                    drift_veto: trg_packet.drift_veto_counter(),
                    scaledown: trg_packet.scaledown_counter(),
                    pulser: Some(trg_packet.pulser_counter()),
                    output: Some(trg_packet.output_counter()),
                })
            } else {
                Some(Row {
                    serial_number,
                    ..Default::default()
                })
            }
        },
    );

    let output = args.output.with_extension("csv");
    let mut wtr = std::fs::File::create(&output)
        .with_context(|| format!("failed to create `{}`", output.display()))?;
    eprintln!("Created `{}`", output.display());
    wtr.write_all(
        format!(
            "# {} {}\n# {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            std::env::args().collect::<Vec<_>>().join(" ")
        )
        .as_bytes(),
    )
    .context("failed to write csv header")?;
    let mut wtr = csv::Writer::from_writer(wtr);
    for row in rows {
        wtr.serialize(row)
            .context("failed to write row to csv data")?;
    }
    wtr.flush().context("failed to flush csv data")?;

    Ok(())
}
