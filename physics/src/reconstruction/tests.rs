use super::*;
use alpha_g_detector::padwing::map::{CATHODE_PADS_RADIUS, DETECTOR_LENGTH};
use std::f64::consts::PI;
use uom::si::angle::radian;
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

fn trivial_helix_fit(x0: Length, y0: Length, z0: Length, r: Length, phi0: Angle, h: Length) {
    let mut raw_points = Vec::new();
    let num_points = 2000;
    for i in 0..num_points {
        let t = Angle::FULL_TURN * i as f64 / num_points as f64 - Angle::HALF_TURN;
        let coord = Coordinate {
            x: r * (t + phi0).cos() + x0,
            y: r * (t + phi0).sin() + y0,
            z: (h / Angle::FULL_TURN) * t + z0,
        };

        let point = SpacePoint {
            r: coord.x.hypot(coord.y),
            phi: coord.y.atan2(coord.x),
            z: coord.z,
        };

        if is_within_tpc_volume(&point) {
            raw_points.push(point);
        }
    }

    let clustering_result = cluster_spacepoints(raw_points);
    assert_eq!(clustering_result.clusters.len(), 2);

    for cluster in clustering_result.clusters {
        let mut points = cluster.0.clone();
        points.sort_unstable_by(|a, b| a.r.partial_cmp(&b.r).unwrap());

        let track = Track::try_from(cluster).unwrap();

        let inner = track.at(track.t_inner());
        let diff = (inner.x - points[0].r * points[0].phi.cos()).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));
        let diff = (inner.y - points[0].r * points[0].phi.sin()).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));
        let diff = (inner.z - points[0].z).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));

        let outer = track.at(track.t_outer());
        let diff =
            (outer.x - points[points.len() - 1].r * points[points.len() - 1].phi.cos()).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));
        let diff =
            (outer.y - points[points.len() - 1].r * points[points.len() - 1].phi.sin()).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));
        let diff = (outer.z - points[points.len() - 1].z).abs();
        assert!(diff < Length::new::<centimeter>(1e-6));
    }
}

#[test]
fn trivial_track_fitting() {
    // Helix center in first cuadrant
    trivial_helix_fit(
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(-3.0 * PI / 4.0),
        Length::new::<centimeter>(50.0),
    );
    trivial_helix_fit(
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(-3.0 * PI / 4.0),
        Length::new::<centimeter>(-50.0),
    );
    // Second cuadrant
    trivial_helix_fit(
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(-PI / 4.0),
        Length::new::<centimeter>(50.0),
    );
    trivial_helix_fit(
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(-PI / 4.0),
        Length::new::<centimeter>(-50.0),
    );
    // Third cuadrant
    trivial_helix_fit(
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(PI / 4.0),
        Length::new::<centimeter>(50.0),
    );
    trivial_helix_fit(
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(PI / 4.0),
        Length::new::<centimeter>(-50.0),
    );
    // Fourth cuadrant
    trivial_helix_fit(
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(3.0 * PI / 4.0),
        Length::new::<centimeter>(50.0),
    );
    trivial_helix_fit(
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(3.0 * PI / 4.0),
        Length::new::<centimeter>(-50.0),
    );
}

#[test]
fn track_fitting_h_equal_zero_regression() {
    trivial_helix_fit(
        Length::new::<centimeter>(20.0),
        Length::new::<centimeter>(-20.0),
        Length::new::<centimeter>(0.0),
        Length::new::<centimeter>(30.0),
        Angle::new::<radian>(3.0 * PI / 4.0),
        Length::new::<centimeter>(0.0),
    );
}

#[test]
fn track_fitting_bad_initial_parameters_regression_repeated_points() {
    let mut points = Vec::new();
    for _ in 0..100 {
        points.push(SpacePoint {
            r: Length::new::<centimeter>(15.0),
            phi: Angle::new::<radian>(0.0),
            z: Length::new::<centimeter>(0.0),
        });
    }

    let clustering_result = cluster_spacepoints(points);
    assert_eq!(clustering_result.clusters.len(), 1);

    let cluster = clustering_result.clusters[0].clone();
    assert!(Track::try_from(cluster).is_err());
}

#[test]
fn track_fitting_bad_initial_parameters_regression_collinear_points() {
    let mut points = Vec::new();
    for i in 0..100 {
        points.push(SpacePoint {
            r: Length::new::<centimeter>(10.0 + 0.09 * i as f64),
            phi: Angle::new::<radian>(0.0),
            z: Length::new::<centimeter>(0.0),
        });
    }

    let clustering_result = cluster_spacepoints(points);
    assert_eq!(clustering_result.clusters.len(), 1);

    let cluster = clustering_result.clusters[0].clone();
    assert!(Track::try_from(cluster).is_err());
}

fn test_trivial_vertex(z: Length) {
    let mut points = Vec::new();

    let r = Length::new::<centimeter>(20.0);
    let num_points = 2000;
    for i in 0..num_points {
        let theta = Angle::FULL_TURN * i as f64 / num_points as f64;
        let x = r * theta.cos() + r;
        let y = r * theta.sin();

        let point = SpacePoint {
            r: (x * x + y * y).sqrt(),
            phi: y.atan2(x),
            z,
        };

        if is_within_tpc_volume(&point) {
            points.push(point);
        }
    }

    let clusters = cluster_spacepoints(points).clusters;
    let tracks = clusters
        .into_iter()
        .map(|cluster| Track::try_from(cluster).unwrap())
        .collect();
    let vertex = fit_vertices(tracks).primary.unwrap().position;
    let diff_x = (vertex.x - Length::new::<meter>(0.0)).abs();
    let diff_y = (vertex.y - Length::new::<meter>(0.0)).abs();
    let diff_z = (vertex.z - z).abs();
    assert!(diff_x < Length::new::<meter>(1e-6));
    assert!(diff_y < Length::new::<meter>(1e-6));
    assert!(diff_z < Length::new::<meter>(1e-6));
}

#[test]
fn trivial_vertex_fitting() {
    test_trivial_vertex(Length::new::<meter>(0.0));
    test_trivial_vertex(Length::new::<meter>(0.5));
    test_trivial_vertex(Length::new::<meter>(1.0));
    test_trivial_vertex(Length::new::<meter>(-0.5));
    test_trivial_vertex(Length::new::<meter>(-1.0));
}
