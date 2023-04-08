use crate::distribution::CumulativeDistribution;
use alpha_g_detector::alpha16::{ADC_MAX, ADC_MIN};
use pgfplots::axis::plot::*;

impl CumulativeDistribution {
    /// Return a plot of the cumulative distribution.
    // `num_coordinates` is the number of equally spaced coordinates to use
    // between the last 0.0 and the first 1.0.
    pub(crate) fn plot(&self, num_coordinates: usize) -> Plot2D {
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);

        let mut plot = Plot2D::new();
        let coordinates = self
            .samples
            .iter()
            .enumerate()
            // A LOT of the samples at the beginning are 0.0, and a LOT of the
            // samples at the end are 1.0. We don't care about plotting those.
            .filter(|&(_, &value)| value > 0.0 && value < 1.0)
            .map(|(index, value)| (f64::from(min + index as i32), *value).into());
        // We want only `num_coordinates`
        let step = (coordinates.clone().count() - 1) / (num_coordinates - 1);
        plot.coordinates = coordinates.step_by(step).collect();

        plot
    }
}
