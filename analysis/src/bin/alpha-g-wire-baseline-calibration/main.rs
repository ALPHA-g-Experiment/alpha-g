//! Generate a calibration file with the baseline of all anode wire channels.

use crate::plot::calibration_picture;
use alpha_g_detector::alpha16::{
    aw_map::TpcWirePosition,
    {AdcPacket, ChannelId},
};
use alpha_g_detector::midas::{Adc32BankName, EventId};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use midasio::read::file::{initial_timestamp_unchecked, run_number_unchecked, FileView};
use pgfplots::Engine;
use std::collections::{HashMap, HashSet};
use std::fs::{copy, File};
use std::path::PathBuf;

/// Generate plots of calibration data.
mod plot;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Generate a calibration file with the baseline of all anode wire channels")]
struct Args {
    /// MIDAS files from the calibration run.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Save the PDF plot in the `output_path`. Do not open the file.
    #[arg(long)]
    batch_mode: bool,
    /// Path where calibration file will be saved into.
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

    let (errors_count, noise_samples) =
        try_noise_samples(mmaps, args.verbose).context("failed to sample noise")?;

    let calibration: HashMap<_, _> = noise_samples
        .into_iter()
        .map(|(wire, noise)| {
            let mean = mean(&noise);
            let std_dev = std_dev(&noise);
            // Estimator of the standard error of the mean.
            let mean_error = std_dev / (noise.len() as f64).sqrt();
            // The baseline in this calibration will be used as an integer
            // offset, so we round the mean to the nearest integer.
            // Round the error up to the nearest integer. This is a conservative
            // estimate of the error.
            // Furthermore, storing the calibration as integers rather than
            // floats reduces the size of the calibration file significantly.
            (wire, (mean.round() as i16, mean_error as u16 + 1))
        })
        .collect();

    let output_name = format!("wire_baseline_calibration_{}", run_number);

    let json_output = args.output_path.join(format!("{output_name}.json"));
    let json_string =
        serde_json::to_string(&calibration).context("failed to serialize the calibration")?;
    std::fs::write(&json_output, json_string).with_context(|| {
        format!(
            "failed to write calibration data to `{}`",
            json_output.display()
        )
    })?;

    let spinner = ProgressBar::new_spinner()
        .with_message("Compiling PDF...")
        .with_style(ProgressStyle::default_spinner().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    // Regardless of `batch_mode`, always first compile into `/tmp` to keep
    // any intermediate files there.
    let tmp_pdf = calibration_picture(run_number, &calibration)
        .to_pdf(std::env::temp_dir(), &output_name, Engine::PdfLatex)
        .context("failed to compile PDF")?;
    if args.batch_mode {
        let pdf_output = args.output_path.join(format!("{output_name}.pdf"));
        copy(&tmp_pdf, &pdf_output)
            .with_context(|| {
                format!(
                    "failed to copy the contents from `{}` to `{}`",
                    tmp_pdf.display(),
                    pdf_output.display()
                )
            })
            .context("failed to save PDF")?;
    } else {
        opener::open(&tmp_pdf)
            .with_context(|| format!("failed to open `{}`", tmp_pdf.display()))?;
    }
    spinner.finish_and_clear();

    if errors_count != 0 {
        eprintln!("Warning: found `{errors_count}` error(s)/warning(s)");
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
// Do not validate the entire MIDAS format because it is too expensive.
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

/// Get noise samples of all anode wire channels given a collection of memory
/// mapped MIDAS files.
/// Count the number of non-critical errors/warnings found.
///
/// Return an error if a memory map is not a valid MIDAS file.
/// If verbose is true, print the errors/warnings to stderr.
fn try_noise_samples(
    mmaps: Vec<(PathBuf, Mmap)>,
    verbose: bool,
) -> Result<(usize, HashMap<TpcWirePosition, Vec<i16>>)> {
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
                noise_samples
                    .entry(wire_position)
                    .or_default()
                    .push(waveform[middle_index]);
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    Ok((errors_count, noise_samples))
}

/// Determine if the waveform is noise.
// Those which are not noise are not considered to estimate the baseline.
fn is_noise(_waveform: &[i16]) -> bool {
    true
}

/// Calculate the mean of a slice of `i16`.
fn mean(slice: &[i16]) -> f64 {
    slice.iter().map(|&x| f64::from(x)).sum::<f64>() / slice.len() as f64
}

/// Calculate the standard deviation of a slice of `i16`.
fn std_dev(slice: &[i16]) -> f64 {
    let mean = mean(slice);
    let sum = slice
        .iter()
        .map(|&x| f64::from(x))
        .fold(0.0, |acc, x| acc + (x - mean).powi(2));
    (sum / (slice.len() - 1) as f64).sqrt()
}
