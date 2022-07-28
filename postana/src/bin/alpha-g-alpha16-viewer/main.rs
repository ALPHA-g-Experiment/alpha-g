use crate::filter::{Correctness, Detector, Filter, Overflow};
use crate::next::{worker, Packet, TryNextPacketError};
use alpha_g_detector::alpha16::AdcPacket;
use alpha_g_detector::alpha16::ChannelId::{A16, A32};
use clap::Parser;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, LinearLayout, ListView, RadioGroup, TextView};
use cursive::{Cursive, With};
use pgfplots::axis::{plot::*, *};
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
/// application accepts/rejects the package given a set of conditions/filters.
/// A user is only interested in seeing [`Packet`]s that pass the filters.
mod filter;

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

/// Jobname for pdflatex.
const JOBNAME: &str = "figure";

fn main() {
    let args = Args::parse();
    // Unbuffered channel that blocks until receive.
    let (sender, receiver) = mpsc::sync_channel(0);
    std::thread::spawn(move || worker(sender, &args.files));

    let user_data = UserData {
        receiver,
        dir: tempdir().expect("unable to create temporary directory"),
        filter: Filter::default(),
    };

    let mut siv = cursive::default();
    siv.set_window_title("Alpha16 Packet Viewer");
    siv.set_autohide_menu(false);
    siv.set_user_data(user_data);
    // Just update the plot with anything that would produce an empty plot.
    // update_plot actually just re-creates it. So it gets created here for the
    // first time.
    update_plot(&mut siv, &Err(TryNextPacketError::AllConsumed));
    let dir = &siv.user_data::<UserData>().unwrap().dir;
    opener::open(dir.path().join(JOBNAME.to_string() + ".pdf"))
        .expect("unable to open temporary plot");

    siv.menubar()
        .add_leaf("Filters", draw_filters)
        .add_delimiter();

    siv.add_layer(
        Dialog::around(
            TextView::new("Press <Next> to jump to the next Alpha16 packet.").with_name("metadata"),
        )
        .title("Packet Metadata")
        .button("Quit", |s| s.quit())
        .button("Next", |s| {
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
                    Err(_) => {
                        panic!("receiver disconnected");
                    }
                }
            };
            update_packet_metadata(s, &result);
            update_plot(s, &result);
        }),
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
fn draw_filters(s: &mut Cursive) {
    s.set_autohide_menu(true);

    let mut correctness: RadioGroup<Option<Correctness>> = RadioGroup::new();
    let mut detector: RadioGroup<Option<Detector>> = RadioGroup::new();
    let mut keep_bit: RadioGroup<Option<bool>> = RadioGroup::new();
    let mut overflow: RadioGroup<Option<Overflow>> = RadioGroup::new();

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
                        "Correctness:",
                        make_radio(
                            [
                                ("Any", None, 9),
                                ("Good packet", Some(Correctness::Good), 17),
                                ("Bad packet", Some(Correctness::Bad), 16),
                            ],
                            &mut correctness,
                            &current_filter.correctness,
                        ),
                    )
                    .child(
                        "Detector:",
                        make_radio(
                            [
                                ("Any", None, 9),
                                ("BV", Some(Detector::Bv), 17),
                                ("TPC", Some(Detector::Tpc), 16),
                            ],
                            &mut detector,
                            &current_filter.detector,
                        ),
                    )
                    .child(
                        "Keep bit:",
                        make_radio(
                            [
                                ("Any", None, 9),
                                ("True", Some(true), 17),
                                ("False", Some(false), 16),
                            ],
                            &mut keep_bit,
                            &current_filter.keep_bit,
                        ),
                    )
                    .child(
                        "Overflow:",
                        make_radio(
                            [
                                ("Any", None, 9),
                                ("Positive", Some(Overflow::Positive), 17),
                                ("Negative", Some(Overflow::Negative), 16),
                                ("Both", Some(Overflow::Both), 10),
                                ("Neither", Some(Overflow::Neither), 11),
                            ],
                            &mut overflow,
                            &current_filter.overflow,
                        ),
                    ),
            )
            .button("Done", move |s| {
                s.with_user_data(|user_data: &mut UserData| {
                    user_data.filter.correctness = *correctness.selection();
                    user_data.filter.detector = *detector.selection();
                    user_data.filter.keep_bit = *keep_bit.selection();
                    user_data.filter.overflow = *overflow.selection();
                })
                .unwrap();

                s.pop_layer();
                s.set_autohide_menu(false);
            }),
    );
}

/// Update the Metadata text box with information about the last received packet
fn update_packet_metadata(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) {
    let text = match next_result {
        Ok(packet) => match AdcPacket::try_from(&packet.adc_packet[..]) {
            Ok(packet) => packet.to_string(),
            Err(error) => format!("Error: {error}"),
        },
        Err(error) => {
            let text = format!("Error: {error}");
            s.add_layer(Dialog::info(text));
            String::from("Press <Next> to jump to the next Alpha16 packet.")
        }
    };

    s.call_on_name("metadata", |view: &mut TextView| view.set_content(text));
}

/// Re-create the plot based on an input Packet attempt. The "updating" is
/// actually done by the PDF viewer; we just re-compile the file.
fn update_plot(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) {
    let user_data = s.user_data::<UserData>().unwrap();
    let mut legend = Vec::new();
    let mut axis = Axis::new();
    if let Ok(packet) = next_result {
        if let Ok(adc_packet) = AdcPacket::try_from(&packet.adc_packet[..]) {
            axis.set_title(format!("{} Waveform", packet.bank_name));
            axis.set_x_label(format!(
                "Samples~[{} ns]",
                1e9 / adc_packet.channel_id().sampling_rate()
            ));
            axis.set_y_label("Amplitude~[a.u.]");
            axis.add_key(AxisKey::Custom("ymin=-32768, ymax=32767".to_string()));

            let last_index = if adc_packet.waveform().is_empty() {
                adc_packet.requested_samples()
            } else {
                adc_packet.waveform().len()
            };
            let suppression_threshold = match adc_packet.channel_id() {
                A16(_) => packet.a16_suppression,
                A32(_) => packet.a32_suppression,
            };
            if let Some(threshold) = suppression_threshold {
                if let Some(baseline) = adc_packet.suppression_baseline() {
                    let baseline: f64 = baseline.into();
                    let mut suppression = Plot2D::new();
                    suppression
                        .coordinates
                        .push((0.0, baseline + threshold).into());
                    suppression
                        .coordinates
                        .push((0.0, baseline - threshold).into());
                    suppression
                        .coordinates
                        .push((last_index as f64, baseline - threshold).into());
                    suppression
                        .coordinates
                        .push((last_index as f64, baseline + threshold).into());
                    suppression.add_key(PlotKey::Custom("fill=gray!20, draw=gray!20".to_string()));
                    axis.plots.push(suppression);
                    legend.push(String::from("Data suppression"));
                }
            }

            if !adc_packet.waveform().is_empty() {
                let mut signal = Plot2D::new();
                signal.coordinates = adc_packet
                    .waveform()
                    .iter()
                    .enumerate()
                    .map(|c| (c.0 as f64, f64::from(*c.1)).into())
                    .collect();
                axis.plots.push(signal);
                legend.push(String::from("Waveform"));
            }
        }
    }
    axis.add_key(AxisKey::Custom(format!(
        "legend entries={{{}}}",
        legend.join(",")
    )));
    axis.add_key(AxisKey::Custom("legend style={font=\\tiny}".to_string()));

    let argument = axis.standalone_string().replace('\n', "").replace('\t', "");
    Command::new("pdflatex")
        .current_dir(&user_data.dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-interaction=batchmode")
        .arg("-halt-on-error")
        .arg("-jobname=".to_string() + JOBNAME)
        .arg(argument)
        .status()
        .expect("failed to run pdflatex");
}
