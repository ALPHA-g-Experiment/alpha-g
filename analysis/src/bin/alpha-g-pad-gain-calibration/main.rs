//! Gain calibration for the cathode pads.

use crate::distribution::{CumulativeDistribution, Distribution};
use crate::minimization::try_minimization;
use alpha_g_detector::midas::{EventId, PadwingBankName};
use alpha_g_detector::padwing::map::{
    TpcPadColumn, TpcPadPosition, TpcPadRow, TPC_PADS, TPC_PAD_COLUMNS, TPC_PAD_ROWS,
};
use alpha_g_detector::padwing::{ChannelId, Chunk, PwbPacket, PWB_MAX, PWB_MIN};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use midasio::file::{initial_timestamp_unchecked, run_number_unchecked};
use pgfplots::{axis::*, Engine, Picture};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

// CDF and Kolmogorov-Smirnov distance implementation
mod distribution;
// Minimization of the Kolmogorov-Smirnov distance implementation
mod minimization;
// Plotting of distributions
mod plot;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Gain calibration of the cathode pads", long_about = None)]
struct Args {
    /// Baseline calibration RON file
    #[arg(short, long, value_parser(parse_baseline_file))]
    baseline_calibration: HashMap<TpcPadPosition, i16>,
    /// MIDAS files from the calibration run.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Test the effect of a previous gain calibration on the given data files
    #[arg(short, long, value_parser(parse_gain_file))]
    previous_gain_calibration: Option<HashMap<TpcPadPosition, f64>>,
    /// Path where all output files will be saved into
    #[arg(short, long, default_value = "./", value_parser(parse_directory))]
    output_path: PathBuf,
    /// Print detailed information about errors (if any)
    #[arg(short, long)]
    verbose: bool,
}

// To get the best estimate of the gain, we need to apply the worst-case
// scenario suppression threshold. To keep only signal data, this value has to
// be large enough to suppress noise waveforms from the noisiest channel (after
// re-scaling). Furthermore, it has to be larger than
// `data_suppression_threshold` * `largest_rescaling_factor`; otherwise, the
// channels with the largest rescaling factor will have had more signal
// suppressed (during data acquisition) than the other channels.
const ARBITRARY_LARGE_SUPPRESSION_THRESHOLD: i16 = 300;

fn main() -> Result<()> {
    let args = Args::parse();
    let (run_number, mmaps) = try_valid_mmaps(args.files).context("invalid input file")?;
    let (errors_count, distributions) =
        try_amplitude_distributions(mmaps, &args.baseline_calibration, args.verbose)
            .context("failed to sample amplitude distributions")?;
    if errors_count != 0 {
        eprintln!("Warning: found `{errors_count}` error(s)/warning(s)");
    }

    let spinner = ProgressBar::new_spinner()
        .with_style(ProgressStyle::default_spinner().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    // To get the best estimate of the gain, it is best if all distributions
    // saturate and are suppressed at the same level. This is only possible if
    // we treat them all by the worst case scenario i.e. smallest saturation
    // level and largest suppression level (it is impossible to recover any
    // saturated/suppressed waveforms).
    let (min, max) = try_baseline_extrema(&args.baseline_calibration)
        .context("failed to find baseline extrema")?;
    let negative_saturation = PWB_MIN.checked_sub(min).unwrap();
    let positive_saturation = PWB_MAX.checked_sub(max).unwrap();
    // Following this `worst-case scenario`, we would need to pivot on the
    // channel with the smallest rescaling factor. We don't know which channel
    // this is, so we start with a random channel and iterate the minimization
    // with whichever channel has the smallest rescaling factor.
    // The only important thing is that the initial pivot pad is a valid key in
    // the `distributions` map. It doesn't matter which one.
    // It is safe to unwrap because `try_amplitude_distributions` has already
    // checked that the map is not empty.
    let mut pivot = *distributions.keys().next().unwrap();
    // We need to keep track of the pivot pads that have already been tried.
    // The minimization could go into an infinite loop where the same couple of
    // pivots (which are extremely similar, but they keep swapping) are tried
    // over and over again.
    let mut tried_pivots = HashSet::new();
    let gain_calibration = loop {
        spinner.set_message(format!("Minimizing KS distance... (pivot: {:?})", pivot));
        let sol = try_minimize_ks_distance(
            &distributions,
            pivot,
            negative_saturation,
            positive_saturation,
            ARBITRARY_LARGE_SUPPRESSION_THRESHOLD,
        )
        .context("failed to minimize KS distance")?;

        let (min_pad, _) = rescaling_extrema(&sol);
        if sol.get(&min_pad).unwrap() < &1.0 && tried_pivots.insert(min_pad) {
            pivot = min_pad;
        } else {
            break sol;
        }
    };
    let gain_filename = args
        .output_path
        .join(format!("pad_gain_calibration_{run_number}.ron"));
    let gain_file = File::create(&gain_filename)
        .with_context(|| format!("failed to create `{}`", gain_filename.display()))?;
    ron::ser::to_writer(gain_file, &gain_calibration).with_context(|| {
        format!(
            "failed to serialize gain calibration into `{}`",
            gain_filename.display()
        )
    })?;
    spinner.println(format!("Created `{}`", gain_filename.display()));

    spinner.set_message("Compiling PDF...");
    let picture = calibration_picture(
        &distributions,
        &gain_calibration,
        &args.previous_gain_calibration,
        negative_saturation,
        positive_saturation,
        ARBITRARY_LARGE_SUPPRESSION_THRESHOLD,
    );
    picture
        .show_pdf(Engine::PdfLatex)
        .context("failed to show PDF")?;
    spinner.finish_and_clear();

    Ok(())
}

/// Parse a baseline calibration file.
/// The file is expected to be valid RON, and should deserialize to a HashMap
/// of TpcPadPosition to (f64, f64, usize).
fn parse_baseline_file(s: &str) -> Result<HashMap<TpcPadPosition, i16>> {
    let contents = std::fs::read(s).with_context(|| format!("failed to read `{s}`"))?;
    let map: HashMap<TpcPadPosition, (f64, f64, usize)> =
        ron::de::from_bytes(&contents).with_context(|| format!("failed to deserialize `{s}`"))?;

    Ok(map
        .into_iter()
        .map(|(pad, (baseline, _, _))| (pad, baseline.round() as i16))
        .collect())
}

/// Parse a gain calibration file.
/// The file is expected to be valid RON, and should deserialize to a HashMap
/// of TpcPadPosition to f64.
fn parse_gain_file(s: &str) -> Result<HashMap<TpcPadPosition, f64>> {
    let contents = std::fs::read(s).with_context(|| format!("failed to read `{s}`"))?;
    ron::de::from_bytes(&contents).with_context(|| format!("failed to deserialize `{s}`"))
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

/// Get the amplitude distribution of all pad channels given a collection of
/// memory mapped MIDAS files.
/// Count the number of non-critical errors/warnings found.
///
/// Return an error if a memory map is not a valid MIDAS file.
/// If verbose is true, print the errors/warnings to stderr.
fn try_amplitude_distributions(
    mmaps: Vec<(PathBuf, Mmap)>,
    baselines: &HashMap<TpcPadPosition, i16>,
    verbose: bool,
) -> Result<(usize, HashMap<TpcPadPosition, Distribution>)> {
    let mut errors_count = 0;
    let mut distributions = HashMap::new();

    let bar = ProgressBar::new(mmaps.len().try_into().unwrap()).with_style(
        ProgressStyle::with_template("  Sampling [{bar:25}] {percent}%,  ETA: {eta}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar.tick();
    for (path, mmap) in mmaps {
        let file_view = midasio::FileView::try_from(&mmap[..])
            .with_context(|| format!("`{}` is not a valid MIDAS file", path.display()))?;
        let run_number = file_view.run_number();

        for event_view in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
        {
            // Need to group chunks by board and chip.
            let mut pwb_chunks_map: HashMap<_, Vec<_>> = HashMap::new();

            for bank_view in event_view
                .iter()
                .filter(|bank| PadwingBankName::try_from(bank.name()).is_ok())
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

                        let Some(baseline) = baselines.get(&pad_position) else {
                            // Any missing channel is counted as a single error at the
                            // end of sampling. Don't spam the user with warnings here
                            // for every bank.
                            continue;
                        };
                        let amplitude = waveform
                            .iter()
                            .map(|sample| sample.checked_sub(*baseline).unwrap())
                            .max_by_key(|amplitude| amplitude.abs())
                            // Waveform is not empty, so we can unwrap
                            .unwrap();
                        distributions
                            .entry(pad_position)
                            .or_insert(Distribution::new())
                            .add_sample(amplitude, 1);
                    }
                }
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    let missing_channels = TPC_PADS - distributions.len();
    ensure!(missing_channels != TPC_PADS, "no pad signals found");
    errors_count += missing_channels;
    if verbose && missing_channels > 0 {
        for row in 0..TPC_PAD_ROWS {
            let row = TpcPadRow::try_from(row).unwrap();
            for column in 0..TPC_PAD_COLUMNS {
                let column = TpcPadColumn::try_from(column).unwrap();

                let pad_position = TpcPadPosition { column, row };
                if !distributions.contains_key(&pad_position) {
                    eprintln!("Warning: no amplitude samples for `{pad_position:?}`");
                }
            }
        }
    }

    Ok((errors_count, distributions))
}

/// Try to get the minimum and maximum baseline values
/// Return an error if the map is empty.
fn try_baseline_extrema(baselines: &HashMap<TpcPadPosition, i16>) -> Result<(i16, i16)> {
    ensure!(!baselines.is_empty(), "empty baselines map");

    let values = baselines.values();
    let min = values.clone().min().unwrap();
    let max = values.max().unwrap();

    Ok((*min, *max))
}

/// Try to minimize the KS distance between the amplitude distributions of
/// cathode pads.
/// Take a given pad position as the pivot for all the other pads.
fn try_minimize_ks_distance(
    distributions: &HashMap<TpcPadPosition, Distribution>,
    pivot: TpcPadPosition,
    negative_saturation: i16,
    positive_saturation: i16,
    suppression_threshold: i16,
) -> Result<HashMap<TpcPadPosition, f64>> {
    let pivot_distribution = distributions
        .get(&pivot)
        .with_context(|| format!("no amplitude distribution for pivot `{pivot:?}`"))?;

    distributions
        .iter()
        .map(|(pad, distribution)| {
            let best_param = try_minimization(
                pivot_distribution,
                distribution,
                negative_saturation,
                positive_saturation,
                suppression_threshold,
                1.0,
            )
            .with_context(|| {
                format!("minimization failed for `{pad:?}` with `{pivot:?}` as pivot")
            })?
            .best_param
            .with_context(|| {
                format!("no best parameter for `{pad:?}` with `{pivot:?}` as pivot")
            })?;

            Ok((*pad, best_param))
        })
        .collect()
}

/// Get the TpcPadPositions with the minimum and maximum rescaliing
/// factors.
// We know that the map is not empty (guaranteed by
// `try_amplitude_distributions`).
fn rescaling_extrema(gains: &HashMap<TpcPadPosition, f64>) -> (TpcPadPosition, TpcPadPosition) {
    let mut min_gain = f64::INFINITY;
    let mut max_gain = f64::NEG_INFINITY;
    let mut min_gain_pad = None;
    let mut max_gain_pad = None;

    for (pad_position, gain) in gains {
        if gain < &min_gain {
            min_gain = *gain;
            min_gain_pad = Some(*pad_position);
        }
        if gain > &max_gain {
            max_gain = *gain;
            max_gain_pad = Some(*pad_position);
        }
    }

    (min_gain_pad.unwrap(), max_gain_pad.unwrap())
}

/// Picture to visually validate the gain calibration.
fn calibration_picture(
    distributions: &HashMap<TpcPadPosition, Distribution>,
    best_rescaling: &HashMap<TpcPadPosition, f64>,
    previous_rescaling: &Option<HashMap<TpcPadPosition, f64>>,
    negative_saturation: i16,
    positive_saturation: i16,
    suppression_threshold: i16,
) -> Picture {
    // Arbitrary best values that worked during testing. Trying anything more
    // than this makes PGFPLOTS choke.
    const POINTS_PER_LINE: usize = 50;
    const LINES_PER_PLOT: usize = 300;

    let mut picture = Picture::new();

    let mut raw_axis = Axis::new();
    raw_axis.add_key(AxisKey::Custom(String::from("name=raw")));
    raw_axis.add_key(AxisKey::Custom(String::from("ymin=0, ymax=1.1")));
    raw_axis.set_x_label("Max. Amplitude~[a.u.]");
    raw_axis.set_y_label("Cumulative Distribution");
    raw_axis.set_title("No Calibration");
    raw_axis.plots = distributions
        .iter()
        .map(|(_, distribution)| {
            let distribution = distribution
                .clone()
                .saturate(negative_saturation, positive_saturation)
                .suppress(suppression_threshold);

            CumulativeDistribution::from_distribution(&distribution).plot(POINTS_PER_LINE)
        })
        .take(LINES_PER_PLOT)
        .collect();
    picture.axes.push(raw_axis);

    let mut new_axis = Axis::new();
    new_axis.add_key(AxisKey::Custom(String::from("name=new")));
    new_axis.add_key(AxisKey::Custom(String::from("at=(raw.east)")));
    new_axis.add_key(AxisKey::Custom(String::from("anchor=west")));
    new_axis.add_key(AxisKey::Custom(String::from("xshift=30pt")));
    new_axis.add_key(AxisKey::Custom(String::from("ymin=0, ymax=1.1")));
    new_axis.set_x_label("Max. Amplitude~[a.u.]");
    new_axis.set_title("New Calibration");
    new_axis.plots = distributions
        .iter()
        .map(|(pad, distribution)| {
            let rescaling = best_rescaling.get(pad).unwrap();
            let distribution = distribution
                .clone()
                .rescale(*rescaling)
                .saturate(negative_saturation, positive_saturation)
                .suppress(suppression_threshold);

            CumulativeDistribution::from_distribution(&distribution).plot(POINTS_PER_LINE)
        })
        .take(LINES_PER_PLOT)
        .collect();
    picture.axes.push(new_axis);

    if let Some(previous_rescaling) = previous_rescaling {
        let mut old_axis = Axis::new();
        old_axis.add_key(AxisKey::Custom(String::from("at=(new.east)")));
        old_axis.add_key(AxisKey::Custom(String::from("anchor=west")));
        old_axis.add_key(AxisKey::Custom(String::from("xshift=30pt")));
        old_axis.add_key(AxisKey::Custom(String::from("ymin=0, ymax=1.1")));
        old_axis.set_x_label("Max. Amplitude~[a.u.]");
        old_axis.set_title("Previous Calibration");
        old_axis.plots = distributions
            .iter()
            .map(|(pad, distribution)| {
                let rescaling = previous_rescaling.get(pad).unwrap();
                let distribution = distribution
                    .clone()
                    .rescale(*rescaling)
                    .saturate(negative_saturation, positive_saturation)
                    .suppress(suppression_threshold);

                CumulativeDistribution::from_distribution(&distribution).plot(POINTS_PER_LINE)
            })
            .take(LINES_PER_PLOT)
            .collect();
        picture.axes.push(old_axis);
    }

    picture
}
