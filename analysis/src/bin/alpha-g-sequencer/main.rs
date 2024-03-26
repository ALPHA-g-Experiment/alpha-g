use alpha_g_detector::midas::{EventId, Seq2BankName};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
/// Extract the sequencer data for a single run
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the output to `OUTPUT.csv` [default: `R<run_number>_sequencer.csv`]
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Default, serde::Serialize)]
struct Row {
    serial_number: u32,
    midas_timestamp: u32,
    header: String,
    xml: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (run_number, files) =
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

        for event in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Sequencer2)))
        {
            let serial_number = event.serial_number();
            let midas_timestamp = event.timestamp();

            let [seq_bank] = event.into_iter().collect::<Vec<_>>()[..] else {
                bail!("unexpected number of sequencer data banks");
            };
            ensure!(
                Seq2BankName::try_from(seq_bank.name()).is_ok(),
                "unexpected sequencer bank name"
            );

            let data = std::str::from_utf8(
                seq_bank
                    .data_slice()
                    .strip_suffix(b"\x00")
                    .context("failed to remove trailing 0 from sequencer data")?,
            )
            .context("failed to parse data as UTF-8")?;
            let index = data.find('<').context("failed to find XML start tag")?;
            let (header, xml) = data.split_at(index);

            let row = Row {
                serial_number,
                midas_timestamp,
                header: header.trim_end().to_string(),
                xml: xml.to_string(),
            };
            rows.push(row);
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    let output = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("R{run_number}_sequencer")))
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
        wtr.serialize(row)
            .context("failed to write row to csv data")?;
    }
    wtr.flush().context("failed to flush csv data")?;

    Ok(())
}
