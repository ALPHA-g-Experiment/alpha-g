use crate::Packet;
use alpha_g_detector::padwing::ChannelId::{Fpn, Pad, Reset};
use alpha_g_detector::padwing::{suppression_baseline, PADWING_RATE};
use pgfplots::axis::{plot::*, *};
use std::path::Path;
use std::process::{Command, Stdio};

/// Jobname for pdflatex.
pub const JOBNAME: &str = "figure";

/// Create an empty plot.
pub fn empty_plot<P: AsRef<Path>>(dir: P) {
    let axis = Axis::new();
    let argument = axis.standalone_string().replace(['\n', '\t'], "");
    Command::new("pdflatex")
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-interaction=batchmode")
        .arg("-halt-on-error")
        .arg("-jobname=".to_string() + JOBNAME)
        .arg(argument)
        .status()
        .expect("failed to run pdflatex");
}

/// Create a plot based on an input Packet.
pub fn create_plot<P: AsRef<Path>>(dir: P, packet: &Packet) {
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
    axis.set_x_label(format!("Samples~[{} ns]", 1e9 / PADWING_RATE));
    axis.set_y_label("Amplitude~[a.u.]");
    axis.add_key(AxisKey::Custom("ymin=-2048, ymax=2047".to_string()));

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
    let argument = axis.standalone_string().replace(['\n', '\t'], "");
    Command::new("pdflatex")
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-interaction=batchmode")
        .arg("-halt-on-error")
        .arg("-jobname=".to_string() + JOBNAME)
        .arg(argument)
        .status()
        .expect("failed to run pdflatex");
}
