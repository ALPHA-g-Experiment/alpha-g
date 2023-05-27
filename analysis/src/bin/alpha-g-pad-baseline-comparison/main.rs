//! Compare pad baseline calibration files to determine if there is a
//! statistically significant difference between them.
//!
//! Generate an updated calibration file if there is a significant difference.

use crate::statistics::tost;
use alpha_g_detector::padwing::map::{
    TpcPadColumn, TpcPadPosition, TpcPadRow, TPC_PAD_COLUMNS, TPC_PAD_ROWS,
};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

mod statistics;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Compare pad baseline calibration files", long_about = None)]
struct Args {
    /// Previous calibration file.
    #[arg(long, value_parser(parse_calibration_file))]
    // The tuple represents (baseline, error, number of samples).
    previous: HashMap<TpcPadPosition, (f64, f64, usize)>,
    /// New calibration file used to identify potential changes.
    #[arg(long, value_parser(parse_calibration_file))]
    discovery: HashMap<TpcPadPosition, (f64, f64, usize)>,
    /// Path where output files will be saved to.
    #[arg(short, long, default_value = "./", value_parser(parse_directory))]
    output_path: PathBuf,
}

// This is the minimum difference in baseline that is considered significant.
const TOST_LIMIT: f64 = 9.0;

fn main() -> Result<()> {
    let args = Args::parse();
    ensure!(
        args.previous != args.discovery,
        "both calibration files are identical"
    );

    let changed_channels = find_different_channels(&args.previous, &args.discovery);
    if changed_channels.is_empty() {
        println!("No significant changes were found");
    } else {
        println!("Found {} significant change(s)", changed_channels.len());
        // The values in `new_update` are `Option` to keep track of channels
        // that stopped working (i.e. present in previous, but absent in
        // discovery). These channels are overwritten with None.
        let new_update: HashMap<_, _> = changed_channels
            .iter()
            .map(|&pad| (pad, args.discovery.get(&pad).copied()))
            .collect();
        let mut new_complete = args.previous.clone();
        for (pad, value) in new_update.iter() {
            match value {
                Some(value) => new_complete.insert(*pad, *value),
                None => new_complete.remove(pad),
            };
        }

        let update_filename = args.output_path.join("pad_baseline_update.ron");
        let update_file = File::create(&update_filename)
            .with_context(|| format!("failed to create `{}`", update_filename.display()))?;
        ron::ser::to_writer(update_file, &new_update).with_context(|| {
            format!(
                "failed to serialize update calibration to `{}`",
                update_filename.display()
            )
        })?;
        println!("Created `{}`", update_filename.display());

        let complete_filename = args.output_path.join("pad_baseline_complete.ron");
        let complete_file = File::create(&complete_filename)
            .with_context(|| format!("failed to create `{}`", complete_filename.display()))?;
        ron::ser::to_writer(complete_file, &new_complete).with_context(|| {
            format!(
                "failed to serialize complete calibration to `{}`",
                complete_filename.display()
            )
        })?;
        println!("Created `{}`", complete_filename.display());
    }
    Ok(())
}

/// Parse a baseline calibration file.
/// The file is expected to be valid RON, and should deserialize to a HashMap
/// of TpcPadPosition to (f64, f64, usize).
fn parse_calibration_file(s: &str) -> Result<HashMap<TpcPadPosition, (f64, f64, usize)>> {
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

/// Compare two baseline calibration files to determine if there is a
/// statistically significant difference between them.
/// Return a vector with the TpcPadPosition of the pads that have a
/// statistically significant difference.
fn find_different_channels(
    previous: &HashMap<TpcPadPosition, (f64, f64, usize)>,
    discovery: &HashMap<TpcPadPosition, (f64, f64, usize)>,
) -> Vec<TpcPadPosition> {
    let mut changed_pads = Vec::new();

    for row in 0..TPC_PAD_ROWS {
        let row = TpcPadRow::try_from(row).unwrap();
        for column in 0..TPC_PAD_COLUMNS {
            let column = TpcPadColumn::try_from(column).unwrap();
            let pad = TpcPadPosition { row, column };

            let previous_sample = previous.get(&pad);
            let discovery_sample = discovery.get(&pad);

            match (previous_sample, discovery_sample) {
                // Channel is, and was, working
                (Some(previous_sample), Some(discovery_sample)) => {
                    let p_value = tost(*previous_sample, *discovery_sample, TOST_LIMIT);
                    // p-value less than 0.05 indicates a statistically significant
                    // effect to reject the null hypothesis.
                    // In our case, the null hypothesis is that the difference
                    // between the two sample means is greater than TOST_LIMIT.
                    // i.e.
                    // low p-value    ->    means are equivalent within TOST_LIMIT
                    if p_value >= 0.05 {
                        changed_pads.push(pad);
                    }
                }
                // Channel is not working, but was not working before
                // i.e. nothing changed
                (None, None) => (),
                // A new channel stopped/started working.
                _ => changed_pads.push(pad),
            }
        }
    }

    changed_pads
}
