use statrs::distribution::{ContinuousCDF, StudentsT};

/// Calculate the t-statistic given a pair of samples and an equivalence limit.
fn t_statistic(
    // The samples are: (mean, error, number of samples)
    new_calib: (f64, f64, usize),
    old_calib: (f64, f64, usize),
    limit: f64,
) -> f64 {
    let (new_mean, new_std_err, _) = new_calib;
    let (old_mean, old_std_err, _) = old_calib;
    // Welch's unequal variance t-test
    (new_mean - old_mean - limit) / (new_std_err.powi(2) + old_std_err.powi(2)).sqrt()
}

/// Calculate the degrees of freedom given a pair of samples.
fn dof(new_calib: (f64, f64, usize), old_calib: (f64, f64, usize)) -> f64 {
    let (_, new_std_err, new_sample_size) = new_calib;
    let (_, old_std_err, old_sample_size) = old_calib;
    // Sattherthwaite correction
    (old_std_err.powi(2) + new_std_err.powi(2)).powi(2)
        / (old_std_err.powi(4) / (old_sample_size - 1) as f64
            + new_std_err.powi(4) / (new_sample_size - 1) as f64)
}

/// Calculate the p-value given t_lower, t_upper and degrees of freedom.
fn p_value(t_lower: f64, t_upper: f64, dof: f64) -> f64 {
    let student_t = StudentsT::new(0.0, 1.0, dof).unwrap();

    let p_lower = 1.0 - student_t.cdf(t_lower);
    let p_upper = student_t.cdf(t_upper);
    // Two samples are considered equivalent if both p-values are below 0.05.
    // Hence, the maximum of the two p-values is returned.
    if p_lower > p_upper {
        p_lower
    } else {
        p_upper
    }
}

/// Perform a TOST test on a pair of samples and an equivalence limit.
/// Returns the p-value of the test.
// The null hypothesis is that the difference between the two means is outside
// the equivalence limit.
// i.e. low p-value -> they are equivalent
pub(crate) fn tost(new_calib: (f64, f64, usize), old_calib: (f64, f64, usize), limit: f64) -> f64 {
    let limit = limit.abs();

    let t_lower = t_statistic(new_calib, old_calib, -limit);
    let t_upper = t_statistic(new_calib, old_calib, limit);
    let dof = dof(new_calib, old_calib);

    p_value(t_lower, t_upper, dof)
}

#[cfg(test)]
mod tests;
