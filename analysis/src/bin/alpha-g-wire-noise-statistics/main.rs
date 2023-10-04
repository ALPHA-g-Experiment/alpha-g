//! Statistical analysis of the anode wire signals during a noise run.

use crate::statistics::{cov, is_noise, mean, std_dev};
use alpha_g_detector::alpha16::{
    aw_map::{TpcWirePosition, TPC_ANODE_WIRES},
    {AdcPacket, ChannelId},
};
use alpha_g_detector::midas::{
    Adc32BankName, EventId, ADC32_SUPPRESSION_ENABLE_JSON_PTR, BSC_PULSER_ENABLE_JSON_PTR,
    FIELD_WIRE_PULSER_ENABLE_JSON_PTR, PULSER_ENABLE_JSON_PTR, TRIGGER_PULSER_JSON_PTR,
    TRIGGER_SOURCES_JSON_PTR,
};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use midasio::read::file::{initial_timestamp_unchecked, run_number_unchecked, FileView};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

/// Statistics implementation.
mod statistics;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Statistical analysis of the anode wire signals during a noise run", long_about = None)]
struct Args {
    /// MIDAS files from the noise run.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Path where all output files will be saved into.
    #[arg(short, long, default_value = "./", value_parser(parse_directory))]
    output_path: PathBuf,
    /// Write the channel covariance matrix to a file.
    #[arg(long)]
    save_covariance: bool,
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

    let (errors_count, noise_samples) =
        try_noise_samples(mmaps, args.verbose).context("failed to sample noise")?;
    if errors_count != 0 {
        eprintln!("Warning: found `{errors_count}` error(s)/warning(s)");
    }

    let baseline_stats = get_baseline_statistics(&noise_samples);
    let baseline_stats = serde_json::to_string(&baseline_stats)
        .context("failed to serialize the baseline statistics")?;
    let baseline_file = args
        .output_path
        .join(format!("wire_baseline_statistics_{run_number}.json"));
    std::fs::write(&baseline_file, baseline_stats).with_context(|| {
        format!(
            "failed to write baseline statistics to `{}`",
            baseline_file.display()
        )
    })?;
    println!("Created `{}`", baseline_file.display());
    // The covariance data won't be useful to many people, so we only write it
    // to a file if the user explicitly asks for it.
    if args.save_covariance {
        let cov_matrix = get_covariance_matrix(&noise_samples);
        let cov_matrix =
            ron::to_string(&cov_matrix).context("failed to serialize the covariance matrix")?;
        let cov_file = args
            .output_path
            // JSON makes it difficult to deal with non-string keys. Not worth
            // the effort. RON is trivial to serialize and deserialize.
            .join(format!("wire_covariance_matrix_{run_number}.ron"));
        std::fs::write(&cov_file, cov_matrix).with_context(|| {
            format!(
                "failed to write channel covariance matrix to `{}`",
                cov_file.display()
            )
        })?;
        println!("Created `{}`", cov_file.display());
    }

    Ok(())
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

/// Validate the settings for a given ODB.
/// - Pulser must be enabled.
/// - Field wire pulser must be disabled.
/// - BSC pulser must be disabled.
/// - Trigger pulser must be enabled.
/// - There should be no other active trigger sources.
/// - Anode wire data suppression must be disabled.
fn validate_odb_settings(odb: &[u8]) -> Result<()> {
    let odb: Value = serde_json::from_slice(odb).context("failed to parse ODB as JSON")?;

    let pulser_enable = odb
        .pointer(PULSER_ENABLE_JSON_PTR)
        .with_context(|| format!("failed to read `{PULSER_ENABLE_JSON_PTR}` from ODB"))?;
    ensure!(pulser_enable == &json!(true), "pulser not enabled");

    let field_wire_pulser_enable = odb
        .pointer(FIELD_WIRE_PULSER_ENABLE_JSON_PTR)
        .with_context(|| {
            format!("failed to read `{FIELD_WIRE_PULSER_ENABLE_JSON_PTR}` from ODB")
        })?;
    ensure!(
        field_wire_pulser_enable == &json!(false),
        "field wire pulser not disabled"
    );

    let bsc_pulser_enable = odb
        .pointer(BSC_PULSER_ENABLE_JSON_PTR)
        .with_context(|| format!("failed to read `{BSC_PULSER_ENABLE_JSON_PTR}` from ODB"))?;
    ensure!(
        bsc_pulser_enable == &json!(false),
        "bsc pulser not disabled"
    );

    let trigger_pulser = odb
        .pointer(TRIGGER_PULSER_JSON_PTR)
        .with_context(|| format!("failed to read `{TRIGGER_PULSER_JSON_PTR}` from ODB"))?;
    ensure!(trigger_pulser == &json!(true), "trigger pulser not enabled");

    let Value::Object(trigger_sources) = odb
        .pointer(TRIGGER_SOURCES_JSON_PTR)
        .with_context(|| format!("failed to read `{TRIGGER_SOURCES_JSON_PTR}` from ODB"))?
    else {
        bail!("invalid `{TRIGGER_SOURCES_JSON_PTR}` in ODB");
    };
    let active_trigger_sources = trigger_sources
        .values()
        .filter_map(|value| value.as_bool())
        .filter(|&value| value)
        .count();
    ensure!(
        active_trigger_sources == 1,
        "found `{active_trigger_sources}` active trigger sources (expected 1)"
    );

    let adc32_suppression_enable = odb
        .pointer(ADC32_SUPPRESSION_ENABLE_JSON_PTR)
        .with_context(|| {
            format!("failed to read `{ADC32_SUPPRESSION_ENABLE_JSON_PTR}` from ODB")
        })?;
    ensure!(
        adc32_suppression_enable == &json!(false),
        "anode wire data suppression not disabled"
    );

    Ok(())
}

/// Get noise samples of all anode wire channels given a collection of memory
/// mapped MIDAS files.
/// Count the number of non-critical errors/warnings found.
///
/// Return an error if a memory map is not a valid MIDAS file, or an invalid
/// setting is found in the ODB.
/// If verbose is true, print the errors/warnings to stderr.
// Allow this complex return type because this is not a library.
// It is just convenient.
#[allow(clippy::type_complexity)]
fn try_noise_samples(
    mmaps: Vec<(PathBuf, Mmap)>,
    verbose: bool,
    // Noise samples are Option because it is important to keep samples aligned
    // in time between different channels. Hence, add None to the vector when
    // there is no sample for a given channel.
    // This is particularly important to correctly calculate the covariance
    // matrix.
) -> Result<(usize, HashMap<TpcWirePosition, Vec<Option<i16>>>)> {
    let mut errors_count = 0;
    let mut noise_samples: HashMap<_, Vec<_>> = HashMap::new();

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
        validate_odb_settings(file_view.initial_odb())
            .with_context(|| format!("invalid ODB settings in `{}`", path.display()))?;

        for event_view in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
        {
            // This temporary hash map keeps track of missing channels in the
            // current event. Helps to maintain time alignment between channels.
            let mut temp = HashMap::new();

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
                if waveform.is_empty() || !is_noise(waveform) {
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
                // No particular reason to take the sample from the middle of
                // the waveform. I just wanted to avoid the first and last
                // samples in case there is some behaviour at the edges I don't
                // understand (like it happens for the pads).
                let middle_index = (waveform.len() - 1) / 2;
                temp.insert(wire_position, waveform[middle_index]);
            }
            // Add all the noise samples found in the current event to the
            // final hash map.
            // If a channel is missing in the current event, add None to the
            // vector to keep the time alignment.
            for i in 0..TPC_ANODE_WIRES {
                let wire_position = TpcWirePosition::try_from(i).unwrap();
                let sample = temp.get(&wire_position).copied();
                noise_samples.entry(wire_position).or_default().push(sample);
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();
    // If all samples are None, it means that the channel is missing.
    // Just remove it from the hash map.
    noise_samples.retain(|_, samples| samples.iter().any(Option::is_some));

    let missing_channels = TPC_ANODE_WIRES - noise_samples.len();
    errors_count += missing_channels;
    if verbose && missing_channels > 0 {
        for wire in 0..TPC_ANODE_WIRES {
            let wire_position = TpcWirePosition::try_from(wire).unwrap();
            if !noise_samples.contains_key(&wire_position) {
                eprintln!("Warning: no noise samples for `{wire_position:?}`");
            }
        }
    }

    Ok((errors_count, noise_samples))
}

/// Get the wire baseline statistics from the noise samples.
fn get_baseline_statistics(
    noise_samples: &HashMap<TpcWirePosition, Vec<Option<i16>>>,
    // The tuple is (mean, error, number of samples).
) -> HashMap<TpcWirePosition, (f64, f64, usize)> {
    noise_samples
        .iter()
        .map(|(&wire, noise)| {
            let noise: Vec<_> = noise.iter().filter_map(|sample| *sample).collect();
            let mean = mean(&noise);
            let std_dev = std_dev(&noise);
            // Estimator of the standard error of the mean.
            // https://en.wikipedia.org/wiki/Standard_error#Estimate
            let mean_error = std_dev / (noise.len() as f64).sqrt();

            (wire, (mean, mean_error, noise.len()))
        })
        .collect()
}

/// Get the covariance matrix from the noise samples.
fn get_covariance_matrix(
    noise_samples: &HashMap<TpcWirePosition, Vec<Option<i16>>>,
) -> HashMap<(TpcWirePosition, TpcWirePosition), f64> {
    let mut covariance_matrix = HashMap::new();
    for i in 0..TPC_ANODE_WIRES {
        let wire_i = TpcWirePosition::try_from(i).unwrap();
        let Some(noise_i) = noise_samples.get(&wire_i) else {
            continue;
        };
        // The matrix is symmetric, so we only need to compute the upper
        // triangle.
        for j in i..TPC_ANODE_WIRES {
            let wire_j = TpcWirePosition::try_from(j).unwrap();
            let Some(noise_j) = noise_samples.get(&wire_j) else {
                continue;
            };
            // Only interested when both wires have a sample at the same time.
            // i.e. skip whenever there is a None in either of the two vectors.
            let (noise_i, noise_j): (Vec<_>, Vec<_>) = noise_i
                .iter()
                .zip(noise_j.iter())
                .filter_map(|(sample_i, sample_j)| {
                    if let (Some(sample_i), Some(sample_j)) = (sample_i, sample_j) {
                        Some((sample_i, sample_j))
                    } else {
                        None
                    }
                })
                .unzip();
            let covariance = cov(&noise_i, &noise_j);
            covariance_matrix.insert((wire_i, wire_j), covariance);
        }
    }

    covariance_matrix
}
