//! Visualize the rate of TRG signals for a single run.

use crate::delta_packet::DeltaPacket;
use crate::plot::{create_plot, Figure, JOBNAME};
use alpha_g_detector::midas::{EventId, TriggerBankName};
use alpha_g_detector::trigger::TrgPacket;
use alpha_g_detector::trigger::TRG_CLOCK_FREQ;
use clap::Parser;
use memmap2::Mmap;
use midasio::read::file::FileView;
use std::error::Error;
use std::fs::{copy, File};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tempfile::tempdir;

/// Difference between TRG packets.
mod delta_packet;

/// Update and create plots based on TRG data packets rate.
mod plot;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Visualize the rate of TRG signals for a single run", long_about = None)]
// If you add a new argument that changes the behaviour of the final plot,
// remember to include this in the Hash trait below.
pub(crate) struct Args {
    /// MIDAS files from the run you want to inspect.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Time step (in seconds) between plotted points.
    #[arg(short, long, default_value = "1.0", value_parser = valid_time_step)]
    time_step: f64,
    /// Skip packets with an `output_counter` between `[1..=SKIP]`
    /// The first packet not skipped sets `t=0`
    // The default here is used to skip the initial 10 synchronization software
    // triggers.
    #[arg(long, default_value = "10", verbatim_doc_comment)]
    skip: u32,
    /// Minimum time (in seconds)
    /// Ignore all packets with a timestamp `t < min_time`
    #[arg(
        long = "min-time",
        value_name = "MIN_TIME",
        default_value = "0.0",
        value_parser(valid_time_limit),
        verbatim_doc_comment
    )]
    // Ask in seconds to the user, but parse as u64 in clock units to have this
    // validation as early as possible.
    // i.e. parse, don't validate
    min_timestamp: u64,
    /// Maximum time (in seconds)
    /// Ignore all packets with a timestamp `t > max_time`
    #[arg(
        long = "max-time",
        value_name = "MAX_TIME",
        value_parser(valid_time_limit),
        verbatim_doc_comment
    )]
    // Same as `min_time`. Parse, don't validate
    max_timestamp: Option<u64>,
    /// Include the `drift_veto_counter` in the final plot.
    #[arg(long)]
    include_drift_veto_counter: bool,
    /// Include the `pulser_counter` in the final plot.
    #[arg(long)]
    include_pulser_counter: bool,
    /// Include the `scaledown_counter` in the final plot.
    #[arg(long)]
    include_scaledown_counter: bool,
    /// Remove the `input_counter` from the final plot.
    #[arg(long)]
    remove_input_counter: bool,
    /// Remove the `output_counter` from the final plot.
    #[arg(long)]
    remove_output_counter: bool,
    /// Save the PDF plot in the `output_path`. Do not open the file.
    #[arg(long)]
    batch_mode: bool,
    /// Path where the output PDF file will be saved into when running in `batch_mode`.
    #[arg(
        short,
        long,
        default_value = "./",
        value_parser(is_directory),
        requires("batch_mode")
    )]
    output_path: PathBuf,
    /// Print detailed information about errors (if any).
    #[arg(short, long)]
    verbose: bool,
}

// The hash value of the arguments used to generate the final graph is used
// to give a unique name to the output PDF. This avoids overwriting any equal
// files generated in e.g. a bash script.
impl Hash for Args {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.files.hash(state);
        ((self.time_step * TRG_CLOCK_FREQ) as u128).hash(state);
        self.skip.hash(state);
        self.min_timestamp.hash(state);
        self.max_timestamp.hash(state);
        self.include_drift_veto_counter.hash(state);
        self.include_pulser_counter.hash(state);
        self.include_scaledown_counter.hash(state);
        self.remove_input_counter.hash(state);
        self.remove_output_counter.hash(state);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // Creating and checking all FileViews at the start is significantly
    // (2-3 times) slower than creating, validating, and sorting on the fly.
    // Each file is loaded into RAM more than once.
    // On the other hand, this allows for faster feedback on incorrect input e.g.
    // a bad last file wont make you wait a lot just to crash at the end.
    //
    // File IO is a bottleneck for this simple program (if for some reason the
    // current implementation is not fast enough), then sort and validate the
    // raw memory-mapped buffers and just create the FileViews on the fly. If it
    // is clean enough, consider pushing upstream to `midasio` crate.
    let mmaps = try_mmaps(args.files.clone())?;
    let file_views = try_file_views(mmaps.iter())?;
    check_input_file_views(&file_views)?;

    let mut previous_packet = None;
    let mut cumulative_timestamp: u64 = 0;
    let mut figures = Vec::new();
    let mut count_errors: u32 = 0;

    // Need to keep track of the `final_timestamp` of each file in order to
    // detect time between contiguous files.
    let mut previous_file_timestamp = file_views[0].initial_timestamp();
    'outer: for file in file_views {
        // This is the time (in seconds) between 2 contiguous MIDAS files.
        // Files are already sorted by timestamp, so this is guaranteed to be
        // non-negative.
        let seconds_between_files = file.initial_timestamp() - previous_file_timestamp;
        // If this is not `0`, it means that there is a missing file in between.
        // Deal with this by "starting over" and making a hole between the plots.
        if seconds_between_files != 0 {
            count_errors += 1;
            if args.verbose {
                eprintln!(
                    "Warning: missing file(s) between `{previous_file_timestamp}` and `{}`. Timestamp is no longer exact.",
                    file.initial_timestamp()
                );
            }
            // The cumulative timestamp is no longer an absolute/exact time with
            // respect to t=0. There is some small offset introduced by this
            // "jump exact amount of seconds"
            cumulative_timestamp += u64::from(seconds_between_files) * (TRG_CLOCK_FREQ as u64);
            previous_packet = None;
        }
        if previous_packet.is_none() {
            figures.push(Figure::new(cumulative_timestamp, args.time_step));
        }

        for event in file
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Main)))
        {
            for bank in event
                .into_iter()
                .filter(|bank| TriggerBankName::try_from(bank.name()).is_ok())
            {
                let packet = match TrgPacket::try_from(bank.data_slice()) {
                    Ok(packet) => packet,
                    Err(error) => {
                        count_errors += 1;
                        if args.verbose {
                            eprintln!("Error: event `{}`; {error}", event.id());
                        }
                        continue;
                    }
                };
                if packet.output_counter() <= args.skip {
                    continue;
                }
                if let Some(previous) = &previous_packet {
                    match DeltaPacket::try_from(&packet, previous) {
                        Ok(delta) => {
                            cumulative_timestamp += u64::from(delta.timestamp);
                            if let Some(max_timestamp) = args.max_timestamp {
                                if cumulative_timestamp > max_timestamp {
                                    break 'outer;
                                }
                            }
                            figures.last_mut().unwrap().update(&delta);
                        }
                        Err(error) => {
                            count_errors += 1;
                            if args.verbose {
                                eprintln!("Error: event `{}`; {error}", event.id());
                            }
                            continue;
                        }
                    }
                }
                previous_packet = Some(packet);
            }
        }
        previous_file_timestamp = file.final_timestamp();
    }
    // The final name of the plot should be a unique name based on the input
    // arguments given to the CLI. This prevents overwriting different plots.
    let output_name = format!("trg_rates_{}.pdf", {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        args.hash(&mut hasher);
        hasher.finish()
    });
    // Compile the plot in a temporary directory. This automatically cleans all
    // the auxiliary files, and handles the security vulnerability of executing
    // a file with a fixed name.
    let rand_temp_dir = tempdir()?;
    if !create_plot(&rand_temp_dir, figures, &args)?.success() {
        return Err(
            "PDF compilation failed (most likely too many points). Try a larger `time_step`".into(),
        );
    }

    let rand_temp_plot = rand_temp_dir.path().join(JOBNAME.to_string() + ".pdf");
    if args.batch_mode {
        copy(rand_temp_plot, args.output_path.join(output_name))?;
    } else {
        // My `rand_temp_dir` above will delete everything in it once it gets
        // dropped. Need to move the final plot to /tmp (to keep it alive
        // and open it).
        let output_file = std::env::temp_dir().join(output_name);
        copy(rand_temp_plot, &output_file)?;
        opener::open(output_file)?;
    }

    if count_errors != 0 {
        eprintln!("Warning: found {count_errors} error(s)/warning(s)");
    }
    Ok(())
}

/// Validate `time_step` argument.
// It has to be a positive float.
// Cannot be NaN, inf, nor 0.0.
fn valid_time_step(s: &str) -> Result<f64, String> {
    let time_step: f64 = s.parse().map_err(|_| format!("`{s}` isn't a number"))?;
    // Also ignore subnormal numbers because they would make the computer run
    // out of RAM anyways.
    if time_step.is_normal() && time_step.is_sign_positive() {
        Ok(time_step)
    } else {
        Err(format!("`{time_step}` isn't a valid time_step"))
    }
}

/// Validate `max_time` and `min_time` arguments.
// It has to be a positive float.
// Cannot be NaN, or inf.
// Has to fit into u64 (max cumulative timestamp).
fn valid_time_limit(s: &str) -> Result<u64, String> {
    let time_limit: f64 = s.parse().map_err(|_| format!("`{s}` isn't a number"))?;
    if time_limit.is_finite() && time_limit.is_sign_positive() {
        let clock_limit = (time_limit * TRG_CLOCK_FREQ) as u64;
        // Casting saturates to the maximum value of the integer type
        if clock_limit != u64::MAX {
            Ok(clock_limit)
        } else {
            Err(format!(
                "`{time_limit}` is larger than the maximum possible time limit"
            ))
        }
    } else {
        Err(format!("`{time_limit}` isn't a valid time limit"))
    }
}

/// Parse `--output-path` flag as valid directory
fn is_directory(s: &str) -> Result<PathBuf, String> {
    let path: PathBuf = s.into();
    if path.is_dir() {
        Ok(path)
    } else {
        Err(String::from("path is not pointing at a directory on disk"))
    }
}

/// Try to get a vector of memory maps from a collection of paths. Return an
/// error if there is a problem opening the file or creating the Mmap.
// This function should preserve the order of the Mmaps (same as input paths)
// in order to be able to provide feedback in `try_file_views` by the index
// of which file failed.
fn try_mmaps(file_names: impl IntoIterator<Item = PathBuf>) -> Result<Vec<Mmap>, String> {
    file_names
        .into_iter()
        .map(|path| {
            let file =
                File::open(&path).map_err(|_| format!("unable to open `{}`", path.display()))?;
            unsafe { Mmap::map(&file) }
                .map_err(|_| format!("unable to memory map `{}`", path.display()))
        })
        .collect()
}

/// Try to get a vector of MIDAS file views from a collection of memory maps.
/// Return an error if there is a problem creating a FileView from the memory
/// map.
fn try_file_views<'a>(mmaps: impl Iterator<Item = &'a Mmap>) -> Result<Vec<FileView<'a>>, String> {
    let mut file_views = mmaps
        .enumerate() // Include index to give some information about which file
        // failed to create a FileView
        .map(|(index, mmap)| {
            FileView::try_from(&mmap[..])
                .map_err(|_| format!("unable to FileView file index `{index}`"))
        })
        .collect::<Result<Vec<FileView>, String>>()?;

    file_views.sort_unstable_by_key(|file| file.initial_timestamp());
    Ok(file_views)
}

/// Check that all files satisfy the conditions required to produce a correct
/// result.
// 1. All files belong to the same run number.
// 2. There are no duplicate files.
fn check_input_file_views(file_views: &Vec<FileView>) -> Result<(), String> {
    if file_views.len() > 1 {
        if file_views
            .iter()
            .any(|&f| f.run_number() != file_views[0].run_number())
        {
            return Err("found files from multiple run numbers".to_string());
        }
        // The vector is already sorted, only check contiguous elements for
        // a duplicate initial timestamp
        for pair in file_views.windows(2) {
            if pair[0].initial_timestamp() == pair[1].initial_timestamp() {
                return Err("found files with same initial timestamp".to_string());
            }
        }
    }
    Ok(())
}
