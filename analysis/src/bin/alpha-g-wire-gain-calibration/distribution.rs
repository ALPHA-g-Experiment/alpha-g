use alpha_g_detector::alpha16::{ADC_MAX, ADC_MIN};

pub(crate) struct Distribution {
    samples: Vec<usize>,
}

impl Distribution {
    /// Create a new empty distribution.
    pub(crate) fn new() -> Self {
        // The minimum possible amplitude value is ADC_MIN - ADC_MAX.
        // The maximum possible amplitude value is ADC_MAX - ADC_MIN.
        // Therefore, the number of possible amplitude values is
        // 2 * (ADC_MAX - ADC_MIN) + 1
        Self {
            // Need to cast to i32 to avoid overflow
            samples: vec![0; (2 * (i32::from(ADC_MAX) - i32::from(ADC_MIN)) + 1) as usize],
        }
    }
    /// Add a sample to the distribution.
    /// The sample is expected to be in the range [ADC_MIN - ADC_MAX, ADC_MAX - ADC_MIN].
    /// If the sample is outside this range, it will be clamped to the range.
    pub(crate) fn add_sample(&mut self, sample: i32, multiplicity: usize) {
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);
        let max = i32::from(ADC_MAX) - i32::from(ADC_MIN);
        let sample = sample.clamp(min, max);
        // The minimum value corresponds to index 0
        let index = (sample - min) as usize;
        self.samples[index] += multiplicity;
    }
    /// Re-scale the samples with a given factor.
    pub(crate) fn rescale(&self, factor: f64) -> Self {
        // Sample that corresponds to index 0
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);

        let mut scaled = Self::new();
        for (index, count) in self
            .samples
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
        {
            let sample = min + index as i32;
            let scaled_sample = f64::from(sample) * factor;

            // Add `count` samples to the scaled distribution
            scaled.add_sample(scaled_sample.round() as i32, *count);
        }

        scaled
    }
}

#[cfg(test)]
mod tests;
