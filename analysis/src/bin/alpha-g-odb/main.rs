use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
/// Get an ODB dump from a MIDAS file
struct Args {
    /// Path to the MIDAS file to parse
    file: PathBuf,
    /// Write the output to `OUTPUT.json` [default:
    /// `R<run_number>_<initial|final>_odb.json`]
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Get the final ODB dump instead of the initial (default) one
    #[arg(long)]
    r#final: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let contents = alpha_g_analysis::read(&args.file)
        .with_context(|| format!("failed to read `{}`", args.file.display()))?;
    let file_view = midasio::FileView::try_from(&contents[..])
        .with_context(|| format!("failed to parse `{}`", args.file.display()))?;
    let odb = if args.r#final {
        file_view.final_odb()
    } else {
        file_view.initial_odb()
    };
    let odb = std::str::from_utf8(odb).context("failed to parse ODB as UTF-8")?;

    let output = args
        .output
        .unwrap_or_else(|| {
            let run_number = file_view.run_number();
            let suffix = if args.r#final { "final" } else { "initial" };
            PathBuf::from(format!("R{run_number}_{suffix}_odb"))
        })
        .with_extension("json");
    std::fs::write(
        &output,
        format!(
            "# {} {}\n# {}\n{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            std::env::args().collect::<Vec<_>>().join(" "),
            odb
        )
        .as_bytes(),
    )
    .context("failed to write ODB dump")?;
    eprintln!("Created `{}`", output.display());

    Ok(())
}
