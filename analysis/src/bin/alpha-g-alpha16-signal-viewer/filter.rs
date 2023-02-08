use crate::Packet;
use alpha_g_detector::alpha16::{ADC_MAX, ADC_MIN};
use alpha_g_detector::midas::Alpha16BankName::{self, A16, A32};

/// Source of an ADC data packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detector {
    /// Barrel Veto detector.
    Bv,
    /// Radial Time Projection Chamber.
    Tpc,
}

/// Possible overflow of the waveform in an AdcPacket
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub detector: Option<Detector>,
    pub keep_bit: Option<bool>,
    pub overflow: Option<Overflow>,
}

impl Packet {
    /// Return the [`Detector`] from which the [`Packet`] comes from. This
    /// function only checks the `bank_name` field to determine the source of
    /// the packet.
    // This function never fails because the `worker` function in the `next`
    // module only iterates over valid Alpha16BankNames
    fn detector(&self) -> Detector {
        match Alpha16BankName::try_from(self.bank_name.as_str()).unwrap() {
            A16(_) => Detector::Bv,
            A32(_) => Detector::Tpc,
        }
    }
    /// Return the `keep_bit` of the inner `adc_packet` slice.
    fn keep_bit(&self) -> Option<bool> {
        self.adc_packet.keep_bit()
    }
    /// Return the [`Overflow`] of the inner `adc_packet` slice.
    fn overflow(&self) -> Overflow {
        let waveform = self.adc_packet.waveform();
        let min = waveform.iter().min();
        let max = waveform.iter().max();
        match (min, max) {
            (Some(&ADC_MIN), Some(&ADC_MAX)) => Overflow::Both,
            (Some(&ADC_MIN), _) => Overflow::Negative,
            (_, Some(&ADC_MAX)) => Overflow::Positive,
            _ => Overflow::Neither,
        }
    }
    /// Return [`true`] if the [`Packet`] satisfies a user-defined [`Filter`].
    pub fn passes_filter(&self, filter: &Filter) -> bool {
        if let Some(detector) = filter.detector {
            if self.detector() != detector {
                return false;
            }
        }
        if let Some(keep_bit) = filter.keep_bit {
            if self.keep_bit() != Some(keep_bit) {
                return false;
            }
        }
        if let Some(overflow) = filter.overflow {
            if self.overflow() != overflow {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
