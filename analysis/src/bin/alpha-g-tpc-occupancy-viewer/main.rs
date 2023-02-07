//! Iterate through a MIDAS file and visualize the anode wire and cathode pad
//! occupancy on each event.

use crate::filter::Filter;
use crate::next::{worker, Packet};
use crate::plot::{create_picture, empty_picture};
use anyhow::{Context, Result};
use clap::Parser;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, ListView, TextView};
use cursive::Cursive;
use pgfplots::Engine;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use tempfile::{tempdir, TempDir};

/// Iterate through MIDAS events.
///
/// The application iterates through the input MIDAS files. Each time the "Next"
/// button is pressed, a [`Packet`] is sent (blocking) between a worker and the
/// main thread.
mod next;

/// Accept or reject an event based on user-defined filters.
///
/// Every time a new [`Packet`] is sent by the `worker` thread, the main
/// application accepts/rejects the package given a set of conditions/filters.
/// A user is only interested in visualizing events that satisfy these
/// conditions.
mod filter;

/// Create the plots based on the event data.
///
/// The main application creates a plot for each event. The plots are
/// automatically updated when a new event is accepted.
mod plot;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Visualize the wire and pad occupancy of the rTPC", long_about = None)]
struct Args {
    /// MIDAS files that you want to inspect
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

/// Structure stored in Cursive object that needs to be accessed while modifying
/// the layout.
struct UserData {
    receiver: mpsc::Receiver<Result<Packet>>,
    jobname: String,
    dir: TempDir,
    filter: Filter,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // Unbuffered channel that blocks until receive.
    let (sender, receiver) = mpsc::sync_channel(0);
    std::thread::spawn(move || worker(sender, &args.files));

    let dir = tempdir().context("failed to create temporary directory")?;
    let jobname = String::from("tpc_occupancy_viewer");
    let pdf_path = empty_picture()
        .to_pdf(&dir, &jobname, Engine::PdfLatex)
        .context("failed to compile empty PDF")?;
    opener::open(&pdf_path).with_context(|| format!("failed to open `{}`", pdf_path.display()))?;

    let mut siv = cursive::default();
    siv.set_window_title("rTPC Occupancy Viewer");
    siv.set_autohide_menu(false);
    siv.set_user_data(UserData {
        receiver,
        jobname,
        dir,
        filter: Filter::default(),
    });

    siv.menubar()
        .add_leaf("Filters", select_filters)
        .add_delimiter();

    siv.add_layer(
        Dialog::around(
            TextView::new("Press <Next> to jump to the next event.").with_name("metadata"),
        )
        .title("Event Metadata")
        .button("Quit", Cursive::quit)
        .button("Next", iterate),
    );

    siv.run();

    Ok(())
}

/// Create the EditView for the user to enter the filter values.
// Just keeps the `select_filters` function more readable.
fn make_edit_view<S: Into<String>>(
    name: S,
    current_filter: Option<usize>,
    size: usize,
) -> impl cursive::View {
    EditView::new()
        .content(current_filter.map_or(String::new(), |v| v.to_string()))
        .with_name(name)
        .fixed_width(size)
}

/// Get the filter values from the EditView by name.
// Same as above. Just keeps the `select_filters` function more readable.
fn try_value_from_edit_view(siv: &mut Cursive, name: &str) -> Result<Option<usize>> {
    siv.call_on_name(name, |view: &mut EditView| {
        let text = view.get_content();
        if text.is_empty() {
            Ok(None)
        } else {
            text.parse::<usize>()
                .map(Some)
                .with_context(|| format!("failed to parse `{text}` as a valid filter value"))
        }
    })
    .unwrap()
}

/// Draw the filter selection pop-up window.
fn select_filters(s: &mut Cursive) {
    s.set_autohide_menu(true);

    // Get the current filters to draw the correct status.
    let current_filter = s
        .with_user_data(|user_data: &mut UserData| user_data.filter)
        .unwrap();

    s.add_layer(
        Dialog::new()
            .title("Filters")
            .content(
                ListView::new()
                    .child(
                        "Min number of anode wires: ",
                        make_edit_view("min_anode_wires", current_filter.min_anode_wires, 10),
                    )
                    .child(
                        "Max number of anode wires: ",
                        make_edit_view("max_anode_wires", current_filter.max_anode_wires, 10),
                    )
                    .child(
                        "Min number of pads: ",
                        make_edit_view("min_pads", current_filter.min_pads, 10),
                    )
                    .child(
                        "Max number of pads: ",
                        make_edit_view("max_pads", current_filter.max_pads, 10),
                    ),
            )
            .button("Done", move |s| {
                let min_anode_wires = try_value_from_edit_view(s, "min_anode_wires");
                let max_anode_wires = try_value_from_edit_view(s, "max_anode_wires");
                let min_pads = try_value_from_edit_view(s, "min_pads");
                let max_pads = try_value_from_edit_view(s, "max_pads");

                match (min_anode_wires, max_anode_wires, min_pads, max_pads) {
                    (Ok(min_anode_wires), Ok(max_anode_wires), Ok(min_pads), Ok(max_pads)) => {
                        s.with_user_data(|user_data: &mut UserData| {
                            user_data.filter = Filter {
                                min_anode_wires,
                                max_anode_wires,
                                min_pads,
                                max_pads,
                            };
                        })
                        .unwrap();
                        s.pop_layer();
                        s.set_autohide_menu(false);
                    }
                    (Err(error), _, _, _)
                    | (_, Err(error), _, _)
                    | (_, _, Err(error), _)
                    | (_, _, _, Err(error)) => {
                        s.add_layer(Dialog::info(format!("Error: {error:?}")));
                    }
                }
            }),
    );
}

/// Iterate through the MIDAS file until a [`Packet`] is found that satisfies
/// the user-defined [`Filter`]. Update the packet metadata and the plots.
fn iterate(s: &mut Cursive) {
    let filter = s
        .with_user_data(|user_data: &mut UserData| user_data.filter)
        .unwrap();
    let result = loop {
        match s.user_data::<UserData>().unwrap().receiver.recv() {
            Ok(result) => match result {
                Ok(ref packet) => {
                    if packet.passes_filter(filter) {
                        break result;
                    }
                }
                Err(_) => break result,
            },
            Err(_) => {
                s.quit();
                return;
            }
        }
    };
    update_event_metadata(s, &result);
    update_plot(s, &result);
}

/// Update the event metadata box with the last received packet.
fn update_event_metadata(s: &mut Cursive, result: &Result<Packet>) {
    let text = match result {
        Ok(packet) => {
            let mut text = format!("Serial number: {}\n", packet.serial_number);
            writeln!(text, "Number of anode wires: {}", packet.num_anode_wires()).unwrap();
            write!(text, "Number of pads: {}", packet.num_pads()).unwrap();
            text
        }
        Err(error) => {
            let text = format!("Error: {error:?}");
            s.add_layer(Dialog::info(text));

            String::from("Press <Next> to jump to the next event.")
        }
    };

    s.call_on_name("metadata", |view: &mut TextView| {
        view.set_content(text);
    });
}

/// Update the plot with the last received packet.
fn update_plot(s: &mut Cursive, result: &Result<Packet>) {
    let jobname = s
        .with_user_data(|user_data: &mut UserData| user_data.jobname.clone())
        .unwrap();
    let dir = &s.user_data::<UserData>().unwrap().dir;
    match result {
        Ok(packet) => match create_picture(packet) {
            Ok(picture) => {
                if picture.to_pdf(dir, &jobname, Engine::PdfLatex).is_err() {
                    empty_picture()
                        .to_pdf(dir, &jobname, Engine::PdfLatex)
                        .expect("failed to compile empty picture");
                    s.add_layer(Dialog::info("Too many points. PDF compilation failed"));
                }
            }
            Err(error) => {
                empty_picture()
                    .to_pdf(dir, &jobname, Engine::PdfLatex)
                    .expect("failed to compile empty picture");

                let text = format!("Error: {error:?}");
                s.add_layer(Dialog::info(text));
            }
        },
        Err(_) => {
            empty_picture()
                .to_pdf(dir, &jobname, Engine::PdfLatex)
                .expect("failed to compile empty picture");
        }
    };
}
