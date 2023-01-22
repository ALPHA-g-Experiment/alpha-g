use crate::next::Packet;

/// User-selected conditions that a [`Packet`] has to satisfy in order to be
/// displayed.
#[derive(Default, Clone, Copy, Debug)]
pub struct Filter {
    /// Minimum number of pads in the event.
    pub min_pads: Option<usize>,
    /// Maximum number of pads in the event.
    pub max_pads: Option<usize>,
}

impl Packet {
    /// Returns the number of pad waveforms in the event.
    // Count only non-empty and non-flat waveforms.
    pub fn num_pads(&self) -> usize {
        let mut num_pads = 0;
        for pwb_packet in self.pwb_packets.iter() {
            for &channel_id in pwb_packet.channels_sent() {
                let waveform = pwb_packet.waveform_at(channel_id).unwrap();
                // If channel was sent, waveform is guaranteed to be non-empty.
                let is_flat = waveform.iter().all(|&v| v == waveform[0]);
                if !is_flat {
                    num_pads += 1;
                }
            }
        }
        num_pads
    }
    /// Returns `true` if the packet satisfies the filter.
    pub fn passes_filter(&self, filter: Filter) -> bool {
        let num_pads = self.num_pads();
        if let Some(min_pads) = filter.min_pads {
            if num_pads < min_pads {
                return false;
            }
        }
        if let Some(max_pads) = filter.max_pads {
            if num_pads > max_pads {
                return false;
            }
        }
        true
    }
}
