use alpha_g_detector::midas::EventId;
use alpha_g_physics::MainEvent;
use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{
    MultiProgress, ParallelProgressIterator, ProgressBar, ProgressDrawTarget, ProgressStyle,
};
use midasio::file::{initial_timestamp_unchecked, run_number_unchecked};
use rayon::prelude::*;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Reconstruct the annihilation vertices for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to reconstruct.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Ignore the first `SKIP` number of events.
    /// The first event not skipped sets `t=0`
    // The default here is used to skip the initial 10 synchronization software
    // triggers.
    #[arg(long, default_value = "10", verbatim_doc_comment)]
    skip: usize,
    /// Print detailed information about errors (if any).
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = sort_and_validate_files(args.files).context("invalid input file")?;
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
        let contents = std::fs::read(&file)?;
        let file_view = midasio::FileView::try_from(&contents[..])?;
        let run_number = file_view.run_number();
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

    Ok(())
}

// Sort the files by their initial odb dump timestamp.
// Also check that all files correspond to the same run number and that there
// are no repeated initial timestamps.
fn sort_and_validate_files(files: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut files = files
        .into_iter()
        .map(|path| {
            let mut file = std::fs::File::open(&path)
                .with_context(|| format!("failed to open `{}`", path.display()))?;
            // The first 12 bytes contain both the run number and the initial
            // timestamp.
            let mut buffer = [0; 12];
            file.read_exact(&mut buffer)
                .with_context(|| format!("failed to read `{}`", path.display()))?;

            let run_number = run_number_unchecked(&buffer)
                .with_context(|| format!("failed to parse run number from `{}`", path.display()))?;
            let initial_timestamp = initial_timestamp_unchecked(&buffer).with_context(|| {
                format!(
                    "failed to parse initial timestamp from `{}`",
                    path.display()
                )
            })?;

            Ok((path, run_number, initial_timestamp))
        })
        .collect::<Result<Vec<_>>>()?;

    assert!(!files.is_empty());
    let expected_run_number = files[0].1;
    for (file, run_number, _) in &files {
        ensure!(
            run_number == &expected_run_number,
            "bad run number in `{}` (expected `{}`, found `{}`)",
            file.display(),
            expected_run_number,
            run_number
        );
    }

    files.sort_unstable_by_key(|(_, _, initial_timestamp)| *initial_timestamp);
    // They are sorted, so it is enough to check that consecutive files have
    // different timestamps.
    for window in files.windows(2) {
        let [(path0, _, time0), (path1, _, time1)] = window else {
            unreachable!()
        };
        ensure!(
            time0 != time1,
            "duplicate initial timestamp in `{}` and `{}`",
            path0.display(),
            path1.display()
        );
    }

    Ok(files.into_iter().map(|(path, _, _)| path).collect())
}
