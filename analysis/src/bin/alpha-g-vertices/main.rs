use alpha_g_detector::midas::EventId;
use alpha_g_physics::{MainEvent, TRG_CLOCK_FREQ};
use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{
    MultiProgress, ParallelProgressIterator, ProgressBar, ProgressDrawTarget, ProgressStyle,
};
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use uom::si::length::meter;
use uom::si::time::second;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Reconstruct the annihilation vertices for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to reconstruct
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the reconstructed vertices to `OUTPUT.csv`
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

#[derive(Debug, serde::Serialize)]
struct Row {
    serial_number: u32,
    trg_time: Option<f64>,
    reconstructed_x: Option<f64>,
    reconstructed_y: Option<f64>,
    reconstructed_z: Option<f64>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (run_number, files) =
        alpha_g_analysis::sort_run_files(args.files).context("failed to sort input files")?;
    // Progress bars were flickering with the default draw target rate.
    let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(1));
    // ETA is 0 until the first file is processed. So just don't show it until
    // then.
    let tp_bar = multi_progress.add(
        ProgressBar::new(files.len().try_into().unwrap())
            .with_style(ProgressStyle::with_template("[{pos}/{len}] Processing").unwrap()),
    );
    tp_bar.tick();

    let mut rows = Vec::new();
    let mut previous_final_timestamp = None;
    for file in files {
        let contents = alpha_g_analysis::read(&file)?;
        let file_view = midasio::FileView::try_from(&contents[..])?;
        if let Some(previous_final_timestamp) = previous_final_timestamp {
            ensure!(
                file_view.initial_timestamp() - previous_final_timestamp <= 1,
                "missing file before `{}`",
                file.display()
            );
        }
        previous_final_timestamp = Some(file_view.final_timestamp());

        let pb = multi_progress.add(
            ProgressBar::new(file_view.iter().len().try_into().unwrap())
                .with_style(
                    ProgressStyle::with_template("[{bar:25}] {percent}%, ETA: {eta}    ({msg})")
                        .unwrap()
                        .progress_chars("=> "),
                )
                .with_message(format!("{}", file.display())),
        );
        rows.par_extend(
            file_view
                .into_par_iter()
                .progress_with(pb.clone())
                .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
                .map(|event| {
                    let serial_number = event.serial_number();

                    let banks = event
                        .into_iter()
                        .map(|bank| (bank.name(), bank.data_slice()));
                    match MainEvent::try_from_banks(run_number, banks) {
                        Ok(event) => (serial_number, Some(event.timestamp()), event.vertex()),
                        Err(error) => {
                            if args.verbose {
                                // Use `pb` rather than `tp_bar`. Otherwise the
                                // ETA gets all messed up because of their
                                // current implementation.
                                pb.println(format!("Error in event `{serial_number}`: {error}"));
                            }
                            (serial_number, None, None)
                        }
                    }
                }),
        );
        // I set the style here rather than right after the first tick because a
        // println above would make this new style appear before this point.
        tp_bar.set_style(
            ProgressStyle::with_template("[{pos}/{len}] Processing, ETA: {eta}").unwrap(),
        );
        tp_bar.inc(1);
    }
    tp_bar.finish_and_clear();

    let rows = rows
        .into_iter()
        .skip(args.skip)
        .scan(
            (None, 0),
            |(previous, cumulative), (serial_number, timestamp, vertex)| {
                let current = timestamp.unwrap_or(previous.unwrap_or(0));
                let delta = current.wrapping_sub(previous.unwrap_or(current));
                *previous = Some(current);
                *cumulative += u64::from(delta);

                if timestamp.is_none() {
                    Some((serial_number, None, None))
                } else {
                    Some((serial_number, Some(*cumulative), vertex))
                }
            },
        )
        .map(|(serial_number, cumulative, vertex)| Row {
            serial_number,
            trg_time: cumulative.map(|t| (t as f64 / TRG_CLOCK_FREQ).get::<second>()),
            reconstructed_x: vertex.map(|v| v.x.get::<meter>()),
            reconstructed_y: vertex.map(|v| v.y.get::<meter>()),
            reconstructed_z: vertex.map(|v| v.z.get::<meter>()),
        });

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
    .context("failed to write comment to csv")?;
    let mut wtr = csv::Writer::from_writer(wtr);
    for row in rows {
        wtr.serialize(row).context("failed to write row to csv")?;
    }
    wtr.flush().context("failed to flush csv")?;

    Ok(())
}
