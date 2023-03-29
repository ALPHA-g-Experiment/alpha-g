/// Determine if the waveform is noise.
pub(crate) fn is_noise(_waveform: &[i16]) -> bool {
    true
}

/// Calculate the mean of a slice of `i16`.
pub(crate) fn mean(slice: &[i16]) -> f64 {
    slice.iter().map(|&x| f64::from(x)).sum::<f64>() / slice.len() as f64
}

/// Calculate the sample covariance of two slices of `i16`.
// Truncate the longer slice to the length of the shorter slice.
pub(crate) fn cov(slice_1: &[i16], slice_2: &[i16]) -> f64 {
    // Still truncate before zip to calculate the correct mean.
    let (slice_1, slice_2) = if slice_1.len() > slice_2.len() {
        (&slice_1[..slice_2.len()], slice_2)
    } else {
        (slice_1, &slice_2[..slice_1.len()])
    };
    let mean_1 = mean(slice_1);
    let mean_2 = mean(slice_2);
    slice_1
        .iter()
        .zip(slice_2.iter())
        .map(|(&x, &y)| (f64::from(x) - mean_1) * (f64::from(y) - mean_2))
        .sum::<f64>()
        / (slice_1.len() - 1) as f64
}

/// Calculate the sample standard deviation of a slice of `i16`.
pub(crate) fn std_dev(slice: &[i16]) -> f64 {
    cov(slice, slice).sqrt()
}

#[cfg(test)]
mod tests;
