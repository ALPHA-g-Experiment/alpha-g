use alpha_g_detector::chronobox::chronobox_fifo;
use alpha_g_detector::midas::{ChronoboxBankName, EventId};
use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Extract the Chronobox timestamps for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the Chronobox timestamps to `OUTPUT.csv`
    #[arg(short, long)]
    output: PathBuf,
    /// Print detailed information about errors (if any)
    #[arg(short, long)]
    verbose: bool,
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

    let mut cb_buffers: BTreeMap<_, Vec<_>> = BTreeMap::new();
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

        for event_view in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Chronobox)))
        {
            for bank_view in event_view {
                let Ok(name) = ChronoboxBankName::try_from(bank_view.name()) else {
                    continue;
                };
                let data = bank_view.data_slice();

                cb_buffers
                    .entry(name.board_id.name().to_string())
                    .or_default()
                    .extend(data.iter());
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    let cb_fifos = cb_buffers
        .into_iter()
        .map(|(name, buffer)| {
            let mut input = &buffer[..];
            let fifo = chronobox_fifo(&mut input);
            ensure!(input.is_empty(), "bad FIFO data for chronobox `{name}`");
            Ok((name, fifo))
        })
        .collect::<Result<BTreeMap<_, _>>>()
        .context("failed to parse FIFO data")?;

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

    Ok(())
}
