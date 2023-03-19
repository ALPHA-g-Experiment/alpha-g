use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};
use std::collections::HashMap;

pub(crate) fn calibration_picture(
    run_number: u32,
    // Recall the tuple is (baseline, error, number of samples)
    map: &HashMap<TpcWirePosition, (f64, f64, usize)>,
) -> Picture {
    let mut plot = Plot2D::new();
    plot.add_key(PlotKey::Custom(String::from("scatter, only marks")));
    plot.add_key(PlotKey::YError(ErrorCharacter::Absolute));
    plot.add_key(PlotKey::YErrorDirection(ErrorDirection::Both));
    plot.coordinates = map
        .iter()
        .map(|(wire, (y, sigma, _))| (wire.phi(), *y, None, Some(*sigma)).into())
        .collect();

    let mut axis = Axis::from(plot);
    axis.set_title(format!("Run {run_number}. Anode wire baseline calibration"));
    axis.set_x_label("$\\phi$~[rad]");
    axis.set_y_label("Baseline~[a.u.]");
    axis.add_key(AxisKey::Custom(String::from("no markers")));
    // Default height but twice the default width.
    // It just makes the points be less squished together.
    axis.add_key(AxisKey::Custom(String::from("width=480pt, height=207pt")));

    Picture::from(axis)
}
