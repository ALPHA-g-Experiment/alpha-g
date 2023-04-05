use super::*;

#[test]
fn minimization_try_minimization() {
    let mut d1 = Distribution::new();
    for sample in -10000..0 {
        d1.add_sample(sample, sample.abs() as usize);
    }
    for sample in 0..10000 {
        d1.add_sample(sample, sample as usize);
    }

    let d2 = d1.clone().rescale(0.5);
    let best_param = try_minimization(&d1, &d2, -20000, 20000, 0, 1.0)
        .unwrap()
        .best_param
        .unwrap();
    let diff = (best_param - 1.0 / 0.5).abs();
    assert!(diff < 0.0001);

    let d2 = d1.clone().rescale(2.0);
    let best_param = try_minimization(&d1, &d2, -20000, 20000, 0, 1.0)
        .unwrap()
        .best_param
        .unwrap();
    let diff = (best_param - 1.0 / 2.0).abs();
    assert!(diff < 0.0001);
}
