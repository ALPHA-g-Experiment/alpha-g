use alpha_g_detector::alpha16::AdcPacket;
use alpha_g_detector::midas::{
    Alpha16BankName, EventId, ADC16_SUPPRESSION_THRESHOLD_JSON_PTR,
    ADC32_SUPPRESSION_THRESHOLD_JSON_PTR,
};
use anyhow::{anyhow, Context, Result};
use memmap2::Mmap;
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::SyncSender;

/// Data that the worker thread is trying to collect from the MIDAS file with
/// every iteration of "next".
// All these have to be owned to avoid lifetime restrictions of Cursive's
// user_data.
#[derive(Clone, Debug)]
pub struct Packet {
    /// ADC packet.
    pub adc_packet: AdcPacket,
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
pub fn worker<P>(sender: SyncSender<Result<Packet>>, file_names: impl IntoIterator<Item = P>)
where
    P: AsRef<Path> + std::marker::Copy,
{
    for file_name in file_names {
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!("failed to open `{}`", file_name.as_ref().display())
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!("failed to memory map `{}`", file_name.as_ref().display())
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let file_view = match midasio::FileView::try_from(&mmap[..]) {
            Ok(file_view) => file_view,
            Err(error) => {
                if sender
                    .send(Err(error).with_context(|| {
                        format!(
                            "`{}` is not a valid MIDAS file",
                            file_name.as_ref().display()
                        )
                    }))
                    .is_err()
                {
                    return;
                }
                continue;
            }
        };
        let odb = serde_json::from_slice::<Value>(file_view.initial_odb());
        let (a16_suppression, a32_suppression) = if let Ok(odb) = odb {
            (
                odb.pointer(ADC16_SUPPRESSION_THRESHOLD_JSON_PTR)
                    .and_then(|v| v.as_f64()),
                odb.pointer(ADC32_SUPPRESSION_THRESHOLD_JSON_PTR)
                    .and_then(|v| v.as_f64()),
            )
        } else {
            (None, None)
        };

        for event_view in file_view
            .into_iter()
            .filter(|e| matches!(EventId::try_from(e.id()), Ok(EventId::Main)))
        {
            for bank_view in (&event_view)
                .into_iter()
                .filter(|b| Alpha16BankName::try_from(b.name()).is_ok())
            {
                let adc_packet = match AdcPacket::try_from(bank_view.data_slice()) {
                    Ok(adc_packet) => adc_packet,
                    Err(error) => {
                        if sender
                            .send(Err(error).with_context(|| {
                                format!(
                                    "bad alpha16 data bank in event `{}`",
                                    event_view.serial_number()
                                )
                            }))
                            .is_err()
                        {
                            return;
                        }
                        continue;
                    }
                };
                let packet = Packet {
                    adc_packet,
                    bank_name: bank_view.name().to_owned(),
                    a16_suppression,
                    a32_suppression,
                };
                if sender.send(Ok(packet)).is_err() {
                    return;
                }
            }
        }
    }
    let _ = sender.send(Err(anyhow!("no more files")));
}
