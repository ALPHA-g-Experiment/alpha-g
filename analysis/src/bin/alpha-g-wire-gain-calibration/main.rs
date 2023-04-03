//! Gain calibration of the anode wires.

use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use memmap2::Mmap;
use midasio::read::file::{initial_timestamp_unchecked, run_number_unchecked};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

// CDF and Kolmogorov-Smirnov distance implementation
mod distribution;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Gain calibration of the anode wires", long_about = None)]
struct Args {
    /// Baseline calibration JSON file
    #[arg(short, long, value_parser(parse_baseline_file))]
    baseline_calibration: HashMap<TpcWirePosition, i16>,
    /// MIDAS files from the calibration run.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Path where all output files will be saved into.
    #[arg(short, long, default_value = "./", value_parser(parse_directory))]
    output_path: PathBuf,
    /// Print detailed information about errors (if any).
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mmaps = try_valid_mmaps(args.files).context("invalid input file")?;
    // It is safe to unwrap because this has already been checked in
    // `try_valid_mmaps`.
    let run_number = run_number_unchecked(&mmaps[0].1).unwrap();
    Ok(())
}

/// Parse a baseline calibration file.
/// The file is expected to be valid JSON, and should deserialize to a HashMap
/// of TpcWirePosition to (f64, f64, usize).
fn parse_baseline_file(s: &str) -> Result<HashMap<TpcWirePosition, i16>> {
    let contents = std::fs::read(s).with_context(|| format!("failed to read `{s}`"))?;
    let map: HashMap<TpcWirePosition, (f64, f64, usize)> = serde_json::from_slice(&contents)
        .with_context(|| format!("failed to deserialize `{s}`"))?;

    Ok(map
        .into_iter()
        .map(|(wire, (baseline, _, _))| (wire, baseline.round() as i16))
        .collect())
}

/// Parse `--output-path` flag as valid directory
fn parse_directory(s: &str) -> Result<PathBuf> {
    let path: PathBuf = s.into();
    if path.is_dir() {
        Ok(path)
    } else {
        bail!(
            "`{}` is not pointing to a directory on disk",
            path.display()
        )
    }
}

/// Try to get a vector of valid memory maps from a collection of paths. Ensure
/// that all the memory maps are valid:
/// - All belong to the same run number.
/// - There are no duplicates (by timestamp).
// Do not validate the entire MIDAS format (here) because it is too expensive.
// Instead, only validate the run number and the timestamp.
//
// Return tuple to keep some context about each memory map.
// This is useful for error reporting.
fn try_valid_mmaps(file_names: impl IntoIterator<Item = PathBuf>) -> Result<Vec<(PathBuf, Mmap)>> {
    let mut run_number = None;
    let mut timestamps = HashSet::new();

    file_names
        .into_iter()
        .map(|path| {
            let file = File::open(&path)
                .with_context(|| format!("failed to open `{}`", path.display()))?;
            let mmap = unsafe { Mmap::map(&file) }
                .with_context(|| format!("failed to memory map `{}`", path.display()))?;

            let current_run_number = run_number_unchecked(&mmap).with_context(|| {
                format!("failed to read run number from `{}`", path.display())
            })?;
            if let Some(run_number) = run_number {
                ensure!(
                    run_number == current_run_number,
                    "bad run number in `{}` (expected `{run_number}`, found `{current_run_number}`)",
                    path.display()
                );
            } else {
                run_number = Some(current_run_number);
            }

            let initial_timestamp = initial_timestamp_unchecked(&mmap).with_context(|| {
                format!("failed to read initial timestamp from `{}`", path.display())
            })?;
            ensure!(
                timestamps.insert(initial_timestamp),
                "duplicate initial timestamp `{initial_timestamp}` in `{}`",
                path.display()
            );

            Ok((path, mmap))
        })
        .collect()
}
