use super::*;

#[test]
fn new_empty_distribution() {
    let d = Distribution::new();
    assert_eq!(
        d.samples.len(),
        (2 * (i32::from(ADC_MAX) - i32::from(ADC_MIN)) + 1) as usize
    );
    assert_eq!(d.samples.iter().sum::<usize>(), 0);
}

#[test]
fn distribution_add_samples() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d = Distribution::new();
    d.add_sample(min - max, 1);
    assert_eq!(d.samples[0], 1);
    assert_eq!(d.samples.iter().sum::<usize>(), 1);

    d.add_sample(min - max - 1, 5);
    assert_eq!(d.samples[0], 6);
    assert_eq!(d.samples.iter().sum::<usize>(), 6);

    d.add_sample(max - min, 1);
    assert_eq!(d.samples.last().unwrap(), &1);
    assert_eq!(d.samples.iter().sum::<usize>(), 7);

    d.add_sample(max - min + 1, 5);
    assert_eq!(d.samples.last().unwrap(), &6);
    assert_eq!(d.samples.iter().sum::<usize>(), 12);

    d.add_sample(0, 1000);
    assert_eq!(d.samples[(max - min) as usize], 1000);
    assert_eq!(d.samples.iter().sum::<usize>(), 1012);
}

#[test]
fn distribution_rescale() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d = Distribution::new();
    d.add_sample(0, 1000);
    d.add_sample(1, 1010);
    d.add_sample(-1, 990);

    let rescaled = d.clone().rescale(2.0);
    assert_eq!(rescaled.samples[(max - min) as usize], 1000);
    assert_eq!(rescaled.samples[(max - min + 2) as usize], 1010);
    assert_eq!(rescaled.samples[(max - min - 2) as usize], 990);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);

    let rescaled = d.clone().rescale(0.45);
    assert_eq!(rescaled.samples[(max - min) as usize], 3000);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);

    let rescaled = d.clone().rescale(1000000000.0);
    assert_eq!(rescaled.samples[(max - min) as usize], 1000);
    assert_eq!(rescaled.samples[0], 990);
    assert_eq!(rescaled.samples.last().unwrap(), &1010);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);
}

#[test]
fn distribution_saturate() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d = Distribution::new();
    d.add_sample(-2, 10);
    d.add_sample(-1, 100);
    d.add_sample(0, 1000);
    d.add_sample(1, 10000);
    d.add_sample(2, 100000);

    let saturated = d.clone().saturate(-20, 20);
    assert_eq!(saturated.samples, d.samples);

    let saturated = d.clone().saturate(-2, -1);
    assert_eq!(saturated.samples[(max - min - 2) as usize], 10);
    assert_eq!(saturated.samples[(max - min - 1) as usize], 111100);

    let saturated = d.clone().saturate(1, 2);
    assert_eq!(saturated.samples[(max - min + 1) as usize], 11110);
    assert_eq!(saturated.samples[(max - min + 2) as usize], 100000);
}

#[test]
fn distribution_suppression() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d = Distribution::new();
    d.add_sample(-1, 10);
    d.add_sample(0, 100);
    d.add_sample(1, 1000);

    let suppressed = d.clone().suppress(0);
    assert_eq!(suppressed.samples[(max - min - 1) as usize], 10);
    assert_eq!(suppressed.samples[(max - min) as usize], 0);
    assert_eq!(suppressed.samples[(max - min + 1) as usize], 1000);

    d.add_sample(-2, 5);
    d.add_sample(2, 5000);

    let suppressed = d.clone().suppress(1);
    assert_eq!(suppressed.samples[(max - min - 2) as usize], 5);
    assert_eq!(suppressed.samples[(max - min - 1) as usize], 0);
    assert_eq!(suppressed.samples[(max - min) as usize], 0);
    assert_eq!(suppressed.samples[(max - min + 1) as usize], 0);
    assert_eq!(suppressed.samples[(max - min + 2) as usize], 5000);
}

#[test]
fn cumulative_distribution_from_distribution() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d = Distribution::new();
    d.add_sample(max - min, 1);

    let c = CumulativeDistribution::from_distribution(&d);
    assert_eq!(c.samples[2 * (max - min) as usize], 1.0);
    assert_eq!(c.samples[2 * (max - min) as usize - 1], 0.0);

    d.add_sample(min - max, 1);
    let c = CumulativeDistribution::from_distribution(&d);
    assert_eq!(c.samples[0], 0.5);
    assert_eq!(c.samples[2 * (max - min) as usize - 1], 0.5);
    assert_eq!(c.samples[2 * (max - min) as usize], 1.0);

    let d = Distribution::new();
    let c = CumulativeDistribution::from_distribution(&d);
    assert_eq!(c.samples.iter().sum::<f64>(), 0.0);
}

#[test]
fn cumulative_distribution_ks_distance() {
    let min = i32::from(ADC_MIN);
    let max = i32::from(ADC_MAX);

    let mut d1 = Distribution::new();
    d1.add_sample(max - min, 1);

    let mut d2 = Distribution::new();
    d2.add_sample(min - max, 1);

    let c1 = CumulativeDistribution::from_distribution(&d1);
    let c2 = CumulativeDistribution::from_distribution(&d2);

    assert_eq!(c1.ks_distance(&c2), 1.0);
    assert_eq!(c2.ks_distance(&c1), 1.0);

    let d1 = Distribution::new();
    let d2 = Distribution::new();

    let c1 = CumulativeDistribution::from_distribution(&d1);
    let c2 = CumulativeDistribution::from_distribution(&d2);

    assert_eq!(c1.ks_distance(&c2), 0.0);
    assert_eq!(c2.ks_distance(&c1), 0.0);

    let mut d1 = Distribution::new();
    d1.add_sample(max - min, 1);
    d1.add_sample(min - max, 1);

    let mut d2 = Distribution::new();
    d2.add_sample(max - min, 2);
    d2.add_sample(min - max, 1);

    let c1 = CumulativeDistribution::from_distribution(&d1);
    let c2 = CumulativeDistribution::from_distribution(&d2);

    let ks_dis = c1.ks_distance(&c2);
    let diff = (ks_dis - 1.0 / 6.0).abs();
    assert!(diff < 1e-6);

    let ks_dis = c2.ks_distance(&c1);
    let diff = (ks_dis - 1.0 / 6.0).abs();
    assert!(diff < 1e-6);
}
