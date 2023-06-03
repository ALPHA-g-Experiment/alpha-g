use crate::distribution::CumulativeDistribution;
use pgfplots::axis::plot::*;

impl CumulativeDistribution {
    /// Return a plot of the cumulative distribution.
    // `num_coordinates` is the number of equally spaced coordinates to use
    // between the last 0.0 and the first 1.0.
    pub(crate) fn plot(&self, num_coordinates: usize) -> Plot2D {
        let mut plot = Plot2D::new();
        let coordinates = self
            .samples
            .iter()
            .enumerate()
            // A LOT of the samples at the beginning are 0.0, and a LOT of the
            // samples at the end are 1.0. We don't care about plotting those.
            .filter(|&(_, &value)| value > 0.0 && value < 1.0)
            .map(|(index, value)| (f64::from(self.min + index as i16), *value).into());
        // We want only `num_coordinates`. Or if the cumulative distribution
        // is empty, just leave the plot empty.
        let step = coordinates.clone().count().saturating_sub(1) / (num_coordinates - 1);
        if step > 0 {
            plot.coordinates = coordinates.step_by(step).collect();
        }

        plot
    }
}
