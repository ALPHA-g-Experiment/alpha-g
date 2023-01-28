use alpha_g_detector::midas::{Alpha16BankName, EventId};
use memmap2::Mmap;
use midasio::read::file::{FileView, TryFileViewFromSliceError};
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::SyncSender;
use thiserror::Error;

/// The error type returned when obtaining the next [`Packet`] fails.
#[derive(Error, Debug)]
pub enum TryNextPacketError {
    /// Error opening an input file.
    #[error("failed to open input file")]
    FailedOpen(#[from] std::io::Error),
    /// Input file is not MIDAS file.
    #[error("not a valid MIDAS file")]
    FailedFileView(#[from] TryFileViewFromSliceError),
    /// All input files have been consumed.
    #[error("no more input files")]
    AllConsumed,
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
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
        };
        let file_view = match FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(error) => {
                if sender.send(Err(error.into())).is_err() {
                    return;
                }
                continue;
            }
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

        for bank_view in file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)))
            .flatten()
            .filter(|b| Alpha16BankName::try_from(b.name()).is_ok())
        {
            let packet = Packet {
                adc_packet: bank_view.data_slice().to_owned(),
                bank_name: bank_view.name().to_owned(),
                a16_suppression,
                a32_suppression,
            };
            if sender.send(Ok(packet)).is_err() {
                return;
            }
        }
    }
    let _ = sender.send(Err(TryNextPacketError::AllConsumed));
}
