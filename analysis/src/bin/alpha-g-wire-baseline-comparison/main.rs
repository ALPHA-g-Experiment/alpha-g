//! Compare anode wire baseline calibration files to determine if there is a
//! statistically significant difference between them.
//!
//! Generate an updated calibration file if there is a significant difference.

use crate::statistics::tost;
use alpha_g_detector::alpha16::aw_map::{TpcWirePosition, TPC_ANODE_WIRES};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

mod statistics;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Compare anode wire baseline calibration files", long_about = None)]
struct Args {
    /// Previous calibration file.
    #[arg(long, value_parser(parse_calibration_file))]
    // The tuple represents (baseline, error, number of samples).
    previous: HashMap<TpcWirePosition, (f64, f64, usize)>,
    /// New calibration file used to identify potential changes.
    #[arg(long, value_parser(parse_calibration_file))]
    discovery: HashMap<TpcWirePosition, (f64, f64, usize)>,
    /// Path where output files will be saved to.
    #[arg(short, long, default_value = "./", value_parser(parse_directory))]
    output_path: PathBuf,
}

// This is the minimum difference in baseline that is considered significant.
// This value was determined by looking at the distribution of data suppression
// baselines (they change about +/- 300 across all channels).
// Previous data analysis used the data suppression baseline, and variations
// within that range were considered insignificant.
// Detecting a change in the baseline of 75 is an improvement with respect
// to using the data suppression baseline. Variations within this 75 range are
// considered insignificant.
const TOST_LIMIT: f64 = 75.0;

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
        // The values in `new_overwrite` are `Option` to keep track of channels
        // that stopped working (i.e. present in previous, but absent in
        // discovery). These channels are overwritten with None.
        let new_overwrite: HashMap<_, _> = changed_channels
            .iter()
            .map(|&wire| (wire, args.discovery.get(&wire).copied()))
            .collect();
        let mut new_complete = args.previous.clone();
        for (wire, value) in new_overwrite.iter() {
            match value {
                Some(value) => new_complete.insert(*wire, *value),
                None => new_complete.remove(wire),
            };
        }

        let new_overwrite = serde_json::to_string(&new_overwrite)
            .context("failed to serialize calibration update")?;
        let overwrite_file = args.output_path.join("wire_baseline_overwrite.json");
        std::fs::write(&overwrite_file, new_overwrite).with_context(|| {
            format!(
                "failed to write update calibration to `{}`",
                overwrite_file.display()
            )
        })?;
        println!("Created `{}`", overwrite_file.display());

        let new_complete =
            serde_json::to_string(&new_complete).context("failed to serialize new calibration")?;
        let complete_file = args.output_path.join("wire_baseline_complete.json");
        std::fs::write(&complete_file, new_complete).with_context(|| {
            format!(
                "failed to write new calibration to `{}`",
                complete_file.display()
            )
        })?;
        println!("Created `{}`", complete_file.display());
    }

    Ok(())
}

/// Parse a baseline calibration file.
/// The file is expected to be valid JSON, and should deserialize to a HashMap
/// of TpcWirePosition to (f64, f64, usize).
fn parse_calibration_file(s: &str) -> Result<HashMap<TpcWirePosition, (f64, f64, usize)>> {
    let contents = std::fs::read(s).with_context(|| format!("failed to read `{s}`"))?;
    serde_json::from_slice(&contents).with_context(|| format!("failed to deserialize `{s}`"))
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
/// Return a vector with the TpcWirePosition of the wires that have a
/// statistically significant difference.
fn find_different_channels(
    previous: &HashMap<TpcWirePosition, (f64, f64, usize)>,
    discovery: &HashMap<TpcWirePosition, (f64, f64, usize)>,
) -> Vec<TpcWirePosition> {
    let mut changed_wires = Vec::new();

    for i in 0..TPC_ANODE_WIRES {
        let wire = TpcWirePosition::try_from(i).unwrap();
        let previous_sample = previous.get(&wire);
        let discovery_sample = discovery.get(&wire);

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
                    changed_wires.push(wire);
                }
            }
            // Channel is not working, but was not working before
            // i.e. nothing changed
            (None, None) => (),
            // A new channel stopped/started working.
            _ => changed_wires.push(wire),
        }
    }

    changed_wires
}
