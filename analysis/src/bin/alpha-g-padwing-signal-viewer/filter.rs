use crate::Packet;
use alpha_g_detector::padwing::{PWB_MAX, PWB_MIN};

// Only imported for documentation. If you notice that this is no longer the
// case, please change it.
#[allow(unused_imports)]
use alpha_g_detector::padwing::PwbPacket;

/// Possible overflow of the waveform in a [`PwbPacket`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Exclusively positive.
    Positive,
    /// Exclusively negative.
    Negative,
    /// Both positive and negative overflows.
    Both,
    /// Neither positive nor negative overflow.
    Neither,
}

/// User-selected conditions that a [`Packet`] has to satisfy to be displayed.
#[derive(Default, Clone, Copy, Debug)]
pub struct Filter {
    pub overflow: Option<Overflow>,
}

impl Packet {
    /// Return the [`Overflow`] of the inner `pwb_packet` at the current
    /// `channel_id`.
    fn overflow(&self) -> Overflow {
        // Channel IS SENT, then guaranteed to have non-empty waveform. All
        // these unwraps shouldn't panic.
        let waveform = self.pwb_packet.waveform_at(self.channel_id).unwrap();
        let min = waveform.iter().min().unwrap();
        let max = waveform.iter().max().unwrap();
        match (*min, *max) {
            (PWB_MIN, PWB_MAX) => Overflow::Both,
            (PWB_MIN, _) => Overflow::Negative,
            (_, PWB_MAX) => Overflow::Positive,
            _ => Overflow::Neither,
        }
    }
    /// Return [`true`] if the [`Packet`] satisfies a user-defined [`Filter`].
    pub fn passes_filter(&self, filter: &Filter) -> bool {
        if let Some(overflow) = filter.overflow {
            if overflow != self.overflow() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
