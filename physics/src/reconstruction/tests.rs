use super::*;
use alpha_g_detector::padwing::map::{CATHODE_PADS_RADIUS, DETECTOR_LENGTH};
use uom::si::f64::Angle;
use uom::si::length::meter;

fn is_within_tpc_volume(p: &SpacePoint) -> bool {
    let detector_half_length = Length::new::<meter>(DETECTOR_LENGTH / 2.0);
    let outer_radius = Length::new::<meter>(CATHODE_PADS_RADIUS);
    let inner_radius = Length::new::<centimeter>(10.92);

    p.z.abs() < detector_half_length && p.r < outer_radius && p.r > inner_radius
}

#[test]
fn single_trivial_track_finding() {
    let mut raw_points = Vec::new();

    let r = Length::new::<centimeter>(20.0);
    let num_points = 1000;
    for i in 0..num_points {
        let theta = Angle::HALF_TURN * i as f64 / num_points as f64;
        let x = r * theta.cos() + r;
        let y = r * theta.sin();

        let point = SpacePoint {
            r: (x * x + y * y).sqrt(),
            phi: y.atan2(x),
            z: Length::new::<meter>(0.0),
        };

        if is_within_tpc_volume(&point) {
            raw_points.push(point);
        }
    }

    let clustering_result = cluster_spacepoints(raw_points.clone());

    assert!(clustering_result.remainder.is_empty());
    assert_eq!(clustering_result.clusters.len(), 1);

    let cluster = &clustering_result.clusters[0].0;
    assert_eq!(cluster.len(), raw_points.len());
    for point in cluster {
        assert!(raw_points.contains(point));
    }
}

#[test]
fn two_opposite_tracks() {
    let mut raw_points = Vec::new();

    let r = Length::new::<centimeter>(20.0);
    let num_points = 2000;
    for i in 0..num_points {
        let theta = Angle::FULL_TURN * i as f64 / num_points as f64;
        let x = r * theta.cos() + r;
        let y = r * theta.sin();

        let point = SpacePoint {
            r: (x * x + y * y).sqrt(),
            phi: y.atan2(x),
            z: Length::new::<meter>(0.0),
        };

        if is_within_tpc_volume(&point) {
            raw_points.push(point);
        }
    }

    let clustering_result = cluster_spacepoints(raw_points.clone());

    assert!(clustering_result.remainder.is_empty());
    assert_eq!(clustering_result.clusters.len(), 2);

    let cluster_0 = &clustering_result.clusters[0].0;
    let cluster_1 = &clustering_result.clusters[1].0;
    assert_eq!(cluster_0.len() + cluster_1.len(), raw_points.len());

    for (p0, p1) in cluster_0.iter().zip(cluster_1.iter()) {
        assert!(raw_points.contains(p0));
        assert!(raw_points.contains(p1));
    }
}

#[test]
fn two_on_top_tracks() {
    let mut raw_points = Vec::new();

    let r = Length::new::<centimeter>(20.0);
    let num_points = 1000;
    for i in 0..num_points {
        let theta = Angle::HALF_TURN * i as f64 / num_points as f64;
        let x = r * theta.cos() + r;
        let y = r * theta.sin();

        let point = SpacePoint {
            r: (x * x + y * y).sqrt(),
            phi: y.atan2(x),
            z: Length::new::<meter>(0.5),
        };
        if is_within_tpc_volume(&point) {
            raw_points.push(point);
        }

        let point = SpacePoint {
            r: (x * x + y * y).sqrt(),
            phi: y.atan2(x),
            z: Length::new::<meter>(-0.5),
        };
        if is_within_tpc_volume(&point) {
            raw_points.push(point);
        }
    }

    let clustering_result = cluster_spacepoints(raw_points.clone());

    assert!(clustering_result.remainder.is_empty());
    assert_eq!(clustering_result.clusters.len(), 2);

    let cluster_0 = &clustering_result.clusters[0].0;
    let cluster_1 = &clustering_result.clusters[1].0;
    assert_eq!(cluster_0.len() + cluster_1.len(), raw_points.len());

    for (p0, p1) in cluster_0.iter().zip(cluster_1.iter()) {
        assert!(raw_points.contains(p0));
        assert!(raw_points.contains(p1));
    }
}
