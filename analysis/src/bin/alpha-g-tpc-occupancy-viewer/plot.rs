use crate::next::Packet;
use alpha_g_detector::alpha16::{self, aw_map::*};
use alpha_g_detector::padwing::map::*;
use alpha_g_detector::padwing::ChannelId;
use anyhow::{Context, Result};
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
pub fn create_picture(packet: &Packet) -> Result<Picture> {
    // Shared between both axis environments.
    const X_MIN: f64 = 0.0;
    const X_MAX: f64 = 2.0 * PI;
    // Default width in pt units.
    const WIDTH: f64 = 240.0;

    let mut wires_plot = Plot2D::new();
    wires_plot.add_key(PlotKey::Type2D(Type2D::OnlyMarks));
    wires_plot.add_key(PlotKey::Custom(String::from("mark=square*")));
    // Guessed by trial and error to fill the gap between adjacent wires.
    // Makes it easier to visualize clusters.
    wires_plot.add_key(PlotKey::Custom(String::from(
        "mark options={{xscale=0.10, yscale=0.25}}",
    )));
    wires_plot.add_key(PlotKey::Custom(String::from("draw=black!90")));
    for adc_packet in &packet.adc_packets {
        let waveform = adc_packet.waveform();
        if waveform.is_empty() {
            continue;
        }
        // Additionally check that it is not flat. It can be flat and non-empty
        // when data suppression is disabled.
        let is_flat = waveform.iter().all(|&v| v == waveform[0]);
        if is_flat {
            continue;
        }

        let board_id = adc_packet
            .board_id()
            .expect("board id always exist if waveform is not empty");
        let alpha16::ChannelId::A32(channel_id) = adc_packet.channel_id() else {
            panic!("worker thread checked that only A32 channels are sent");
        };
        let wire_position = TpcWirePosition::try_new(packet.run_number, board_id, channel_id)
            .context("wire position mapping failed. Please open an issue on Github.")?;

        let phi = wire_position.phi();
        wires_plot.coordinates.push((phi, 0.0).into());
    }

    let mut wire_axis = Axis::from(wires_plot);
    wire_axis.set_title("rTPC Occupancy");
    wire_axis.add_key(AxisKey::Custom(String::from("name=wires")));
    wire_axis.add_key(AxisKey::Custom(String::from("ticks=none")));
    wire_axis.add_key(AxisKey::Custom(format!(
        "width={}pt, height=60pt",
        WIDTH as u16,
    )));
    wire_axis.add_key(AxisKey::Custom(format!("xmin={X_MIN}")));
    wire_axis.add_key(AxisKey::Custom(format!("xmax={X_MAX}")));
    wire_axis.add_key(AxisKey::Custom(String::from("ymin=-1")));
    wire_axis.add_key(AxisKey::Custom(String::from("ymax=1")));
    // Drawing the preamp boundaries makes it easier to visualize the mapping.
    wire_axis.plots.extend(vertical_awb_divisions());

    let mut pads_plot = Plot2D::new();
    pads_plot.add_key(PlotKey::Type2D(Type2D::OnlyMarks));
    pads_plot.add_key(PlotKey::Custom(String::from("mark=square*")));
    // Guessed by trial and error to match the size of the pads in the
    // TPC. There is no way in PGFPlots to set the size of the marks in units
    // of the plot.
    pads_plot.add_key(PlotKey::Custom(String::from(
        "mark options={{xscale=1.45, yscale=0.1}}",
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
                    TpcPadPosition::try_new(packet.run_number, board_id, after_id, pad_channel)
                        .context("pad position mapping failed. Please open an issue on GitHub.")?;
                pads_plot
                    .coordinates
                    .push((pad_position.phi(), pad_position.z()).into());
            }
        }
    }

    let mut pad_axis = Axis::from(pads_plot);
    pad_axis.add_key(AxisKey::Custom(String::from("at=(wires.south)")));
    pad_axis.add_key(AxisKey::Custom(String::from("anchor=north")));
    pad_axis.add_key(AxisKey::Custom(String::from("yshift=-5pt")));
    pad_axis.set_x_label("$\\phi$~[rad]");
    pad_axis.set_y_label("$z$~[m]");
    pad_axis.add_key(AxisKey::Custom(format!("xmin={X_MIN}")));
    pad_axis.add_key(AxisKey::Custom(format!("xmax={X_MAX}")));
    pad_axis.add_key(AxisKey::Custom(format!("ymin={}", -DETECTOR_LENGTH / 2.0)));
    pad_axis.add_key(AxisKey::Custom(format!("ymax={}", DETECTOR_LENGTH / 2.0)));
    // Plotting the occupancy preserving the scale of the detector makes it
    // easier to understand it in the context of the detector geometry.
    const S: f64 = 2.0 * PI * CATHODE_PADS_RADIUS;
    const RATIO: f64 = DETECTOR_LENGTH / S;
    // The plot default width is 240 pt, and the default height is 207 pt.
    // Rescale the height to preserve the aspect ratio of the detector.
    const NEW_HEIGHT: f64 = WIDTH * RATIO;
    pad_axis.add_key(AxisKey::Custom(format!(
        "width={}pt, height={}pt",
        WIDTH as u16, NEW_HEIGHT as u16
    )));
    // Drawing the PWB boundaries helps identify problems/hot channels/etc.
    pad_axis.plots.extend(vertical_pwb_divisions());
    pad_axis.plots.extend(horizontal_pwb_divisions());

    let mut picture = Picture::new();
    picture.axes = vec![wire_axis, pad_axis];
    Ok(picture)
}

fn vertical_awb_divisions() -> Vec<Plot2D> {
    let mut awb_divisions = Vec::new();
    // There are 16 preamps, each with 16 wires.
    for awb in 0..=15 {
        let right_wire = TpcWirePosition::try_from(awb * 16 + 15).unwrap();

        let mut awb_edge_plot = Plot2D::new();
        awb_edge_plot.add_key(PlotKey::Custom(String::from("draw=gray!20")));

        awb_edge_plot
            .coordinates
            .push((right_wire.phi() + ANODE_WIRE_PITCH_PHI / 2.0, 1.0).into());
        awb_edge_plot
            .coordinates
            .push((right_wire.phi() + ANODE_WIRE_PITCH_PHI / 2.0, -1.0).into());

        awb_divisions.push(awb_edge_plot);
    }

    awb_divisions
}

fn vertical_pwb_divisions() -> Vec<Plot2D> {
    let mut pwb_divisions = Vec::new();
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

        pwb_divisions.push(pwb_edge_plot);
    }

    pwb_divisions
}

fn horizontal_pwb_divisions() -> Vec<Plot2D> {
    let mut pwb_divisions = Vec::new();
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
        // The above is the position of the center of the pad. We want the edge,
        // so we add half the pitch in both directions.
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

        pwb_divisions.push(pwb_edge_plot);
    }

    pwb_divisions
}
