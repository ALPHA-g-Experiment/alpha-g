use crate::Packet;
use alpha_g_detector::padwing::ChannelId::{Fpn, Pad, Reset};
use alpha_g_detector::padwing::{suppression_baseline, PWB_MAX, PWB_MIN, PWB_RATE};
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};

/// Create an empty axis environment.
pub fn empty_picture() -> Picture {
    let axis = Axis::new();
    Picture::from(axis)
}

/// Create a [`Picture`] based on an input Packet.
pub fn create_picture(packet: &Packet) -> Picture {
    let mut legend = Vec::new();
    let mut axis = Axis::new();
    axis.set_title(format!(
        "Board {}. AFTER {:?}. {}",
        packet.pwb_packet.board_id().name(),
        packet.pwb_packet.after_id(),
        match packet.channel_id {
            Pad(channel) => format!("{:?}", channel),
            Fpn(channel) => format!("{:?}", channel),
            Reset(channel) => format!("{:?}", channel),
        }
    ));
    axis.set_x_label(format!("Samples~[{} ns]", 1e9 / PWB_RATE));
    axis.set_y_label("Amplitude~[a.u.]");
    axis.add_key(AxisKey::Custom(format!("ymin={PWB_MIN}, ymax={PWB_MAX}")));

    if let Some(baseline) = suppression_baseline(
        packet.run_number,
        packet.pwb_packet.waveform_at(packet.channel_id).unwrap(),
    )
    .unwrap()
    {
        if let Some(threshold) = packet.suppression_threshold {
            let mut suppression = Plot2D::new();
            for (mut x, mut y) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                x *= packet.pwb_packet.requested_samples() as f64;
                y = y * 2.0 * threshold + f64::from(baseline) - threshold;
                suppression.coordinates.push((x, y).into());
            }
            suppression.add_key(PlotKey::Custom(
                "fill=gray!20, draw=gray!20, area legend".to_string(),
            ));
            axis.plots.push(suppression);
            legend.push(String::from("Data suppression"));
        }
    }

    let mut signal = Plot2D::new();
    signal.coordinates = packet
        .pwb_packet
        .waveform_at(packet.channel_id)
        .unwrap()
        .iter()
        .enumerate()
        .map(|c| (c.0 as f64, f64::from(*c.1)).into())
        .collect();
    axis.plots.push(signal);
    legend.push(String::from("Waveform"));

    axis.add_key(AxisKey::Custom(format!(
        "legend entries={{{}}}",
        legend.join(",")
    )));
    axis.add_key(AxisKey::Custom("legend pos=south east".to_string()));
    axis.add_key(AxisKey::Custom("legend style={font=\\tiny}".to_string()));

    Picture::from(axis)
}
