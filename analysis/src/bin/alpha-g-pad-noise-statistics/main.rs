//! Statistical analysis of the pad signals during a noise run.

use alpha_g_detector::midas::{
    EventId, PadwingBankName, BSC_PULSER_ENABLE_JSON_PTR, FIELD_WIRE_PULSER_ENABLE_JSON_PTR,
    PULSER_ENABLE_JSON_PTR, PWB_FORCE_CHANNELS_JSON_PTR, TRIGGER_PULSER_JSON_PTR,
    TRIGGER_SOURCES_JSON_PTR,
};
use alpha_g_detector::padwing::map::{
    TpcPadColumn, TpcPadPosition, TpcPadRow, TPC_PADS, TPC_PAD_COLUMNS, TPC_PAD_ROWS,
};
use alpha_g_detector::padwing::{ChannelId, Chunk, PwbPacket};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use midasio::read::file::{initial_timestamp_unchecked, run_number_unchecked, FileView};
use serde_json::{json, Value};
use statrs::statistics::Statistics;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Statistical analysis of the pad signals during a noise run", long_about = None)]
struct Args {
    /// MIDAS files from the noise run.
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
    let (run_number, mmaps) = try_valid_mmaps(args.files).context("invalid input file")?;

    let (errors_count, noise_samples) =
        try_noise_samples(mmaps, args.verbose).context("failed to sample noise")?;
    if errors_count != 0 {
        eprintln!("Warning: found `{errors_count}` error(s)/warning(s)");
    }

    let baseline_stats = get_baseline_statistics(&noise_samples);
    // JSON makes it difficult to deal with non-string keys
    // (e.g. TpcPadPosition). Not worth the effort; use RON instead.
    let baseline_filename = args
        .output_path
        .join(format!("pad_baseline_statistics_{run_number}.ron"));
    let baseline_file = File::create(&baseline_filename)
        .with_context(|| format!("failed to create `{}`", baseline_filename.display()))?;
    ron::ser::to_writer(baseline_file, &baseline_stats).with_context(|| {
        format!(
            "failed to serialize baseline statistics into `{}`",
            baseline_filename.display()
        )
    })?;
    println!("Created `{}`", baseline_filename.display());

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
// Return the memory maps in a tuple with their corresponding file path to keep
// context (helpful for error messages).
fn try_valid_mmaps(
    file_names: impl IntoIterator<Item = PathBuf>,
) -> Result<(u32, Vec<(PathBuf, Mmap)>)> {
    let mut run_number = None;
    let mut timestamps = HashSet::new();

    let mmaps = file_names
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
        .collect::<Result<_>>()?;

    Ok((run_number.unwrap(), mmaps))
}

/// Validate the settings for a given ODB.
/// - Pulser must be enabled.
/// - Field wire pulser must be disabled.
/// - BSC pulser must be disabled.
/// - Trigger pulser must be enabled.
/// - There should be no other active trigger sources.
/// - Pad data suppression must be disabled.
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

    let pwb_force_channels = odb
        .pointer(PWB_FORCE_CHANNELS_JSON_PTR)
        .with_context(|| format!("failed to read `{PWB_FORCE_CHANNELS_JSON_PTR}` from ODB"))?;
    ensure!(
        pwb_force_channels == &json!(true),
        "pad data suppression not disabled"
    );

    Ok(())
}

/// Determine if a waveform is noise.
fn is_noise(_waveform: &[i16]) -> bool {
    true
}

/// Get noise samples of all pad channels given a collection of memory mapped
/// MIDAS files.
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
) -> Result<(usize, HashMap<TpcPadPosition, Vec<Option<i16>>>)> {
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
            // This temporary HashMap helps to keep track of missing channels in
            // the current event. This is important to maintain time alignment
            // between channels.
            let mut temp = HashMap::new();

            // Need to group chunks by board and chip.
            let mut pwb_chunks_map: HashMap<_, Vec<_>> = HashMap::new();

            for bank_view in event_view
                .into_iter()
                .filter(|b| PadwingBankName::try_from(b.name()).is_ok())
            {
                let chunk = match Chunk::try_from(bank_view.data_slice()) {
                    Ok(chunk) => chunk,
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
                let key = (chunk.board_id(), chunk.after_id());
                pwb_chunks_map.entry(key).or_default().push(chunk);
            }

            for chunks in pwb_chunks_map.into_values() {
                let packet = match PwbPacket::try_from(chunks) {
                    Ok(packet) => packet,
                    Err(error) => {
                        errors_count += 1;
                        if verbose {
                            bar.println(format!(
                                "Error: event `{}`, {error}",
                                event_view.serial_number(),
                            ));
                        }
                        continue;
                    }
                };
                let board_id = packet.board_id();
                let after_id = packet.after_id();
                for &channel_id in packet.channels_sent() {
                    if let ChannelId::Pad(pad_channel_id) = channel_id {
                        let pad_position =
                            TpcPadPosition::try_new(run_number, board_id, after_id, pad_channel_id)
                                .context("pad position mapping failed")?;
                        // A waveform is guaranteed to exist and not be empty if
                        // the channel was sent. It is safe to unwrap.
                        let waveform = packet.waveform_at(channel_id).unwrap();
                        if !is_noise(waveform) {
                            continue;
                        }
                        // No particular reason to take the sample from the
                        // middle of the waveform. I just want to avoid the
                        // first and last last few samples (SCA analog mux
                        // switching noise. See elog 3310).
                        let middle_index = (waveform.len() - 1) / 2;
                        temp.insert(pad_position, waveform[middle_index]);
                    }
                }
            }
            // Add all the noise samples found in the current event to the
            // final hash map.
            // If a channel is missing in the current event, add None to the
            // vector to keep the time alignment.
            for row in 0..TPC_PAD_ROWS {
                let row = TpcPadRow::try_from(row).unwrap();
                for column in 0..TPC_PAD_COLUMNS {
                    let column = TpcPadColumn::try_from(column).unwrap();
                    let pad_position = TpcPadPosition { column, row };
                    let sample = temp.get(&pad_position).copied();
                    noise_samples.entry(pad_position).or_default().push(sample);
                }
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();
    // If all samples are None, it means that the channel is missing.
    // Just remove it from the hash map.
    noise_samples.retain(|_, samples| samples.iter().any(Option::is_some));

    let missing_channels = TPC_PADS - noise_samples.len();
    errors_count += missing_channels;
    if verbose && missing_channels > 0 {
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            for column in 0..TPC_PAD_COLUMNS {
                let column = TpcPadColumn::try_from(column).unwrap();
                let pad_position = TpcPadPosition { column, row };
                if !noise_samples.contains_key(&pad_position) {
                    eprintln!("Warning: no noise samples for `{pad_position:?}`");
                }
            }
        }
    }

    Ok((errors_count, noise_samples))
}

/// Get the pad baseline statistics from the noise samples.
fn get_baseline_statistics(
    noise_samples: &HashMap<TpcPadPosition, Vec<Option<i16>>>,
    // The tuple is (mean, error, number of samples).
) -> HashMap<TpcPadPosition, (f64, f64, usize)> {
    noise_samples
        .iter()
        .map(|(&pad_position, samples)| {
            let noise: Vec<f64> = samples
                .iter()
                .filter_map(|&sample| sample.map(f64::from))
                .collect();
            let mean = noise.clone().mean();
            let std_dev = noise.clone().std_dev();
            let mean_error = std_dev / (noise.len() as f64).sqrt();

            (pad_position, (mean, mean_error, noise.len()))
        })
        .collect()
}
