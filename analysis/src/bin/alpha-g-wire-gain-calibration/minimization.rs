use crate::distribution::{CumulativeDistribution, Distribution};
use anyhow::Context;
use argmin::core::{CostFunction, Error, Executor, IterState};
use argmin::solver::goldensectionsearch::GoldenSectionSearch;

struct Problem {
    // Pivot is the target cumulative distribution. Store as a cumulative
    // distribution instead of a distribution to avoid recomputing every
    // iteration.
    pivot: CumulativeDistribution,
    free: Distribution,
    // To get the best estimate of the gain possible, we want both distributions
    // to saturate and be suppressed at the same amplitudes.
    negative_saturation: i32,
    positive_saturation: i32,
    suppression_threshold: i32,
}

impl Problem {
    fn new(
        pivot: Distribution,
        free: Distribution,
        negative_saturation: i32,
        positive_saturation: i32,
        suppression_threshold: i32,
    ) -> Self {
        let pivot = pivot
            .saturate(negative_saturation, positive_saturation)
            .suppress(suppression_threshold);
        let pivot = CumulativeDistribution::from_distribution(&pivot);

        Self {
            pivot,
            free,
            negative_saturation,
            positive_saturation,
            suppression_threshold,
        }
    }
}

impl CostFunction for Problem {
    type Param = f64;
    type Output = f64;

    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let free = self
            .free
            .clone()
            .rescale(*p)
            .saturate(self.negative_saturation, self.positive_saturation)
            .suppress(self.suppression_threshold);
        let free = CumulativeDistribution::from_distribution(&free);

        Ok(self.pivot.ks_distance(&free))
    }
}

/// Try to find the best rescaling factor for the free distribution. The best
/// rescaling factor is the one that minimizes the Kolmogorov-Smirnov distance
/// between the pivot and the free distribution.
pub(crate) fn try_minimization(
    pivot: &Distribution,
    free: &Distribution,
    negative_saturation: i32,
    positive_saturation: i32,
    suppression_threshold: i32,
    initial_rescale_factor: f64,
) -> Result<IterState<f64, (), (), (), f64>, Error> {
    let problem = Problem::new(
        pivot.clone(),
        free.clone(),
        negative_saturation,
        positive_saturation,
        suppression_threshold,
    );
    // With Golden Search method I need to provide a `lower_bound` and an
    // `upper_bound`. The minimization problem we are trying to solve here has
    // a logical lower bound of 0.0 (the free distribution should not be
    // rescaled to a negative value).
    // The upper bound is a bit more tricky. The typical value for the
    // data suppression threshold in anode wire waveforms is approximately
    // 1500. The range of the waveform is approximately between [i16::MIN,
    // i16::MAX]. Then a scaling factor of 10 corresponds to an equivalent
    // suppression of 15000 in another wire. This is already excessive, so any
    // value above 10 is not even worth dealing with (at that point it might
    // be better to mask that wire).
    let solver = GoldenSectionSearch::new(0.0, 10.0)
        .unwrap()
        // Arbitrary low number that gave good results while playing around.
        .with_tolerance(0.0001)
        .unwrap();

    let res = Executor::new(problem, solver)
        .configure(|state| state.param(initial_rescale_factor).target_cost(0.0))
        .run()
        .context("failed to run minimization executor")?;

    Ok(res.state)
}

#[cfg(test)]
mod tests;
