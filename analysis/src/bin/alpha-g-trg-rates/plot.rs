use crate::delta_packet::DeltaPacket;
use alpha_g_detector::trigger::TRG_CLOCK_FREQ;

#[derive(Clone, Debug)]
pub struct Histogram {
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
}

impl Histogram {
    /// Create a new empty histogram.
    // `initial_timestamp` is in clock units, and `time_step` is in seconds.
    pub fn new(initial_timestamp: u64, time_step: f64) -> Histogram {
        Histogram {
            initial_timestamp,
            time_step,
            data: vec![0.0],
            last_timestamp: 0,
        }
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
    pub fn update(&mut self, delta_timestamp: u32, delta_count: u32) {
        let current_timestamp = self.last_timestamp + u64::from(delta_timestamp);
        let current_time = current_timestamp as f64 / TRG_CLOCK_FREQ;
        let delta_time = f64::from(delta_timestamp) / TRG_CLOCK_FREQ;

        if self.current_right_edge() <= current_time {
            let percentage = (self.current_right_edge() - self.current_left_edge()) / delta_time;
            *self.data.last_mut().unwrap() += percentage * f64::from(delta_count);
            self.data.push(0.0);

            while self.current_right_edge() <= current_time {
                let percentage = self.time_step / delta_time;
                *self.data.last_mut().unwrap() += percentage * f64::from(delta_count);
                self.data.push(0.0);
            }
        }

        let left_edge = f64::max(self.previous_right_edge(), self.current_left_edge());
        let percentage = (current_time - left_edge) / delta_time;
        *self.data.last_mut().unwrap() += percentage * f64::from(delta_count);

        self.last_timestamp = current_timestamp;
    }
}

#[derive(Clone, Debug)]
pub struct Figure {
    output: Histogram,
    input: Histogram,
    pulser: Histogram,
    drift_veto: Histogram,
    scaledown: Histogram,
}

impl Figure {
    pub fn new(initial_timestamp: u64, time_step: f64) -> Figure {
        Figure {
            output: Histogram::new(initial_timestamp, time_step),
            input: Histogram::new(initial_timestamp, time_step),
            pulser: Histogram::new(initial_timestamp, time_step),
            drift_veto: Histogram::new(initial_timestamp, time_step),
            scaledown: Histogram::new(initial_timestamp, time_step),
        }
    }
    pub fn update(&mut self, delta_packet: &DeltaPacket) {
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

#[cfg(test)]
mod tests;
