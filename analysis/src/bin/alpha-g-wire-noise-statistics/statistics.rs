/// Determine if the waveform is noise.
// Those which are not noise are not considered to estimate the baseline.
pub(crate) fn is_noise(_waveform: &[i16]) -> bool {
    true
}

/// Calculate the mean of a slice of `i16`.
pub(crate) fn mean(slice: &[i16]) -> f64 {
    slice.iter().map(|&x| f64::from(x)).sum::<f64>() / slice.len() as f64
}

/// Calculate the sample standard deviation of a slice of `i16`.
pub(crate) fn std_dev(slice: &[i16]) -> f64 {
    let mean = mean(slice);
    let sum = slice
        .iter()
        .map(|&x| f64::from(x))
        .fold(0.0, |acc, x| acc + (x - mean).powi(2));
    (sum / (slice.len() - 1) as f64).sqrt()
}

#[cfg(test)]
mod tests;
