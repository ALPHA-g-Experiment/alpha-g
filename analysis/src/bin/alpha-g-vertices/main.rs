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
#[command(version)]
/// Reconstruct the annihilation vertices for a single run
struct Args {
    /// MIDAS files from the run you want to reconstruct
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the output to `OUTPUT.csv` [default: `R<run_number>_vertices.csv`]
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Print detailed information about errors (if any)
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default, serde::Serialize)]
struct Row {
    serial_number: u32,
    trg_time: Option<f64>,
    reconstructed_x: Option<f64>,
    reconstructed_y: Option<f64>,
    reconstructed_z: Option<f64>,
}

fn main() -> Result<()> {
    // The default 2 MiB stack size for threads is not enough.
    rayon::ThreadPoolBuilder::new()
        .stack_size(4 * 1024 * 1024)
        .build_global()
        .context("failed to initialize global thread pool")?;

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
                                // observable ETA in `tp_bar` gets all messed up
                                // because this causes a `tick` and the current
                                // ETA implementation increases exponentially
                                // for slow-updating progress bars.
                                pb.println(format!("Error in event `{serial_number}`: {error}"));
                            }
                            (serial_number, None, None)
                        }
                    }
                }),
        );
        // Set the style here rather than right after the first tick because a
        // println above would make this new style appear before this point.
        tp_bar.set_style(
            ProgressStyle::with_template("[{pos}/{len}] Processing, ETA: {eta}").unwrap(),
        );
        tp_bar.inc(1);
    }
    tp_bar.finish_and_clear();

    let rows = rows.into_iter().scan(
        (None, 0),
        |(previous, cumulative), (serial_number, timestamp, vertex)| {
            // If we don't have a timestamp, it is OK to use the previous one
            // because this counter overflows every 68 seconds.
            // This will only be problematic if we go over a full minute
            // without an event, which is already impossible because DAQ has
            // a 10 seconds timeout before stopping the run.
            let current = timestamp.unwrap_or(previous.unwrap_or(0));
            let delta = current.wrapping_sub(previous.unwrap_or(current));
            *previous = Some(current);
            *cumulative += u64::from(delta);

            if timestamp.is_some() {
                Some(Row {
                    serial_number,
                    trg_time: Some((*cumulative as f64 / TRG_CLOCK_FREQ).get::<second>()),
                    reconstructed_x: vertex.map(|v| v.x.get::<meter>()),
                    reconstructed_y: vertex.map(|v| v.y.get::<meter>()),
                    reconstructed_z: vertex.map(|v| v.z.get::<meter>()),
                })
            } else {
                Some(Row {
                    serial_number,
                    ..Default::default()
                })
            }
        },
    );

    let output = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("R{run_number}_vertices")))
        .with_extension("csv");
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
        wtr.serialize(row).context("failed to write csv row")?;
    }
    wtr.flush().context("failed to flush csv data")?;

    Ok(())
}
