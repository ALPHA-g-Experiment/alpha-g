use crate::Packet;
use pgfplots::axis::{plot::*, *};
use std::path::Path;
use std::process::{Command, Stdio};

/// Jobname for pdflatex.
pub const JOBNAME: &str = "figure";

/// Create an empty plot.
pub fn empty_plot<P: AsRef<Path>>(dir: P) {
    let axis = Axis::new();
    let argument = axis.standalone_string().replace('\n', "").replace('\t', "");
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
    let mut signal = Plot2D::new();
    signal.coordinates = packet
        .pwb_packet
        .waveform_at(packet.channel_id)
        .unwrap()
        .iter()
        .enumerate()
        .map(|c| (c.0 as f64, f64::from(*c.1)).into())
        .collect();

    let mut axis = Axis::new();
    axis.plots.push(signal);
    let argument = axis.standalone_string().replace('\n', "").replace('\t', "");
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
