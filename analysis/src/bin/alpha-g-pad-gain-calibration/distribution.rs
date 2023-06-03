use alpha_g_detector::padwing::{PWB_MAX, PWB_MIN};

#[derive(Clone, Debug)]
pub(crate) struct Distribution {
    samples: Vec<usize>,
    min: i16,
    max: i16,
}

impl Distribution {
    /// Create a new empty distribution.
    pub(crate) fn new() -> Self {
        // The minimum possible amplitude value is PWB_MIN - PWB_MAX.
        // The maximum possible amplitude value is PWB_MAX - PWB_MIN.
        // Therefore, the number of possible amplitude values is
        // 2 * (PWB_MAX - PWB_MIN) + 1
        //
        // In theory these subtraction operations could never overflow (I don't
        // even expect these constants to ever change), but I rather panic if it
        // ever happens. In that case I would have to cast to i32 as the anode
        // wires.
        let min = PWB_MIN.checked_sub(PWB_MAX).unwrap();
        let max = PWB_MAX.checked_sub(PWB_MIN).unwrap();
        Self {
            samples: vec![0; (2 * max + 1) as usize],
            min,
            max,
        }
    }
    /// Add a sample to the distribution.
    /// The sample is expected to be in the range [PWB_MIN - PWB_MAX, PWB_MAX - PWB_MIN].
    /// If the sample is outside this range, it will be clamped to the range.
    pub(crate) fn add_sample(&mut self, sample: i16, multiplicity: usize) {
        let sample = sample.clamp(self.min, self.max);
        // The minimum value corresponds to index 0
        let index = (sample - self.min) as usize;
        self.samples[index] += multiplicity;
    }
    /// Re-scale the samples with a given factor.
    pub(crate) fn rescale(self, factor: f64) -> Self {
        let mut scaled = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = self.min + index as i16;
            let scaled_sample = f64::from(sample) * factor;

            scaled.add_sample(scaled_sample.round() as i16, count);
        }

        scaled
    }
    /// Saturate the distribution at a given threshold.
    /// All samples greater than the `positive_threshold` will be set to
    /// `positive_threshold`.
    /// All samples less than the `negative_threshold` will be set to
    /// `negative_threshold`.
    pub(crate) fn saturate(self, negative_threshold: i16, positive_threshold: i16) -> Self {
        let mut saturated = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = self.min + index as i16;
            let sample = sample.clamp(negative_threshold, positive_threshold);

            saturated.add_sample(sample, count);
        }
        saturated
    }
    /// Suppress the distribution at a given threshold.
    /// All samples between -`threshold` and +`threshold` (inclusive) will be
    /// suppressed.
    pub(crate) fn suppress(self, threshold: i16) -> Self {
        let threshold = threshold.abs();
        let mut suppressed = Self::new();
        for (index, count) in self
            .samples
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            let sample = self.min + index as i16;
            if sample.abs() <= threshold {
                continue;
            }

            suppressed.add_sample(sample, count);
        }
        suppressed
    }
}

pub(crate) struct CumulativeDistribution {
    pub(crate) samples: Vec<f64>,
    pub(crate) min: i16,
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
        // Normalize only if there were samples. Otherwise, leave the cumulative
        // distribution all zeros.
        // It could be argued that a cumulative distribution with all zeros
        // doesn't make sense, but it is completely valid in this context.
        // Such cumulative distribution could show up when all values in the
        // distribution are suppressed.
        // Additionally, the KS distance between such cumulative distribution
        // and a normal (correct) cumulative distribution is 1, which is
        // definitely not the minimum KS distance.
        let normalization = if sum > 0 { sum as f64 } else { 1.0 };
        let samples = samples
            .into_iter()
            .map(|sample| sample as f64 / normalization)
            .collect();

        Self {
            samples,
            min: distribution.min,
        }
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
