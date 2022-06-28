use clap::Parser;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, LinearLayout, RadioGroup, TextView};
use cursive::Cursive;
use detector::alpha16::{
    AdcPacket,
    ChannelId::{A16, A32},
};
use memmap2::Mmap;
use midasio::read::file::FileView;
use pgfplots::axis::{plot::*, *};
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{fmt, fs::File};
use tempfile::{tempdir, TempDir};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// MIDAS files that you want to inspect
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

/// The error type returned when obtaining the next [`Packet`] failed.
#[derive(Clone, Debug)]
enum TryNextPacketError {
    FailedOpen(PathBuf),
    FailedMmap(PathBuf),
    FailedFileView(PathBuf),
    AllConsumed,
}
impl fmt::Display for TryNextPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Self::FailedOpen(file) => write!(f, "failed to open {}", file.display()),
            Self::FailedMmap(file) => write!(f, "failed to memory map {}", file.display()),
            Self::FailedFileView(file) => write!(
                f,
                "failed to create a MIDAS FileView from {}",
                file.display()
            ),
            Self::AllConsumed => write!(f, "consumed all input files"),
        }
    }
}

/// Data structure that the worker thread is trying to collect from the MIDAS
/// file with every iteration of "next".
#[derive(Clone, Debug)]
struct Packet {
    // ADC packet as a slice of bytes. This allows us to attempt the AdcPacket
    // on the receiver end and display some more helpful information for e.g.
    // debugging bad packets.
    adc_packet: Vec<u8>,
    // Name of the data bank that contains the adc_packet as data_slice
    bank_name: String,
    // These are all Option<T> because maybe the fields are not found in the ODB
    // Suppression threshold of the BV channels
    a16_suppression: Option<f64>,
    // Suppression threshold of the rTPC channels
    a32_suppression: Option<f64>,
    // Trigger threshold of the BV channels
    a16_trigger: Option<f64>,
    // Trigger threshold of the rTPC channels
    a32_trigger: Option<f64>,
}

/// Iterate through all the Alpha16 data banks in the given files.
fn worker(sender: mpsc::SyncSender<Result<Packet, TryNextPacketError>>, file_names: &[PathBuf]) {
    for file_name in file_names {
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(_) => {
                sender
                    .send(Err(TryNextPacketError::FailedOpen(file_name.to_path_buf())))
                    .unwrap();
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(_) => {
                sender
                    .send(Err(TryNextPacketError::FailedMmap(file_name.to_path_buf())))
                    .unwrap();
                continue;
            }
        };
        let file_view = match FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(_) => {
                sender
                    .send(Err(TryNextPacketError::FailedFileView(
                        file_name.to_path_buf(),
                    )))
                    .unwrap();
                continue;
            }
        };
        // Get all the suppression and trigger ODB settings
        let (a16_suppression, a16_trigger, a32_suppression, a32_trigger) = odb_settings(&file_view);

        for event_view in file_view.into_iter().filter(|e| e.id() == 1) {
            for bank_view in event_view
                .into_iter()
                .filter(|b| b.name().starts_with('B') || b.name().starts_with('C'))
            {
                sender
                    .send(Ok(Packet {
                        adc_packet: bank_view.data_slice().to_owned(),
                        bank_name: bank_view.name().to_owned(),
                        a16_suppression,
                        a32_suppression,
                        a16_trigger,
                        a32_trigger,
                    }))
                    .unwrap();
            }
        }
    }
    sender.send(Err(TryNextPacketError::AllConsumed)).unwrap();
}

/// Return the ADC trigger and data suppression thresholds from the ODB.
// (a16_suppression, a16_trigger, a32_suppression, a32_trigger)
fn odb_settings(file_view: &FileView) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let initial_odb = serde_json::from_slice::<Value>(file_view.initial_odb());
    match initial_odb {
        Ok(initial_odb) => (
            initial_odb
                .pointer("/Equipment/CTRL/Settings/ADC/adc16_sthreshold")
                .and_then(|v| v.as_f64()),
            initial_odb
                .pointer("/Equipment/CTRL/Settings/ADC/adc16_threshold")
                .and_then(|v| v.as_f64()),
            initial_odb
                .pointer("/Equipment/CTRL/Settings/ADC/adc32_sthreshold")
                .and_then(|v| v.as_f64()),
            initial_odb
                .pointer("/Equipment/CTRL/Settings/ADC/adc32_threshold")
                .and_then(|v| v.as_f64()),
        ),
        Err(_) => (None, None, None, None),
    }
}

/// Structure stored in Cursive object that needs to be accessed while modifying
/// the layout.
struct UserData {
    receiver: mpsc::Receiver<Result<Packet, TryNextPacketError>>,
    dir: TempDir,
    filter: Filter,
}

/// Conditions that the [`Packet`]s have to satisfy for each "next" call
#[derive(Default, Clone, Copy, Debug)]
struct Filter {
    good_packet: Option<bool>,
    detector: Option<Detector>,
    keep_bit: Option<bool>,
    over_trigger: Option<bool>,
    pos_overflow: Option<bool>,
    neg_overflow: Option<bool>,
}

#[derive(Clone, Copy, Debug)]
enum Detector {
    Bv,
    Tpc,
}

fn main() {
    let args = Args::parse();
    // Unbuffered channel that blocks until receive.
    let (sender, receiver) = mpsc::sync_channel(0);
    std::thread::spawn(move || worker(sender, &args.files));

    let user_data = UserData {
        receiver,
        dir: tempdir().expect("Error: unable to create temporary directory"),
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
        .expect("Error: unable to open temporary plot");

    siv.menubar()
        .add_leaf("Filters", |s| {
            s.set_autohide_menu(true);
            let mut good_group: RadioGroup<Option<bool>> = RadioGroup::new();
            let mut keep_group: RadioGroup<Option<bool>> = RadioGroup::new();
            let mut trigger_group: RadioGroup<Option<bool>> = RadioGroup::new();
            let mut detector_group: RadioGroup<Option<Detector>> = RadioGroup::new();
            let mut pos_overflow_group: RadioGroup<Option<bool>> = RadioGroup::new();
            let mut neg_overflow_group: RadioGroup<Option<bool>> = RadioGroup::new();
            s.add_layer(
                Dialog::new()
                    .title("Filters")
                    .content(
                        LinearLayout::vertical()
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        TextView::new("Good packet: ")
                                            .fixed_width(15)
                                            .fixed_height(2),
                                    )
                                    .child(good_group.button(None, "Any").fixed_width(10))
                                    .child(good_group.button(Some(true), "True").fixed_width(11))
                                    .child(good_group.button(Some(false), "False")),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        TextView::new("Detector: ").fixed_width(15).fixed_height(2),
                                    )
                                    .child(detector_group.button(None, "Any").fixed_width(10))
                                    .child(
                                        detector_group
                                            .button(Some(Detector::Bv), "BV")
                                            .fixed_width(11),
                                    )
                                    .child(detector_group.button(Some(Detector::Tpc), "TPC")),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        TextView::new("Keep bit: ").fixed_width(15).fixed_height(2),
                                    )
                                    .child(keep_group.button(None, "Any").fixed_width(10))
                                    .child(keep_group.button(Some(true), "True").fixed_width(11))
                                    .child(keep_group.button(Some(false), "False")),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        TextView::new("Over trigger: ")
                                            .fixed_width(15)
                                            .fixed_height(2),
                                    )
                                    .child(trigger_group.button(None, "Any").fixed_width(10))
                                    .child(trigger_group.button(Some(true), "True").fixed_width(11))
                                    .child(trigger_group.button(Some(false), "False")),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        TextView::new("Pos. overflow: ")
                                            .fixed_width(15)
                                            .fixed_height(2),
                                    )
                                    .child(pos_overflow_group.button(None, "Any").fixed_width(10))
                                    .child(
                                        pos_overflow_group
                                            .button(Some(true), "True")
                                            .fixed_width(11),
                                    )
                                    .child(pos_overflow_group.button(Some(false), "False")),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(TextView::new("Neg. overflow: ").fixed_width(15))
                                    .child(neg_overflow_group.button(None, "Any").fixed_width(10))
                                    .child(
                                        neg_overflow_group
                                            .button(Some(true), "True")
                                            .fixed_width(11),
                                    )
                                    .child(neg_overflow_group.button(Some(false), "False")),
                            ),
                    )
                    .button("Done", move |s| {
                        s.user_data::<UserData>().unwrap().filter = Filter {
                            good_packet: *good_group.selection(),
                            detector: *detector_group.selection(),
                            keep_bit: *keep_group.selection(),
                            over_trigger: *trigger_group.selection(),
                            pos_overflow: *pos_overflow_group.selection(),
                            neg_overflow: *neg_overflow_group.selection(),
                        };
                        s.pop_layer();
                        s.set_autohide_menu(false);
                    }),
            );
        })
        .add_delimiter();

    siv.add_layer(
        Dialog::around(
            TextView::new("Press <Next> to jump to the next Alpha16 packet.").with_name("metadata"),
        )
        .title("Packet Metadata")
        .button("Quit", |s| s.quit())
        .button("Next", |s| {
            let result = loop {
                match s.user_data::<UserData>().unwrap().receiver.recv() {
                    Ok(result) => {
                        if passes_filters(s, &result) {
                            break result;
                        }
                    }
                    Err(_) => {
                        // s.quit() does not work. I DON'T understand why.
                        // I can only quit the application inside this loop
                        // with a panic!()
                        panic!("Error: receiver disconnected");
                    }
                }
            };
            update_packet_metadata(s, &result);
            update_plot(s, &result);
        }),
    );

    siv.run();
}

fn passes_filters(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) -> bool {
    let user_data = s.user_data::<UserData>().unwrap();
    let filter = user_data.filter;
    match next_result {
        Err(_) => true,
        Ok(packet) => {
            let adc_result = AdcPacket::try_from(&packet.adc_packet[..]);
            if let Some(good_filter) = filter.good_packet {
                if good_filter && adc_result.is_err() {
                    return false;
                }
                if !good_filter && adc_result.is_ok() {
                    return false;
                }
            }
            if let Some(detector_filter) = filter.detector {
                match detector_filter {
                    Detector::Bv => {
                        if !packet.bank_name.starts_with('B') {
                            return false;
                        }
                    }
                    Detector::Tpc => {
                        if !packet.bank_name.starts_with('C') {
                            return false;
                        }
                    }
                }
            }
            if let Some(keep_filter) = filter.keep_bit {
                match adc_result {
                    Err(_) => {
                        return false;
                    }
                    Ok(ref adc_packet) => match adc_packet.keep_bit() {
                        None => {
                            return false;
                        }
                        Some(keep_bit) => {
                            if keep_filter && !keep_bit {
                                return false;
                            }
                            if !keep_filter && keep_bit {
                                return false;
                            }
                        }
                    },
                }
            }
            if let Some(trigger_filter) = filter.over_trigger {
                match adc_result {
                    Err(_) => {
                        return false;
                    }
                    Ok(ref adc_packet) => {
                        let trigger;
                        let max_sample;
                        if packet.bank_name.starts_with('B') {
                            trigger = packet.a16_trigger;
                            max_sample = adc_packet.waveform().iter().max();
                        } else if packet.bank_name.starts_with('C') {
                            trigger = packet.a32_trigger;
                            max_sample = adc_packet.waveform().iter().min();
                        } else {
                            trigger = None;
                            max_sample = None;
                        }
                        match (trigger, max_sample) {
                            (Some(threshold), Some(value)) => {
                                let triggers;
                                if packet.bank_name.starts_with('B') {
                                    triggers = f64::from(*value) >= threshold;
                                } else if packet.bank_name.starts_with('C') {
                                    triggers = f64::from(*value) <= threshold;
                                } else {
                                    triggers = false;
                                }
                                if trigger_filter && !triggers {
                                    return false;
                                }
                                if !trigger_filter && triggers {
                                    return false;
                                }
                            }
                            _ => {
                                return false;
                            }
                        }
                    }
                }
            }
            if let Some(overflow) = filter.pos_overflow {
                match adc_result {
                    Err(_) => {
                        return false;
                    }
                    Ok(ref adc_packet) => {
                        let max = adc_packet.waveform().iter().max();
                        if overflow && (max != Some(&32764)) {
                            return false;
                        }
                        if !overflow && (max == Some(&32764)) {
                            return false;
                        }
                    }
                }
            }
            if let Some(overflow) = filter.neg_overflow {
                match adc_result {
                    Err(_) => {
                        return false;
                    }
                    Ok(adc_packet) => {
                        let min = adc_packet.waveform().iter().min();
                        if overflow && (min != Some(&i16::MIN)) {
                            return false;
                        }
                        if !overflow && (min == Some(&i16::MIN)) {
                            return false;
                        }
                    }
                }
            }
            true
        }
    }
}
/// Update the Metadata text box with information about the last received packet
fn update_packet_metadata(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) {
    let text = match next_result {
        Err(error) => {
            let text = format!("Error: {error}");
            s.add_layer(Dialog::info(text));
            String::from("Press <Next> to jump to the next Alpha16 packet.")
        }
        Ok(packet) => metadata(packet),
    };

    s.call_on_name("metadata", |view: &mut TextView| view.set_content(text));
}

/// Obtain the metadata text from a given packet
fn metadata(packet: &Packet) -> String {
    let packet_info = match AdcPacket::try_from(&packet.adc_packet[..]) {
        Err(error) => format!("Error: {error}"),
        Ok(packet) => format!(
            "Packet type: {}
Packet version: {}
Accepted trigger: {}
Module ID: {:?}
Channel ID: {:?}
Requested samples: {}
Event timestamp: {}
MAC address: {}
Trigger offset: {}
Build timestamp: {}
Waveform samples: {}
Suppression baseline: {}
Keep last: {}
Keep bit: {}
Suppression enabled: {}",
            packet.packet_type(),
            packet.packet_version(),
            packet.accepted_trigger(),
            packet.module_id(),
            packet.channel_id(),
            packet.requested_samples(),
            packet.event_timestamp(),
            packet
                .board_id()
                .map_or("None".to_string(), |b| format!("{:?}", b.mac_address())),
            packet
                .trigger_offset()
                .map_or("None".to_string(), |v| v.to_string()),
            packet
                .build_timestamp()
                .map_or("None".to_string(), |v| v.to_string()),
            packet.waveform().len(),
            packet
                .suppression_baseline()
                .map_or("Not applicable".to_string(), |v| v.to_string()),
            packet
                .keep_last()
                .map_or("Not applicable".to_string(), |v| v.to_string()),
            packet
                .keep_bit()
                .map_or("Not applicable".to_string(), |v| v.to_string()),
            packet
                .is_suppression_enabled()
                .map_or("Not applicable".to_string(), |v| v.to_string()),
        ),
    };
    packet_info
}

const JOBNAME: &str = "figure";

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
            let (suppression_threshold, trigger_threshold) = match adc_packet.channel_id() {
                A16(_) => (packet.a16_suppression, packet.a16_trigger),
                A32(_) => (packet.a32_suppression, packet.a32_trigger),
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
            if let Some(threshold) = trigger_threshold {
                let mut trigger = Plot2D::new();
                trigger.coordinates.push((0.0, threshold).into());
                trigger
                    .coordinates
                    .push((last_index as f64, threshold).into());
                trigger.add_key(PlotKey::Custom("dashed".to_string()));
                axis.plots.push(trigger);
                legend.push(String::from("Trigger threshold"));
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
        .expect("Error: failed to run pdflatex");
}
