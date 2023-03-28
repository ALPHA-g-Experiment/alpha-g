use super::*;

#[test]
fn test_t_statistic() {
    let old_calib = (19.27, 1.3624843619, 11);
    let new_calib = (23.69, 2.5302781848, 13);

    let t = t_statistic(old_calib, new_calib, 0.0);
    let diff = (t + 1.538).abs();
    assert!(diff < 0.0001);
}

#[test]
fn test_dof() {
    let old_calib = (19.27, 1.3624843619, 11);
    let new_calib = (23.69, 2.5302781848, 13);

    let dof = dof(old_calib, new_calib);
    let diff = (dof - 18.1378).abs();
    assert!(diff < 0.0001);
}

#[test]
fn test_p_value() {
    let old_calib = (19.27, 1.3624843619, 11);
    let new_calib = (23.69, 2.5302781848, 13);

    let t = t_statistic(old_calib, new_calib, 0.0);

    let p = p_value(t, t, 18.0);
    let diff = (p - 0.9292836703899112).abs();
    assert!(diff < 0.0001);
}

#[test]
fn equivalence_test() {
    let old_calib = (19.27, 1.3624843619, 11);
    let new_calib = (23.69, 2.5302781848, 13);

    let p = tost(old_calib, new_calib, 10.0);
    let diff = (p - 0.033936176335811474).abs();
    assert!(diff < 0.0001);
}
