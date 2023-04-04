use alpha_g_detector::alpha16::{ADC_MAX, ADC_MIN};

#[derive(Clone, Debug)]
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
    pub(crate) fn rescale(self, factor: f64) -> Self {
        // Sample that corresponds to index 0
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);

        let mut scaled = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = min + index as i32;
            let scaled_sample = f64::from(sample) * factor;

            scaled.add_sample(scaled_sample.round() as i32, count);
        }

        scaled
    }
    /// Saturate the distribution at a given threshold.
    /// All samples greater than the `positive_threshold` will be set to
    /// `positive_threshold`.
    /// All samples less than the `negative_threshold` will be set to
    /// `negative_threshold`.
    pub(crate) fn saturate(self, negative_threshold: i32, positive_threshold: i32) -> Self {
        // Sample that corresponds to index 0
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);

        let mut saturated = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = min + index as i32;
            let sample = sample.clamp(negative_threshold, positive_threshold);

            saturated.add_sample(sample, count);
        }
        saturated
    }
    /// Suppress the distribution at a given threshold.
    /// All samples between -`threshold` and +`threshold` (inclusive) will be
    /// suppressed.
    pub(crate) fn suppress(self, threshold: i32) -> Self {
        // Sample that corresponds to index 0
        let min = i32::from(ADC_MIN) - i32::from(ADC_MAX);

        let threshold = threshold.abs();
        let mut suppressed = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = min + index as i32;
            if sample.abs() <= threshold {
                continue;
            }

            suppressed.add_sample(sample, count);
        }
        suppressed
    }
}

pub(crate) struct CumulativeDistribution {
    samples: Vec<f64>,
}

impl CumulativeDistribution {
    /// Create a cumulative distribution from a distribution.
    // The distribution only has raw counts, so we need to first accumulate
    // the counts, and then normalize.
    pub(crate) fn from_distribution(distribution: &Distribution) -> Self {
        let mut samples = Vec::with_capacity(distribution.samples.len());
        let mut sum = 0;
        for count in distribution.samples.iter() {
            sum += count;
            samples.push(sum);
        }

        let total = samples.last().copied().unwrap_or(0);
        let samples = samples
            .into_iter()
            .map(|count| count as f64 / total as f64)
            .collect();

        Self { samples }
    }
    /// Calculate the Kolmogorov-Smirnov distance between two cumulative
    /// distributions.
    pub(crate) fn ks_distance(&self, other: &Self) -> f64 {
        let mut max_distance = 0.0;
        for (self_sample, other_sample) in self.samples.iter().zip(other.samples.iter()) {
            let distance = (self_sample - other_sample).abs();
            if distance > max_distance {
                max_distance = distance;
            }
        }
        max_distance
    }
}

#[cfg(test)]
mod tests;
