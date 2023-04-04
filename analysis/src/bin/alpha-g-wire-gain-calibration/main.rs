//! Gain calibration of the anode wires.

use crate::distribution::Distribution;
use alpha_g_detector::alpha16::{
    aw_map::{TpcWirePosition, TPC_ANODE_WIRES},
    AdcPacket, ChannelId,
};
use alpha_g_detector::midas::{Adc32BankName, EventId};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use midasio::read::file::{initial_timestamp_unchecked, run_number_unchecked, FileView};
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

    let (errors_count, distributions) =
        try_amplitude_distributions(mmaps, &args.baseline_calibration, args.verbose)
            .context("failed to sample amplitude distributions")?;
    if errors_count != 0 {
        eprintln!("Warning: found `{errors_count}` error(s)/warning(s)");
    }

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

/// Get the amplitude distribution of all anode wires given a collection of
/// memory mapped MIDAS files.
/// Count the number of non-critical errors/warnings found.
///
/// Return an error if a memory map is not a valid MIDAS file.
/// If verbose is true, print the errors/warnings to stderr.
fn try_amplitude_distributions(
    mmaps: Vec<(PathBuf, Mmap)>,
    baselines: &HashMap<TpcWirePosition, i16>,
    verbose: bool,
) -> Result<(usize, HashMap<TpcWirePosition, Distribution>)> {
    let mut errors_count = 0;
    let mut distributions = HashMap::new();

    let bar = ProgressBar::new(mmaps.len().try_into().unwrap()).with_style(
        ProgressStyle::with_template("  Sampling [{bar:25}] {percent}%,  ETA: {eta}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar.tick();
    for (path, mmap) in mmaps {
        let file_view = FileView::try_from(&mmap[..])
            .with_context(|| format!("`{}` is not a valid MIDAS file", path.display()))?;
        let run_number = file_view.run_number();

        for event_view in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
        {
            for bank_view in event_view
                .into_iter()
                .filter(|bank| Adc32BankName::try_from(bank.name()).is_ok())
            {
                let packet = match AdcPacket::try_from(bank_view.data_slice()) {
                    Ok(packet) => packet,
                    Err(error) => {
                        errors_count += 1;
                        if verbose {
                            bar.println(format!(
                                "Error: event `{}`, bank `{}`, {error}",
                                event_view.serial_number(),
                                bank_view.name(),
                            ));
                        }
                        continue;
                    }
                };
                let waveform = packet.waveform();
                if waveform.is_empty() {
                    continue;
                }
                // Given that waveform is not empty, we can unwrap safely.
                let board_id = packet.board_id().unwrap();
                let ChannelId::A32(channel_id) = packet.channel_id() else {
                    errors_count += 1;
                    if verbose {
                        bar.println(format!(
                            "Error: anode wire packet `{}` with BV channel_id in event `{}`",
                            bank_view.name(),
                            event_view.serial_number()
                            ));
                    }
                    continue;
                };
                let wire_position = TpcWirePosition::try_new(run_number, board_id, channel_id)
                    .context("wire position mapping failed")?;

                let Some(baseline) = baselines.get(&wire_position) else {
                    // Any missing channel is counted as a single error at the
                    // end of sampling. Don't spam the user with warnings here
                    // for every bank.
                    continue;
                };
                let amplitude = waveform
                    .iter()
                    // Convert to i32 to avoid overflow.
                    .map(|sample| i32::from(*sample) - i32::from(*baseline))
                    .max_by_key(|amplitude| amplitude.abs())
                    // We checked that waveform is not empty, so we can unwrap
                    .unwrap();
                distributions
                    .entry(wire_position)
                    .or_insert(Distribution::new())
                    .add_sample(amplitude, 1);
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    let missing_channels = TPC_ANODE_WIRES - distributions.len();
    errors_count += missing_channels;
    if verbose && missing_channels > 0 {
        for wire in 0..TPC_ANODE_WIRES {
            let wire_position = TpcWirePosition::try_from(wire).unwrap();
            if !distributions.contains_key(&wire_position) {
                eprintln!("Warning: no amplitude samples for `{wire_position:?}`");
            }
        }
    }

    Ok((errors_count, distributions))
}
