use clap::Parser;
use cursive::view::Nameable;
use cursive::views::{Dialog, TextView};
use cursive::Cursive;
use detector::alpha16::AdcPacket;
use memmap2::Mmap;
use midasio::read::file::FileView;
use pgfplots::axis::plot::Plot2D;
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

        for event_view in file_view.into_iter().filter(|e| e.id() == 1) {
            for bank_view in event_view
                .into_iter()
                .filter(|b| b.name().starts_with('B') || b.name().starts_with('C'))
            {
                let data = bank_view.data_slice();
                sender
                    .send(Ok(Packet {
                        adc_packet: data.to_owned(),
                    }))
                    .unwrap();
            }
        }
    }
    sender.send(Err(TryNextPacketError::AllConsumed)).unwrap();
}

struct UserData {
    receiver: mpsc::Receiver<Result<Packet, TryNextPacketError>>,
    dir: TempDir,
}

fn main() {
    let args = Args::parse();
    // Unbuffered channel that blocks until receive.
    let (sender, receiver) = mpsc::sync_channel(0);
    std::thread::spawn(move || worker(sender, &args.files));

    let user_data = UserData {
        receiver,
        dir: tempdir().expect("Error: unable to create temporary directory"),
    };

    let mut siv = cursive::default();
    siv.set_user_data(user_data);

    update_plot(&mut siv, &Err(TryNextPacketError::AllConsumed));
    let dir = &siv.user_data::<UserData>().unwrap().dir;
    opener::open(dir.path().join(JOBNAME.to_string() + ".pdf"))
        .expect("Error: unable to open temporary plot");

    siv.add_layer(
        Dialog::around(
            TextView::new("Press <Next> to jump to the next Alpha16 packet.").with_name("metadata"),
        )
        .title("Packet Metadata")
        .button("Quit", |s| s.quit())
        .button("Next", |s| {
            let user_data = s.user_data::<UserData>().unwrap();

            match user_data.receiver.recv() {
                Ok(result) => {
                    update_packet_metadata(s, &result);
                    update_plot(s, &result);
                }
                Err(_) => {
                    s.quit();
                }
            }
        }),
    );

    siv.run();
}

fn update_packet_metadata(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) {
    let text = match next_result {
        Err(error) => {
            let text = format!("Error: {error}");
            s.add_layer(cursive::views::Dialog::info(text));
            String::from("Press <Next> to jump to the next Alpha16 packet.")
        }
        Ok(packet) => metadata(packet),
    };

    s.call_on_name("metadata", |view: &mut TextView| view.set_content(text));
}

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

fn update_plot(s: &mut Cursive, next_result: &Result<Packet, TryNextPacketError>) {
    let user_data = s.user_data::<UserData>().unwrap();
    let mut plot = Plot2D::new();
    if let Ok(packet) = next_result {
        if let Ok(packet) = AdcPacket::try_from(&packet.adc_packet[..]) {
            plot.coordinates = packet
                .waveform()
                .iter()
                .enumerate()
                .map(|c| (c.0 as f64, f64::from(*c.1)).into())
                .collect();
        }
    }
    let argument = plot.standalone_string().replace('\n', "").replace('\t', "");
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
