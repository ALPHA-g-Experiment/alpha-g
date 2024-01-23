use crate::reconstruction::{Cluster, ClusteringResult};
use crate::SpacePoint;
use alpha_g_detector::alpha16::aw_map::INNER_CATHODE_RADIUS;
use itertools::Itertools;
use statrs::statistics::Statistics;
use std::cmp::Ordering;
use std::collections::HashMap;
use uom::si::f64::{Angle, Length, ReciprocalLength};
use uom::si::length::meter;
use uom::si::ratio::ratio;
use uom::typenum::P2;

// A track, as seen from the x-y plane, will form a circle.
//
// In the x-y plane, the conformal transformation:
// u = x / (x^2 + y^2)
// v = y / (x^2 + y^2)
// maps a circle (and a line) that goes through the origin into a straight line.
// Similarly, it maps circles (and lines) that do not go through the origin into
// circles.
//
// We can filter potential annihilation tracks (which originate close to the
// origin) by finding straight lines in the u-v plane.
pub(crate) fn cluster_spacepoints(
    mut sp: Vec<SpacePoint>,
    min_num_points_per_cluster: usize,
    rho_bins: u32,
    theta_bins: u32,
    max_distance: Length,
) -> ClusteringResult {
    let mut accumulator = HoughSpaceAccumulator {
        rho_bins,
        theta_bins,
        accumulator: HashMap::new(),
    };
    for &point in sp.iter() {
        accumulator.add(point);
    }
    // Given an accumulator in a particular state, identify the best cluster of
    // SpacePoints i.e. largest number of points that form a line in Hough space
    // and are close enough to be a single track.
    // Leave the accumulator in a state where the corresponding points have been
    // removed.
    fn best_cluster(
        accumulator: &mut HoughSpaceAccumulator,
        max_distance: Length,
    ) -> Vec<SpacePoint> {
        let mut prev_best = Vec::new();

        loop {
            let best = largest_cluster(accumulator.most_popular(), max_distance);
            if best.len() <= prev_best.len() {
                break;
            }

            for &point in best.iter() {
                accumulator.remove_unchecked(point);
            }
            for &point in prev_best.iter() {
                accumulator.add(point);
            }

            prev_best = best;
        }

        prev_best
    }

    let mut clusters = Vec::new();
    loop {
        let cluster = best_cluster(&mut accumulator, max_distance);
        if cluster.len() < min_num_points_per_cluster {
            break;
        }

        clusters.push(Cluster(cluster));
    }
    // The remainder is the set of points that were not clustered.
    for &point in clusters.iter().flatten() {
        // All points clustered are guaranteed to come from the original set of
        // SpacePoints; hence it is safe to unwrap.
        let index = sp.iter().position(|&p| p == point).unwrap();
        sp.swap_remove(index);
    }

    ClusteringResult {
        clusters,
        remainder: sp,
    }
}

// The maximum possible `rho` in Hough space is the maximum distance from the
// origin to any point in the u-v plane.
const RHO_MAX: ReciprocalLength = ReciprocalLength {
    dimension: uom::lib::marker::PhantomData,
    units: uom::lib::marker::PhantomData,
    value: 1.0 / INNER_CATHODE_RADIUS,
};

struct HoughSpaceAccumulator {
    rho_bins: u32,
    theta_bins: u32,
    // Simply counting the number of votes for each bin is not enough for our
    // purposes. Keep track explicitly of which SpacePoints have gone through
    // each bin in Hough space.
    // This makes it easier to remove all SpacePoints that contributed to e.g.
    // the most popular bin.
    // First index is theta, second index is rho.
    accumulator: HashMap<(u32, u32), Vec<SpacePoint>>,
}

// Conformal transformation from x-y plane to u-v plane.
fn u_v(point: SpacePoint) -> (ReciprocalLength, ReciprocalLength) {
    let u = point.x() / point.r.powi(P2::new());
    let v = point.y() / point.r.powi(P2::new());

    (u, v)
}

impl HoughSpaceAccumulator {
    // Given a SpacePoint, return all the bins in Hough space that it votes for.
    fn get_bins(&self, point: SpacePoint) -> Vec<(u32, u32)> {
        // Conformal mapping coordinates
        let (u, v) = u_v(point);

        let delta_theta = Angle::FULL_TURN / f64::from(self.theta_bins);
        let delta_rho = RHO_MAX / f64::from(self.rho_bins);

        let mut bins = Vec::new();
        // Hough space is parametrized as:
        // rho = u * cos(theta) + v * sin(theta)
        // The first bin has theta = 0
        let mut prev_rho_bin = (u / delta_rho).get::<ratio>().floor() as i32;
        for theta_bin in 1..=self.theta_bins {
            let theta = f64::from(theta_bin) * delta_theta;
            let (sin, cos) = theta.sin_cos();
            let rho = u * cos + v * sin;
            let rho_bin = (rho / delta_rho).get::<ratio>().floor() as i32;
            // If rho has only been negative between this and the previous
            // iteration, we don't want to vote for any bins.
            // Those bins are just duplicates of other bins with positive values
            // of rho and different theta.
            if !rho_bin.is_negative() || !prev_rho_bin.is_negative() {
                let min_bin = prev_rho_bin.min(rho_bin);
                let max_bin = prev_rho_bin.max(rho_bin);
                for bin in min_bin.max(0)..=max_bin {
                    bins.push((theta_bin - 1, bin.try_into().unwrap()));
                }
            }
            prev_rho_bin = rho_bin;
        }

        bins
    }
    // Add a SpacePoint to the accumulator.
    fn add(&mut self, point: SpacePoint) {
        for bin in self.get_bins(point) {
            self.accumulator.entry(bin).or_default().push(point);
        }
    }
    // Remove a SpacePoint from the accumulator.
    // Panic if the SpacePoint is not in the accumulator.
    fn remove_unchecked(&mut self, point: SpacePoint) {
        for bin in self.get_bins(point) {
            let vec = self.accumulator.get_mut(&bin).unwrap();
            let pos = vec.iter().position(|p| *p == point).unwrap();
            vec.swap_remove(pos);
        }
    }
    // Return the SpacePoints that voted for the most popular bin. Return an
    // empty vector if the accumulator is empty.
    fn most_popular(&self) -> Vec<SpacePoint> {
        // The order in which values of a HashMap are iterated over is random.
        // We need this function to be deterministic, hence we need some tie
        // breaking for the case where multiple bins have the same maximum
        // number of votes.
        self.accumulator
            .values()
            .max_set_by_key(|v| v.len())
            .into_iter()
            .max_by(|c1, c2| cluster_tie_breaker(c1, c2))
            .cloned()
            .unwrap_or_default()
    }
}

// Assign a deterministic ordering/priority of two clusters when they have
// the same number of points.
// The better cluster is `Ordering::Greater`.
fn cluster_tie_breaker(c1: &[SpacePoint], c2: &[SpacePoint]) -> Ordering {
    let v1 = c1.iter().map(|p| p.r.get::<meter>()).variance();
    let v2 = c2.iter().map(|p| p.r.get::<meter>()).variance();

    match v1.partial_cmp(&v2) {
        Some(Ordering::Less) => Ordering::Less,
        Some(Ordering::Greater) => Ordering::Greater,
        Some(Ordering::Equal) => {
            let v1 = c1.iter().map(|p| p.z.get::<meter>()).variance();
            let v2 = c2.iter().map(|p| p.z.get::<meter>()).variance();
            // Can't be NaN since they didn't have NaN variance in r.
            // Therefore, we can unwrap.
            v2.partial_cmp(&v1).unwrap()
        }
        // If the clusters are empty (or are a single point), then we don't
        // really care about the ordering. It can be random because these points
        // will never make it pass the `min_num_points_per_cluster` filter (we
        // need at least 3 for track fitting, etc).
        None => Ordering::Equal,
    }
}

// Given a collection of SpacePoints, find the largest subset of SpacePoints
// such that they all can be reached from each other by a path of SpacePoints
// that are within a certain distance.
//
// This is necessary after identifying lines in Hough space because of the
// following scenarios:
//   1. Two tracks that go in opposite directions will be picked up as one
//   single line in Hough space. These tracks will have a gap in the middle
//   (inner cathode of the rTPC).
//   2. Two tracks that go in the same direction but occur at different values
//   of z. They will be picked as the same track when seen from the x-y (u-v)
//   plane.
fn largest_cluster(mut points: Vec<SpacePoint>, max_distance: Length) -> Vec<SpacePoint> {
    let mut clusters: Vec<Vec<_>> = Vec::new();

    while let Some(point) = points.pop() {
        let mut cluster = vec![point];
        let mut i = 0;
        while i < cluster.len() {
            let mut j = 0;
            while j < points.len() {
                if cluster[i].distance(points[j]) <= max_distance {
                    cluster.push(points.swap_remove(j));
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        clusters.push(cluster);
    }

    clusters
        .into_iter()
        .max_set_by_key(|c| c.len())
        .into_iter()
        .max_by(|c1, c2| cluster_tie_breaker(c1, c2))
        .unwrap_or_default()
}
