use crate::delta_packet::DeltaPacket;
use crate::Args;
use alpha_g_detector::trigger::TRG_CLOCK_FREQ;
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};

/// Control the style of each histogram in a plot.
#[derive(Clone, Copy, Debug)]
enum HistoStyle {
    Output,
    Scaledown,
    DriftVeto,
    Input,
    Pulser,
}

impl HistoStyle {
    fn type_2d(&self) -> PlotKey {
        PlotKey::Type2D(Type2D::ConstRight)
    }
    fn color(&self) -> PlotKey {
        match self {
            HistoStyle::Output => PlotKey::Custom("black".to_string()),
            HistoStyle::Scaledown => PlotKey::Custom("red!40!gray".to_string()),
            HistoStyle::DriftVeto => PlotKey::Custom("blue!40!gray".to_string()),
            HistoStyle::Input => PlotKey::Custom("gray!50".to_string()),
            HistoStyle::Pulser => PlotKey::Custom("green!40!gray".to_string()),
        }
    }
}

/// Data for an individual counter.
// Automatically bin incoming data by the appropriate `time-step`.
#[derive(Clone, Debug)]
struct Histogram {
    // Corresponds to t=0 (in clock units) of the histogram
    initial_timestamp: u64,
    // Bin size (in seconds)
    time_step: f64,
    // Counts on each bin
    data: Vec<f64>,
    // Book-keeping of the timestamp from the last data entered into the
    // histogram. This is basically the `current left edge` but in clock units
    // with respect to the `initial_timestamp`
    last_timestamp: u64,
    // Style of how to draw the histogram
    style: HistoStyle,
}

impl Histogram {
    /// Create a new empty histogram.
    // `initial_timestamp` is in clock units, and `time_step` is in seconds.
    fn new(initial_timestamp: u64, time_step: f64, style: HistoStyle) -> Histogram {
        Histogram {
            initial_timestamp,
            time_step,
            data: vec![0.0],
            last_timestamp: 0,
            style,
        }
    }
    fn initial_time(&self) -> f64 {
        self.initial_timestamp as f64 / TRG_CLOCK_FREQ
    }
    fn current_left_edge(&self) -> f64 {
        self.last_timestamp as f64 / TRG_CLOCK_FREQ
    }
    fn current_right_edge(&self) -> f64 {
        (self.data.len() as f64) * self.time_step
    }
    fn previous_right_edge(&self) -> f64 {
        ((self.data.len() - 1) as f64) * self.time_step
    }
    /// Update the histogram data.
    // `delta_X` represents the difference between this and the previous data
    // that was added to the histogram.
    fn update(&mut self, delta_timestamp: u32, delta_count: u32) {
        // The new timestamp corresponds to that of the last count.
        // All the other counts i.e. `delta_count - 1` are assumed to be
        // uniformly distributed between the last and the current timestamp.
        // We have no other information about these counts other than they
        // happened in this window.
        //
        // This counter can be zero e.g. for pulser counters; hence the
        // satutating subtraction.
        let spread_count = delta_count.saturating_sub(1);

        let current_timestamp = self.last_timestamp + u64::from(delta_timestamp);
        let current_time = current_timestamp as f64 / TRG_CLOCK_FREQ;
        let delta_time = f64::from(delta_timestamp) / TRG_CLOCK_FREQ;

        if self.current_right_edge() <= current_time {
            let percentage = (self.current_right_edge() - self.current_left_edge()) / delta_time;
            *self.data.last_mut().unwrap() += percentage * f64::from(spread_count);
            self.data.push(0.0);

            while self.current_right_edge() <= current_time {
                let percentage = self.time_step / delta_time;
                *self.data.last_mut().unwrap() += percentage * f64::from(spread_count);
                self.data.push(0.0);
            }
        }

        let left_edge = f64::max(self.previous_right_edge(), self.current_left_edge());
        let percentage = (current_time - left_edge) / delta_time;
        *self.data.last_mut().unwrap() +=
            percentage * f64::from(spread_count) + f64::from(delta_count - spread_count);

        self.last_timestamp = current_timestamp;
    }
}

/// Define the default conversion from Histogram to a Plot2D.
impl From<Histogram> for Plot2D {
    fn from(hist: Histogram) -> Self {
        let mut plot = Plot2D::new();
        // The last bin needs to be normalized differently because it doesn't
        // have the same length (time_step) as all the others.
        let last_bin_length = hist.current_left_edge() - hist.previous_right_edge();
        // Duplicate the t=`time_step` point at t=0 to draw the first bin
        // correctly starting at 0 (needed due to the `ConstRight` type plot).
        plot.coordinates.push(
            (
                hist.initial_time(),
                hist.data[0] / {
                    // Need to check that the first bin is not the last bin.
                    if hist.data.len() != 1 {
                        hist.time_step
                    } else if hist.data[0] != 0.0 {
                        last_bin_length
                    } else {
                        // Nan from 0.0 / 0.0 causes the PDF compilation to fail
                        // If there are 0 counts, this just means 0 Hz; hence
                        // divide by infinity instead of 0.
                        f64::INFINITY
                    }
                },
            )
                .into(),
        );
        // Push every point assuming the same `time_step` normalization.
        // The last point will be "renormalized" again at the end; this is just
        // easier than stop iteration before last point.
        (1u32..)
            .zip(hist.data.iter())
            .map(|(i, &v)| {
                (
                    f64::from(i) * hist.time_step + hist.initial_time(),
                    v / hist.time_step,
                )
                    .into()
            })
            .for_each(|point| plot.coordinates.push(point));

        *plot.coordinates.last_mut().unwrap() = (
            hist.initial_time() + hist.current_left_edge(),
            hist.data.last().unwrap() / {
                if hist.data.last().unwrap() == &0.0 {
                    // Same as above. 0/0 is just be 0 Hz. Avoid Nan
                    f64::INFINITY
                } else {
                    last_bin_length
                }
            },
        )
            .into();

        plot.add_key(hist.style.type_2d());
        plot.add_key(hist.style.color());
        plot
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Figure {
    output: Histogram,
    input: Histogram,
    pulser: Histogram,
    drift_veto: Histogram,
    scaledown: Histogram,
}

impl Figure {
    pub(crate) fn new(initial_timestamp: u64, time_step: f64) -> Figure {
        Figure {
            output: Histogram::new(initial_timestamp, time_step, HistoStyle::Output),
            input: Histogram::new(initial_timestamp, time_step, HistoStyle::Input),
            pulser: Histogram::new(initial_timestamp, time_step, HistoStyle::Pulser),
            drift_veto: Histogram::new(initial_timestamp, time_step, HistoStyle::DriftVeto),
            scaledown: Histogram::new(initial_timestamp, time_step, HistoStyle::Scaledown),
        }
    }
    pub(crate) fn update(&mut self, delta_packet: &DeltaPacket) {
        let delta_timestamp = delta_packet.timestamp;

        self.output
            .update(delta_timestamp, delta_packet.output_counter);
        self.input
            .update(delta_timestamp, delta_packet.input_counter);
        self.pulser
            .update(delta_timestamp, delta_packet.pulser_counter);
        self.drift_veto
            .update(delta_timestamp, delta_packet.drift_veto_counter);
        self.scaledown
            .update(delta_timestamp, delta_packet.scaledown_counter);
    }
}

// Create the final `Picture` that will be used to generate the PDF.
pub(crate) fn create_picture(figures: Vec<Figure>, args: &Args) -> Picture {
    // By default I will only show the `input` and `output` counter.
    // I think these are the most relevant, and keeps the plot "clean".
    // Any other counter can be brought (or removed) with a flag.
    let mut axis = Axis::new();
    axis.set_title(format!("TRG Scalers. Time step of {} s", args.time_step));
    axis.set_x_label("Time~[s]");
    axis.set_y_label("Rate~[Hz]");
    axis.add_key(AxisKey::Custom("ymin=0".to_string()));

    for fig in figures {
        // Add them in the order from higher to lower counts. It helps to make
        // it easier to understand in case of equal counts (lower count e.g.
        // output is more important, so this should overwrite the previous one).
        if !args.remove_input_counter {
            axis.plots.push(fig.input.into());
        }
        if args.include_drift_veto_counter {
            axis.plots.push(fig.drift_veto.into());
        }
        if args.include_scaledown_counter {
            axis.plots.push(fig.scaledown.into());
        }
        if args.include_pulser_counter {
            axis.plots.push(fig.pulser.into());
        }
        if !args.remove_output_counter {
            axis.plots.push(fig.output.into());
        }
    }

    let mut legend = Vec::new();
    if !args.remove_input_counter {
        legend.push("Input counter".to_string());
    }
    if args.include_drift_veto_counter {
        legend.push("Drift veto counter".to_string());
    }
    if args.include_scaledown_counter {
        legend.push("Scaledown counter".to_string());
    }
    if args.include_pulser_counter {
        legend.push("Pulser counter".to_string());
    }
    if !args.remove_output_counter {
        legend.push("Output counter".to_string());
    }

    axis.add_key(AxisKey::Custom(format!(
        "legend entries={{{}}}",
        legend.join(",")
    )));
    axis.add_key(AxisKey::Custom("legend pos=outer north east".to_string()));
    axis.add_key(AxisKey::Custom("legend style={font=\\tiny}".to_string()));

    Picture::from(axis)
}

#[cfg(test)]
mod tests;
