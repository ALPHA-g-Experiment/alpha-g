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

    let rescaled = d.rescale(2.0);
    assert_eq!(rescaled.samples[(max - min) as usize], 1000);
    assert_eq!(rescaled.samples[(max - min + 2) as usize], 1010);
    assert_eq!(rescaled.samples[(max - min - 2) as usize], 990);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);

    let rescaled = d.rescale(0.45);
    assert_eq!(rescaled.samples[(max - min) as usize], 3000);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);

    let rescaled = d.rescale(1000000000.0);
    assert_eq!(rescaled.samples[(max - min) as usize], 1000);
    assert_eq!(rescaled.samples[0], 990);
    assert_eq!(rescaled.samples.last().unwrap(), &1010);
    assert_eq!(rescaled.samples.iter().sum::<usize>(), 3000);
}
