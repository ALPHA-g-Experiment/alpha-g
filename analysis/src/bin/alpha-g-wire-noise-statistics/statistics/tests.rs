use super::*;

#[test]
fn i16_mean() {
    let slice = [];
    assert!(mean(&slice).is_nan());

    let slice = [1];
    assert_eq!(mean(&slice), 1.0);

    let slice = [1, 2, 3, 4, 5];
    assert_eq!(mean(&slice), 3.0);

    let slice = [-1, -2, -3, -4, -5];
    assert_eq!(mean(&slice), -3.0);

    let slice = [-1, 0, 1];
    assert_eq!(mean(&slice), 0.0);
}

#[test]
fn i16_cov() {
    let slice1 = [1, 2, 3, 4, 5];
    let slice2 = [1, 1, 1, 1, 1];
    let covariance = cov(&slice1, &slice2);
    assert!((covariance - 0.0).abs() < 0.0001);

    let slice1 = [2, 2, 3, 4];
    let slice2 = [8, 10, 12, 14];
    let covariance = cov(&slice1, &slice2);
    assert!((covariance - 2.333333333).abs() < 0.0001);

    let slice1 = [2, 2, 3, 4, 0, 0, 0, 0, 0, 0];
    let slice2 = [8, 10, 12, 14];
    let covariance = cov(&slice1, &slice2);
    assert!((covariance - 2.333333333).abs() < 0.0001);

    let slice1 = [2, 2, 3, 4];
    let slice2 = [8, 10, 12, 14, 0, 0, 0, 0, 0, 0];
    let covariance = cov(&slice1, &slice2);
    assert!((covariance - 2.333333333).abs() < 0.0001);
}

#[test]
fn i16_std_dev() {
    let slice = [1];
    assert!(std_dev(&slice).is_nan());

    let slice = [1, 1];
    assert_eq!(std_dev(&slice), 0.0);

    let slice = [0, 4];
    let std = std_dev(&slice);
    assert!((std - 2.8284271247).abs() < 0.0001);

    let slice = [1, 2, 3, 4, 5];
    let std = std_dev(&slice);
    assert!((std - 1.5811388301).abs() < 0.0001);
}
