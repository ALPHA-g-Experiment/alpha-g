//! Visualize the rate of TRG signals for a single run.

use crate::delta_packet::DeltaPacket;
use crate::plot::Figure;
use alpha_g_detector::midas::{EventId, TriggerBankName};
use alpha_g_detector::trigger::TrgPacket;
use alpha_g_detector::trigger::TRG_CLOCK_FREQ;
use clap::Parser;
use memmap2::Mmap;
use midasio::read::file::FileView;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

/// Difference between TRG packets.
mod delta_packet;

/// Update and create plots based on TRG data packets rate.
mod plot;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Visualize the rate of TRG signals for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Time step (in seconds) between points
    #[arg(short, long, default_value = "1.0")]
    time_step: f64,
    /// Skip packets with an `output_counter` between `[1..=SKIP]`
    // The default here is used to skip the initial 10 synchronization software
    // triggers.
    #[arg(long, default_value = "10")]
    skip: u32,
    /// Print detailed information about errors (if any).
    #[arg(short, long)]
    verbose: bool,
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
    let mmaps = try_mmaps(args.files)?;
    let file_views = try_file_views(mmaps.iter())?;
    check_input_file_views(&file_views)?;

    let mut previous_packet = None;
    let mut cumulative_timestamp: u64 = 0;
    let mut figures = Vec::new();
    let mut count_errors: u32 = 0;

    // Need to keep track of the `final_timestamp` of each file in order to
    // detect time between contiguous files.
    let mut previous_file_timestamp = file_views[0].initial_timestamp();
    for file in file_views {
        // This is the time (in seconds) between 2 contiguous MIDAS files
        // Files are already sorted by timestamp, so this is guaranteed to be
        // non-negative.
        let seconds_between_files = file.initial_timestamp() - previous_file_timestamp;
        // If this is not `0`, it means that there is a missing file in between.
        // Deal with this by "starting over" and making a hole between the plots.
        if seconds_between_files != 0 {
            count_errors += 1;
            if args.verbose {
                eprintln!(
                    "Warning: missing file(s) between `{previous_file_timestamp}` and `{}`",
                    file.initial_timestamp()
                );
            }
            cumulative_timestamp = cumulative_timestamp
                .checked_add(
                    u64::from(seconds_between_files)
                        .checked_mul(TRG_CLOCK_FREQ as u64)
                        .expect("impossible time between files"),
                )
                .expect("impossible cumulative time");
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
                            eprintln!("Error: event `{}`. {error}", event.id());
                        }
                        continue;
                    }
                };
                if packet.output_counter() <= args.skip {
                    continue;
                }
                if let Some(previous) = &previous_packet {
                    let delta_packet = DeltaPacket::try_from(&packet, previous);
                    match delta_packet {
                        Ok(delta) => {
                            cumulative_timestamp += u64::from(delta.timestamp);
                            figures.last_mut().unwrap().update(&delta);
                        }
                        Err(error) => {
                            count_errors += 1;
                            if args.verbose {
                                eprintln!("Error: event `{}`. {error}", event.id());
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

    if count_errors != 0 {
        eprintln!("Warning: found {count_errors} error(s)");
    }

    Ok(())
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
/// result:
/// 1. All files belong to the same run number.
/// 2. There are no duplicate files.
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
