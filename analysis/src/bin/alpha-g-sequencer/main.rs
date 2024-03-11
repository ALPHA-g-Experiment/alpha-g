use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Extract the sequencer data for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the sequencer data to `OUTPUT.csv`
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Debug, Default, serde::Serialize)]
struct Row {
    midas_timestamp: u32,
    header: String,
    xml: String,
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

        bar.inc(1);
    }
    bar.finish_and_clear();

    Ok(())
}
