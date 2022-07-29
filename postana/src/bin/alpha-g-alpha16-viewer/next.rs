use alpha_g_detector::midas::{Alpha16BankName, EventId};
use memmap2::Mmap;
use midasio::read::file::FileView;
use serde_json::Value;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::SyncSender;

/// The error type returned when obtaining the next [`Packet`] fails.
#[derive(Clone, Debug)]
pub enum TryNextPacketError {
    /// Error opening an input file.
    FailedOpen(PathBuf),
    /// Error in the underlying system call to memory map the file.
    FailedMmap(PathBuf),
    /// Input file is not MIDAS file.
    FailedFileView(PathBuf),
    /// All input files have been consumed.
    AllConsumed,
}
impl fmt::Display for TryNextPacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

/// Data that the worker thread is trying to collect from the MIDAS file with
/// every iteration of "next".
// All these have to be owned to avoid lifetime restrictions of Cursive's
// user_data.
#[derive(Clone, Debug)]
pub struct Packet {
    /// ADC packet as a slice of bytes.
    // This allows us to attempt the AdcPacket on the receiver end and react
    // appropriately if the AdcPacket fails.
    pub adc_packet: Vec<u8>,
    /// Name of the data bank that contains the `adc_packet`.
    pub bank_name: String,
    // These are all Option<T> because maybe the fields are not found in the ODB
    /// Data suppression threshold of the BV channels
    pub a16_suppression: Option<f64>,
    /// Data suppression threshold of the rTPC channels
    pub a32_suppression: Option<f64>,
}

/// Worker function that iterates through MIDAS files and tries to send
/// [`Packet`]s to the main thread.
pub fn worker<P>(
    sender: SyncSender<Result<Packet, TryNextPacketError>>,
    file_names: impl IntoIterator<Item = P>,
) where
    P: AsRef<Path> + std::marker::Copy,
{
    for file_name in file_names {
        let file = if let Ok(file) = File::open(file_name) {
            file
        } else {
            sender
                .send(Err(TryNextPacketError::FailedOpen(
                    file_name.as_ref().to_path_buf(),
                )))
                .expect("receiver disconnected on \"failed to open\"");
            continue;
        };
        let mmap = if let Ok(mmap) = unsafe { Mmap::map(&file) } {
            mmap
        } else {
            sender
                .send(Err(TryNextPacketError::FailedMmap(
                    file_name.as_ref().to_path_buf(),
                )))
                .expect("receiver disconnected on \"failed to mmap\"");
            continue;
        };
        let file_view = if let Ok(file_view) = FileView::try_from(&mmap[..]) {
            file_view
        } else {
            sender
                .send(Err(TryNextPacketError::FailedFileView(
                    file_name.as_ref().to_path_buf(),
                )))
                .expect("receiver disconnected on \"failed to FileView\"");
            continue;
        };
        let odb = serde_json::from_slice::<Value>(file_view.initial_odb());
        let (a16_suppression, a32_suppression) = if let Ok(odb) = odb {
            (
                odb.pointer("/Equipment/CTRL/Settings/ADC/adc16_sthreshold")
                    .and_then(|v| v.as_f64()),
                odb.pointer("/Equipment/CTRL/Settings/ADC/adc32_sthreshold")
                    .and_then(|v| v.as_f64()),
            )
        } else {
            (None, None)
        };

        let main_events = file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)));
        for event_view in main_events {
            let alpha16_banks = event_view
                .into_iter()
                .filter(|b| Alpha16BankName::try_from(b.name()).is_ok());
            for bank_view in alpha16_banks {
                sender
                    .send(Ok(Packet {
                        adc_packet: bank_view.data_slice().to_owned(),
                        bank_name: bank_view.name().to_owned(),
                        a16_suppression,
                        a32_suppression,
                    }))
                    .expect("receiver disconnected on \"data packet\"");
            }
        }
    }
    sender
        .send(Err(TryNextPacketError::AllConsumed))
        .expect("receiver disconnected on \"all consumed\"");
}
