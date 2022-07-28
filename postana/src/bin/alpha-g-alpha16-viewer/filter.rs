use crate::Packet;
use alpha_g_detector::alpha16::AdcPacket;
use alpha_g_detector::midas::Alpha16BankName::{self, A16, A32};
use std::mem::discriminant;

/// Correctness of an ADC data packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Correctness {
    /// A good ADC data packet converts without error into an [`AdcPacket`].
    Good,
    /// A bad ADC data packet fails during conversion with
    /// [`TryAdcPacketFromSliceError`]
    Bad,
}

/// Source of an ADC data packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Detector {
    /// Barrel Veto detector.
    Bv,
    /// Radial Time Projection Chamber.
    Tpc,
}

/// Possible overflow of the waveform in an [`AdcPacket`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Overflow {
    /// Exclusively positive.
    Positive,
    /// Exclusively negative.
    Negative,
    /// Both negative and positive overflows.
    Both,
    /// Neither negative nor positive overflow.
    Neither,
}

/// User-selected conditions that a [`Packet`] has to satisfy to be displayed.
#[derive(Default, Clone, Copy, Debug)]
pub struct Filter {
    pub correctness: Option<Correctness>,
    pub detector: Option<Detector>,
    pub keep_bit: Option<bool>,
    pub overflow: Option<Overflow>,
}

impl Packet {
    /// Return the [`Correctness`] of the inner `adc_packet` slice.
    fn correctness(&self) -> Correctness {
        match AdcPacket::try_from(&self.adc_packet[..]) {
            Ok(_) => Correctness::Good,
            Err(_) => Correctness::Bad,
        }
    }
    /// Return the [`Detector`] from which the [`Packet`] comes from. This
    /// function only checks the `bank_name` field to determine the source of
    /// the packet.
    // I don't check for the `adc_packet` itself because it can be a bad packet,
    // hence the name is the only reliable information on the source of the
    // packet.
    // This function never fails because the `worker` function in the `next`
    // module only iterates over valid Alpha16BankNames
    fn detector(&self) -> Detector {
        match Alpha16BankName::try_from(self.bank_name.as_str()).unwrap() {
            A16(_) => Detector::Bv,
            A32(_) => Detector::Tpc,
        }
    }
    /// Return the `keep_bit` of the inner `adc_packet` slice. Additionally
    /// return [`None`] if the conversion fails.
    fn keep_bit(&self) -> Option<bool> {
        match AdcPacket::try_from(&self.adc_packet[..]) {
            Ok(packet) => packet.keep_bit(),
            Err(_) => None,
        }
    }
    /// Return the [`Overflow`] of the inner `adc_packet` slice. Additionally
    /// return [`None`] if the conversion fails or the waveform is empty.
    fn overflow(&self) -> Option<Overflow> {
        match AdcPacket::try_from(&self.adc_packet[..]) {
            Ok(packet) => {
                let min = packet.waveform().iter().min();
                let max = packet.waveform().iter().max();
                match (min, max) {
                    (Some(&i16::MIN), Some(&32764)) => Some(Overflow::Both),
                    (Some(&i16::MIN), Some(_)) => Some(Overflow::Negative),
                    (Some(_), Some(&32764)) => Some(Overflow::Positive),
                    (Some(_), Some(_)) => Some(Overflow::Neither),
                    _ => None,
                }
            }
            Err(_) => None,
        }
    }
    /// Return [`true`] if the [`Packet`] satisfies a user-defined [`Filter`].
    pub fn passes_filter(&self, filter: &Filter) -> bool {
        if let Some(correctness) = filter.correctness {
            if discriminant(&self.correctness()) != discriminant(&correctness) {
                return false;
            }
        }
        if let Some(detector) = filter.detector {
            if discriminant(&self.detector()) != discriminant(&detector) {
                return false;
            }
        }
        if let Some(keep_bit) = filter.keep_bit {
            match self.keep_bit() {
                None => return false,
                Some(value) => {
                    if value != keep_bit {
                        return false;
                    }
                }
            }
        }
        if let Some(overflow) = filter.overflow {
            match self.overflow() {
                None => return false,
                Some(value) => {
                    if discriminant(&overflow) != discriminant(&value) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
