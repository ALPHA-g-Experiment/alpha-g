pub(crate) mod pads;
pub(crate) mod wires;

// Non-negative greedy deconvolution:
//
// In both of our use-cases (wire and pad analysis), the convolution kernel is
// extremely front-heavy. We can take advantage of this and do a "greedy"
// deconvolution where we make a decision at each step based solely on a few
// samples ahead of us. The non-negative constraint is enforced by the fact that
// only positive input values make sense; this also helps to keep small noise
// peaks to a minimum.
// Quantitatively, this gives results extremely close to (expressing the problem
// as a matrix equation) solving least squares iteratively with e.g. Landweber
// iterations. The advantage is that it's orders of magnitude faster.
// Compared to e.g. deconvolution in frequency space (and Wiener filtering),
// this gives better results when imposing non-negative constraints (not as
// easy to do in frequency space). Additionally it doesn't require a good
// understanding of the noise characteristics of the signal.
fn nn_greedy_deconvolution(
    signal: &[f64],
    response: &[f64],
    offset: usize,
    look_ahead: usize,
    // Return the sum of residuals squared together with the reconstructed input
) -> (f64, Vec<f64>) {
    let response_window = &response[offset..][..look_ahead];
    // The response is always negative in the range we are looking at. This is
    // particularly important for the optimization done below.
    // This assert is here just to panic if this assumption is violated
    // accidentally at some point in the future.
    assert!(response_window.iter().all(|&x| x < 0.0));

    let mut residual = signal.to_vec();
    let mut input = vec![0.0; signal.len()];
    // Unnatural way of sliding the window, but it allows a 2x speedup by
    // jumping over chunks of the signal at a time.
    let mut i = 0;
    while i + offset + look_ahead <= residual.len() {
        let residual_window = &residual[i + offset..][..look_ahead];
        // Find the index of the last non-negative value in the residual window.
        // If there is any non-negative value, it means that there was no input,
        // furthermore we can skip time bins until this particular value goes
        // out of the window.
        let last_positive = residual_window
            .iter()
            .enumerate()
            .rev()
            .find(|(_, x)| **x >= 0.0)
            .map(|(i, _)| i);

        if let Some(last_positive) = last_positive {
            i += last_positive + 1;
        } else {
            let val = residual_window
                .iter()
                .zip(response_window)
                .map(|(s, r)| s / r)
                .reduce(f64::min)
                .unwrap();

            input[i] = val;
            residual[i..]
                .iter_mut()
                .zip(response)
                .for_each(|(s, r)| *s -= val * r);

            i += 1;
        }
    }

    let residual = residual.iter().map(|x| x.powi(2)).sum();

    (residual, input)
}

// Least-squares deconvolution:
//
// Given a set of `offset` and `look_ahead` values, return the reconstructed
// input that minimizes the sum of residuals squared.
fn ls_deconvolution<I>(signal: &[f64], response: &[f64], offsets: I, look_aheads: I) -> Vec<f64>
where
    I: Iterator<Item = usize> + Clone,
{
    let mut best_residual = std::f64::INFINITY;
    let mut best_input = Vec::new();

    for offset in offsets {
        for look_ahead in look_aheads.clone() {
            let (residual, input) = nn_greedy_deconvolution(signal, response, offset, look_ahead);
            if residual < best_residual {
                best_residual = residual;
                best_input = input;
            }
        }
    }

    best_input
}
