// READ CAREFULLY ALL THE COMMENTS.
// If this program reports an error, the solution is most likely to analyze the
// Chronobox data manually instead of patching this program.
use alpha_g_detector::chronobox::{
    chronobox_fifo, EdgeType, FifoEntry, TimestampCounter, WrapAroundMarker, TIMESTAMP_BITS,
};
use alpha_g_detector::midas::{ChronoboxBankName, EventId};
use alpha_g_physics::chronobox::TIMESTAMP_CLOCK_FREQ;
use anyhow::{ensure, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use uom::si::f64::Time;
use uom::si::time::second;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Extract the Chronobox timestamps for a single run", long_about = None)]
struct Args {
    /// MIDAS files from the run you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Write the Chronobox timestamps to `OUTPUT.csv`
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Debug, Default, serde::Serialize)]
struct Row {
    board: String,
    channel: u8,
    leading_edge: bool,
    chronobox_time: Option<f64>,
}

fn chronobox_time(
    tsc: TimestampCounter,
    previous_marker: Option<WrapAroundMarker>,
    next_marker: Option<WrapAroundMarker>,
) -> Option<Time> {
    if let (Some(previous), Some(next)) = (previous_marker, next_marker) {
        if (previous.wrap_around_counter() + 1 == next.wrap_around_counter())
            && (previous.timestamp_top_bit != next.timestamp_top_bit)
        {
            // The epoch counter increments twice per wrap-around (because of
            // half markers). Furthermore, the counter 0 corresponds to the
            // first half wrap-around marker.
            let epoch_counter = (previous.wrap_around_counter() + 1) / 2;
            let timestamp_counter = tsc.timestamp();
            let top_bit = (timestamp_counter >> (TIMESTAMP_BITS - 1)) == 1;
            if top_bit != previous.timestamp_top_bit {
                let time = u64::from(timestamp_counter)
                    + u64::from(epoch_counter) * (1u64 << TIMESTAMP_BITS);
                Some(time as f64 / TIMESTAMP_CLOCK_FREQ)
            } else {
                // In theory, this case can be handled by knowing an appropriate
                // threshold for how close a timestamp can be to a marker to
                // arrive the FIFO in the wrong order.
                None
            }
        } else {
            None
        }
    } else {
        // If the timestamp is not "sandwiched" between two markers, it is best
        // to not assume what the missing marker is. This can naturally happen
        // with the last batch of timestamps, but these are most likely not
        // needed anyway.
        None
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (_, files) =
        alpha_g_analysis::sort_run_files(args.files).context("failed to sort input files")?;

    let bar = ProgressBar::new(files.len().try_into().unwrap()).with_style(
        ProgressStyle::with_template("  Processing [{bar:25}] {percent}%,  ETA: {eta}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar.tick();

    let mut cb_buffers: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut previous_final_timestamp = None;
    for file in files {
        let contents = alpha_g_analysis::read(&file)
            .with_context(|| format!("failed to read `{}`", file.display()))?;
        let file_view = midasio::FileView::try_from(&contents[..])
            .with_context(|| format!("failed to parse `{}`", file.display()))?;
        if let Some(previous_final_timestamp) = previous_final_timestamp {
            ensure!(
                file_view.initial_timestamp() - previous_final_timestamp <= 1,
                "missing file before `{}`",
                file.display()
            );
        }
        previous_final_timestamp = Some(file_view.final_timestamp());

        for event_view in file_view
            .into_iter()
            .filter(|event| matches!(EventId::try_from(event.id()), Ok(EventId::Chronobox)))
        {
            for bank_view in event_view {
                let Ok(name) = ChronoboxBankName::try_from(bank_view.name()) else {
                    continue;
                };
                let data = bank_view.data_slice();

                cb_buffers
                    .entry(name.board_id.name().to_string())
                    .or_default()
                    .extend(data.iter());
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();
    // The Chronobox data is probably the most "complicated" to deal with given
    // that it is designed to be unpacked with a state machine across events.
    // If anything isn't 100% as expected, it is better to fail completely
    // and analyze the data manually. There is no guaranteed way to recover with
    // complete certainty (it is not correct to e.g. skip until the next marker;
    // we could mistakenly find a word in the middle of a scalers block, etc.).
    let cb_fifos = cb_buffers
        .into_iter()
        .map(|(name, buffer)| {
            let mut input = &buffer[..];
            let mut fifo = chronobox_fifo(&mut input);
            ensure!(input.is_empty(), "bad FIFO data for chronobox `{name}`");
            // Replicate `alphasoft`'s behavior and ignore anything until the
            // 0th marker counter.
            let epoch_0_index = fifo
                .iter()
                .position(|entry| match entry {
                    FifoEntry::WrapAroundMarker(marker) => marker.wrap_around_counter() == 0,
                    _ => false,
                })
                .context("missing epoch 0 marker in chronobox `{name}`")?;
            let fifo = fifo.split_off(epoch_0_index);
            // This is important, otherwise the `epoch_counter` in
            // `chronobox_time` will be wrong every other marker.
            ensure!(
                {
                    let FifoEntry::WrapAroundMarker(marker) = fifo[0] else {
                        unreachable!();
                    };
                    !marker.timestamp_top_bit
                },
                "bad first marker in chronobox `{name}`"
            );

            Ok((name, fifo))
        })
        .collect::<Result<BTreeMap<_, _>>>()
        .context("failed to parse FIFO data")?;

    let output = args.output.with_extension("csv");
    let mut wtr = std::fs::File::create(&output)
        .with_context(|| format!("failed to create `{}`", output.display()))?;
    eprintln!("Created `{}`", output.display());
    wtr.write_all(
        format!(
            "# {} {}\n# {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            std::env::args().collect::<Vec<_>>().join(" ")
        )
        .as_bytes(),
    )
    .context("failed to write csv header")?;

    let mut wtr = csv::Writer::from_writer(wtr);
    for (name, fifo) in cb_fifos {
        let mut previous_marker: Option<WrapAroundMarker> = None;
        for chunk in fifo.split_inclusive(|n| matches!(n, FifoEntry::WrapAroundMarker(_))) {
            let (next_marker, timestamps) = match chunk.split_last() {
                Some((&FifoEntry::WrapAroundMarker(marker), timestamps)) => {
                    (Some(marker), timestamps)
                }
                Some((_, timestamps)) => (None, timestamps),
                _ => unreachable!(),
            };
            for &tsc in timestamps {
                let FifoEntry::TimestampCounter(tsc) = tsc else {
                    unreachable!();
                };
                let row = Row {
                    board: name.clone(),
                    channel: u8::from(tsc.channel),
                    leading_edge: matches!(tsc.edge, EdgeType::Leading),
                    chronobox_time: chronobox_time(tsc, previous_marker, next_marker)
                        .map(|t| t.get::<second>()),
                };

                wtr.serialize(row)
                    .context("failed to write row to csv data")?;
            }
            previous_marker = next_marker;
        }
    }
    wtr.flush().context("failed to flush csv data")?;

    Ok(())
}
