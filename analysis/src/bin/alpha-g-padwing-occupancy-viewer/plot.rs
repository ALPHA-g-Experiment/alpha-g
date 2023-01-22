use crate::next::Packet;
use alpha_g_detector::padwing::map::TpcPadPosition;
use alpha_g_detector::padwing::ChannelId;
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};

/// Create an empty axis environment.
pub fn empty_picture() -> Picture {
    let axis = Axis::new();
    Picture::from(axis)
}

/// Create a [`Picture`] based on an input [`Packet`].
pub fn create_picture(packet: &Packet) -> Picture {
    let run_number = packet.run_number;

    let mut plot = Plot2D::new();
    plot.add_key(PlotKey::Type2D(Type2D::OnlyMarks));
    for pwb_packet in &packet.pwb_packets {
        let board_id = pwb_packet.board_id();
        let after_id = pwb_packet.after_id();

        for &channel_id in pwb_packet.channels_sent() {
            let waveform = pwb_packet.waveform_at(channel_id).unwrap();
            // If channel was sent, waveform is guaranteed to be non-empty.
            // Only plot if it is also not flat.
            let is_flat = waveform.iter().all(|&v| v == waveform[0]);
            if is_flat {
                continue;
            }

            if let ChannelId::Pad(pad_channel) = channel_id {
                let pad_position =
                    TpcPadPosition::try_new(run_number, board_id, after_id, pad_channel)
                        .expect("pad position mapping failed. Please open an issue on GitHub.");
                plot.coordinates
                    .push((pad_position.phi(), pad_position.z()).into());
            }
        }
    }

    let mut axis = Axis::from(plot);
    axis.set_x_label("$\\phi$~[rad]");
    axis.set_y_label("$z$~[m]");
    axis.set_title("rTPC Pad Occupancy");
    axis.add_key(AxisKey::Custom(String::from("xmin=0.0")));
    axis.add_key(AxisKey::Custom(String::from("xmax=6.3")));
    axis.add_key(AxisKey::Custom(String::from("ymin=-1.15")));
    axis.add_key(AxisKey::Custom(String::from("ymax=1.15")));
    axis.add_key(AxisKey::Custom(String::from("width=240pt, height=463.1pt")));

    Picture::from(axis)
}
