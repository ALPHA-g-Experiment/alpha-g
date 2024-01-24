use super::*;
use alpha_g_detector::alpha16::aw_map::{ANODE_WIRES_RADIUS, INNER_CATHODE_RADIUS};
use alpha_g_detector::padwing::map::DETECTOR_LENGTH;
use uom::si::angle::radian;
use uom::si::length::meter;
use uom::si::time::microsecond;

#[test]
fn ascending_time_drift_tables() {
    for (table, _) in DRIFT_TABLES.0.iter() {
        let mut previous_time = None;
        for (time, _, _) in table.0.iter() {
            if let Some(prev_time) = previous_time {
                assert!(time > prev_time);
                previous_time = Some(time);
            } else {
                assert_eq!(time, &Time::new::<microsecond>(0.0));
                previous_time = Some(time);
            }
        }
    }
}

#[test]
fn ascending_lorentz_angle_drift_tables() {
    for (table, _) in DRIFT_TABLES.0.iter() {
        let mut previous_angle = None;
        for (_, _, angle) in table.0.iter() {
            if let Some(prev_angle) = previous_angle {
                assert!(angle >= prev_angle);
                previous_angle = Some(angle);
            } else {
                assert_eq!(angle, &Angle::new::<radian>(0.0));
                previous_angle = Some(angle);
            }
        }
    }
}

#[test]
fn descending_radius_drift_tables() {
    for (table, _) in DRIFT_TABLES.0.iter() {
        let mut previous_radius = None;
        for (_, radius, _) in table.0.iter() {
            if let Some(prev_radius) = previous_radius {
                assert!(radius < prev_radius);
                previous_radius = Some(radius);
            } else {
                let wire_radius = Length::new::<meter>(ANODE_WIRES_RADIUS);
                let diff = (wire_radius - *radius).abs();
                assert!(diff < Length::new::<meter>(5e-4));
                previous_radius = Some(radius);
            }
        }

        let inner_cathode_radius = Length::new::<meter>(INNER_CATHODE_RADIUS);
        let smallest_radius = previous_radius.unwrap();
        let diff = (inner_cathode_radius - *smallest_radius).abs();
        assert!(diff < Length::new::<meter>(5e-4));
    }
}

#[test]
fn all_positive_drift_tables() {
    for (table, upper_bound) in DRIFT_TABLES.0.iter() {
        assert!(upper_bound.is_sign_positive());

        for (time, radius, angle) in table.0.iter() {
            assert!(time.is_sign_positive());
            assert!(radius.is_sign_positive());
            assert!(angle.is_sign_positive());
        }
    }
}

#[test]
fn enough_to_interpolate_drift_tables() {
    for (table, _) in DRIFT_TABLES.0.iter() {
        assert!(table.0.len() >= 2);
    }
}

#[test]
fn ascending_upper_bound_drift_tables() {
    let mut previous_upper_bound = None;
    for (_, upper_bound) in DRIFT_TABLES.0.iter() {
        if let Some(prev_upper_bound) = previous_upper_bound {
            assert!(upper_bound > prev_upper_bound);
            previous_upper_bound = Some(upper_bound);
        } else {
            assert_eq!(upper_bound, &Length::new::<meter>(0.6975));
            previous_upper_bound = Some(upper_bound);
        }
    }
    assert_eq!(
        previous_upper_bound,
        Some(&Length::new::<meter>(DETECTOR_LENGTH / 2.0))
    );
}

#[test]
fn valid_drift_time_lookup() {
    let mut z = Length::new::<meter>(0.0);
    while z <= Length::new::<meter>(DETECTOR_LENGTH / 2.0) {
        let z_neg = -z;

        let mut t = Time::new::<microsecond>(0.0);
        while t <= Time::new::<microsecond>(3.5) {
            let (radius_pos, correction_pos) = DRIFT_TABLES.at(z, t).unwrap();
            let (radius_neg, correction_neg) = DRIFT_TABLES.at(z_neg, t).unwrap();

            assert_eq!(radius_pos, radius_neg);
            assert_eq!(correction_pos, correction_neg);

            t += Time::new::<microsecond>(0.5);
        }
        z += Length::new::<meter>(0.1);
    }
}
