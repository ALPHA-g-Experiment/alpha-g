//! Iterate through a MIDAS file, and visualize the individual PWB waveforms
//! from the cathode pads of the radial Time Projection Chamber.

use crate::filter::{Filter, Overflow};
use crate::next::{worker, Packet, TryNextPacketError};
use crate::plot::{create_plot, empty_plot, JOBNAME};
use clap::Parser;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, LinearLayout, ListView, RadioGroup, TextView};
use cursive::{Cursive, With};
use std::path::PathBuf;
use std::sync::mpsc;
use tempfile::{tempdir, TempDir};

/// Iterate through data packets.
///
/// The application iterates through the input MIDAS files. Each time the "Next"
/// button is pressed, a [`Packet`] is sent (blocking) between a worker and the
/// main thread.
mod next;

/// Accept or reject data packets based on user-defined filters.
///
/// Every time a new [`Packet`] is sent by the `worker` thread, the main
/// application accepts\rejects the package given a set of conditions/filters.
/// A user is only interested in seeing [`Packet`]s that pass the filters.
mod filter;

/// Create and update the waveform plots.
mod plot;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// MIDAS files that you want to inspect
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

/// Structure stored in Cursive object that needs to be accessed while modifying
/// the layout.
struct UserData {
    receiver: mpsc::Receiver<Result<Packet, TryNextPacketError>>,
    dir: TempDir,
    filter: Filter,
}

fn main() {
    let args = Args::parse();
    // Unbuffered channel that blocks until receive.
    let (sender, receiver) = mpsc::sync_channel(0);
    std::thread::spawn(move || worker(sender, &args.files));

    let dir = tempdir().expect("unable to create temporary directory");
    empty_plot(&dir);
    opener::open(dir.path().join(JOBNAME.to_string() + ".pdf"))
        .expect("unable to open temporary plot");

    let mut siv = cursive::default();
    siv.set_window_title("Padwing Waveform Viewer");
    siv.set_autohide_menu(false);
    siv.set_user_data(UserData {
        receiver,
        dir,
        filter: Filter::default(),
    });

    siv.menubar()
        .add_leaf("Filters", select_filters)
        .add_delimiter();

    siv.add_layer(
        Dialog::around(
            TextView::new("Press <Next> to jump to the next Padwing waveform.")
                .with_name("metadata"),
        )
        .title("Packet Metadata")
        .button("Quit", Cursive::quit)
        .button("Next", iterate),
    );

    siv.run();
}

/// Create the radio buttons for a group.
fn make_radio<T: 'static + PartialEq>(
    values: impl IntoIterator<Item = (impl Into<String>, T, usize)>,
    group: &mut RadioGroup<T>,
    current_value: &T,
) -> impl cursive::View {
    LinearLayout::horizontal().with(|layout| {
        for (label, value, width) in values.into_iter() {
            let selected = &value == current_value;
            layout.add_child(
                group
                    .button(value, label)
                    .with_if(selected, |b| {
                        b.select();
                    })
                    .fixed_width(width),
            );
            if selected {
                layout.set_focus_index(layout.len() - 1).unwrap();
            }
        }
    })
}

/// Draw the filter selection pop-up window.
fn select_filters(s: &mut Cursive) {
    s.set_autohide_menu(true);

    let mut overflow: RadioGroup<Option<Overflow>> = RadioGroup::new();

    // Get the current filters to draw the correct status.
    let current_filter = s
        .with_user_data(|user_data: &mut UserData| user_data.filter)
        .unwrap();

    s.add_layer(
        Dialog::new()
            .title("Filters")
            .content(ListView::new().child(
                "Overflow:",
                make_radio(
                    [
                        ("Any", None, 9),
                        ("Positive", Some(Overflow::Positive), 14),
                        ("Negative", Some(Overflow::Negative), 14),
                        ("Both", Some(Overflow::Both), 10),
                        ("Neither", Some(Overflow::Neither), 11),
                    ],
                    &mut overflow,
                    &current_filter.overflow,
                ),
            ))
            .button("Done", move |s| {
                s.with_user_data(|user_data: &mut UserData| {
                    user_data.filter.overflow = *overflow.selection();
                })
                .unwrap();

                s.pop_layer();
                s.set_autohide_menu(false);
            }),
    );
}

/// Iterate through the MIDAS file until a [`Packet`] is found that satisfies
/// the user-defined [`Filter`]. Update the packet metadata and plot
/// appropriately.
fn iterate(s: &mut Cursive) {
    let filter = s
        .with_user_data(|user_data: &mut UserData| user_data.filter)
        .unwrap();
    let result = loop {
        match s.user_data::<UserData>().unwrap().receiver.recv() {
            Ok(result) => match result {
                Ok(ref packet) => {
                    if packet.passes_filter(&filter) {
                        break result;
                    }
                }
                Err(_) => break result,
            },
            Err(_) => panic!("receiver disconnected"),
        }
    };
    let dir = &s.user_data::<UserData>().unwrap().dir;
    match result {
        Ok(packet) => create_plot(dir, &packet),
        Err(_) => empty_plot(dir),
    }
}
