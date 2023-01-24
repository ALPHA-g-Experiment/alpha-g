use crate::next::Packet;
use alpha_g_detector::padwing::map::*;
use alpha_g_detector::padwing::ChannelId;
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};
use std::f64::consts::PI;

/// Create an empty axis environment.
pub fn empty_picture() -> Picture {
    let axis = Axis::new();
    Picture::from(axis)
}

/// Create a [`Picture`] based on an input [`Packet`].
pub fn create_picture(packet: &Packet) -> Picture {
    let run_number = packet.run_number;

    let mut axis = Axis::new();
    axis.set_x_label("$\\phi$~[rad]");
    axis.set_y_label("$z$~[m]");
    axis.set_title("rTPC Pad Occupancy");
    axis.add_key(AxisKey::Custom(format!("xmin={}", -PAD_PITCH_PHI / 2.0)));
    axis.add_key(AxisKey::Custom(format!(
        "xmax={}",
        2.0 * PI - PAD_PITCH_PHI / 2.0
    )));
    axis.add_key(AxisKey::Custom(format!("ymin={}", -DETECTOR_LENGTH / 2.0)));
    axis.add_key(AxisKey::Custom(format!("ymax={}", DETECTOR_LENGTH / 2.0)));
    // Plotting the occupancy preserving the scale of the detector makes it
    // easier to understand it in the context of the detector geometry.
    const S: f64 = 2.0 * PI * CATHODE_PADS_RADIUS;
    const RATIO: f64 = DETECTOR_LENGTH / S;
    // The plot default width is 240 pt, and the default height is 207 pt.
    // Rescale the height to preserve the aspect ratio of the detector.
    const NEW_HEIGHT: f64 = 240.0 * RATIO;
    axis.add_key(AxisKey::Custom(format!(
        "width=240pt, height={}pt",
        NEW_HEIGHT as u16
    )));
    // Drawing the PWB boundaries helps identify problems/hot channels/etc.
    for column in 0..(TPC_PWB_COLUMNS - 1) {
        let column = TpcPwbColumn::try_from(column).unwrap();

        let mut pwb_edge_plot = Plot2D::new();
        pwb_edge_plot.add_key(PlotKey::Custom(String::from("draw=gray!20")));

        let top_row = TpcPwbRow::try_from(TPC_PWB_ROWS - 1).unwrap();
        let top_position = TpcPwbPosition::new(column, top_row);
        let top_right_pad = PwbPadPosition::new(
            PwbPadColumn::try_from(PWB_PAD_COLUMNS - 1).unwrap(),
            PwbPadRow::try_from(PWB_PAD_ROWS - 1).unwrap(),
        );
        let top_right_pad_position = TpcPadPosition::new(top_position, top_right_pad);
        // The above is the position of the center of the pad. We want the edge,
        // so we add half the pitch in both directions.
        pwb_edge_plot.coordinates.push(
            (
                top_right_pad_position.phi() + PAD_PITCH_PHI / 2.0,
                top_right_pad_position.z() + PAD_PITCH_Z / 2.0,
            )
                .into(),
        );

        let bottom_row = TpcPwbRow::try_from(0).unwrap();
        let bottom_position = TpcPwbPosition::new(column, bottom_row);
        let bottom_right_pad = PwbPadPosition::new(
            PwbPadColumn::try_from(PWB_PAD_COLUMNS - 1).unwrap(),
            PwbPadRow::try_from(0).unwrap(),
        );
        let bottom_right_pad_position = TpcPadPosition::new(bottom_position, bottom_right_pad);
        pwb_edge_plot.coordinates.push(
            (
                bottom_right_pad_position.phi() + PAD_PITCH_PHI / 2.0,
                bottom_right_pad_position.z() - PAD_PITCH_Z / 2.0,
            )
                .into(),
        );

        axis.plots.push(pwb_edge_plot);
    }
    for row in 0..(TPC_PWB_ROWS - 1) {
        let row = TpcPwbRow::try_from(row).unwrap();

        let mut pwb_edge_plot = Plot2D::new();
        pwb_edge_plot.add_key(PlotKey::Custom(String::from("draw=gray!20")));

        let left_column = TpcPwbColumn::try_from(0).unwrap();
        let left_position = TpcPwbPosition::new(left_column, row);
        let left_top_pad = PwbPadPosition::new(
            PwbPadColumn::try_from(0).unwrap(),
            PwbPadRow::try_from(PWB_PAD_ROWS - 1).unwrap(),
        );
        let left_top_pad_position = TpcPadPosition::new(left_position, left_top_pad);
        pwb_edge_plot.coordinates.push(
            (
                left_top_pad_position.phi() - PAD_PITCH_PHI / 2.0,
                left_top_pad_position.z() + PAD_PITCH_Z / 2.0,
            )
                .into(),
        );

        let right_column = TpcPwbColumn::try_from(TPC_PWB_COLUMNS - 1).unwrap();
        let right_position = TpcPwbPosition::new(right_column, row);
        let right_top_pad = PwbPadPosition::new(
            PwbPadColumn::try_from(PWB_PAD_COLUMNS - 1).unwrap(),
            PwbPadRow::try_from(PWB_PAD_ROWS - 1).unwrap(),
        );
        let right_top_pad_position = TpcPadPosition::new(right_position, right_top_pad);
        pwb_edge_plot.coordinates.push(
            (
                right_top_pad_position.phi() + PAD_PITCH_PHI / 2.0,
                right_top_pad_position.z() + PAD_PITCH_Z / 2.0,
            )
                .into(),
        );

        axis.plots.push(pwb_edge_plot);
    }

    let mut pads_plot = Plot2D::new();
    pads_plot.add_key(PlotKey::Type2D(Type2D::OnlyMarks));
    pads_plot.add_key(PlotKey::Custom(String::from("mark=square*")));
    // Guessed by trial and error to match the size of the pads in the
    // TPC. There is no way in PGFPlots to set the size of the marks in units
    // of the plot.
    pads_plot.add_key(PlotKey::Custom(format!(
        "mark options={{xscale=1.45, yscale=0.1}}"
    )));
    pads_plot.add_key(PlotKey::Custom(String::from("draw=black!90")));
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
                pads_plot
                    .coordinates
                    .push((pad_position.phi(), pad_position.z()).into());
            }
        }
    }
    axis.plots.push(pads_plot);

    Picture::from(axis)
}
