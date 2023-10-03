use crate::SpacePoint;
use core::slice::Iter;
use std::f64::consts::PI;
use thiserror::Error;
use uom::si::angle::radian;
use uom::si::f64::{Angle, Length, Ratio};
use uom::si::length::centimeter;
use uom::si::ratio::ratio;
use uom::typenum::P2;

// Identify groups of SpacePoints that belong together to potential tracks.
mod track_finding;
// Fit a group of SpacePoints to a track.
mod track_fitting;
// Fit Tracks from an event to vertices.
mod vertex_fitting;

/// Collection of [`SpacePoint`]s.
///
/// A [`Cluster`] represents a group of [`SpacePoint`]s that are potentially
/// part of the same track.
#[derive(Clone, Debug)]
pub struct Cluster(Vec<SpacePoint>);

impl Cluster {
    /// Return an iterator over the [`SpacePoint`]s.
    pub fn iter(&self) -> Iter<'_, SpacePoint> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Cluster {
    type Item = &'a SpacePoint;
    type IntoIter = Iter<'a, SpacePoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Cluster {
    type Item = SpacePoint;
    type IntoIter = std::vec::IntoIter<SpacePoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Result of clustering [`SpacePoint`]s.
#[derive(Clone, Debug)]
pub struct ClusteringResult {
    /// Each [`Cluster`] represents a group of [`SpacePoint`]s that are
    /// potentially part of the same track.
    pub clusters: Vec<Cluster>,
    /// Remaining [`SpacePoint`]s that have not been identified as part of a
    /// [`Cluster`].
    pub remainder: Vec<SpacePoint>,
}

/// Given a collection of [`SpacePoint`]s, cluster them into groups that
/// are potentially part of the same track.
pub fn cluster_spacepoints(sp: Vec<SpacePoint>) -> ClusteringResult {
    track_finding::cluster_spacepoints(
        sp,
        // Maximum number of Clusters.
        2,
        // Minimum number of SpacePoints per Cluster.
        // We need at least 3 points to get an accurate initial guess for the
        // helix through a cluster.
        // Track fitting will panic if this is set to less than 3.
        3,
        // Number of bins along `rho` in Hough space.
        200,
        // Number of bins along `theta` in Hough space.
        200,
        // Maximum clustering distance in Euclidean space.
        Length::new::<centimeter>(1.0),
    )
}

/// A point in 3D space.
#[derive(Clone, Copy, Debug)]
pub struct Coordinate {
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

// To characterise a helix we need only 5 parameters. Nonetheless, I am
// using 6 parameters here because it makes it easier to constraint the
// helix to be a single revolution (otherwise the minimizer will tend
// towards helices with a very short `h` trivially going through all
// SpacePoints).
// The z0 and phi0 are "redundant" (you only really need one of these two) if
// you consider an infinite helix, but when we want to limit the helix to a
// single revolution, it is useful to have both (with phi0 pointing towards
// the center of mass of the SpacePoints).
// Then, the parametric equation of our helix is:
//     x = r * cos(t + phi0) + x0
//     y = r * sin(t + phi0) + y0
//     z = (h / 2pi) * t + z0
//
// Where t in [-pi, pi] gives you a single revolution.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Helix {
    x0: Length,
    y0: Length,
    z0: Length,
    r: Length,
    phi0: Angle,
    h: Length,
}

// Return the (signed) angle from v1 to v2.
// i.e. a positive angle means that we need to rotate v1 counter-clockwise to
// get to v2.
// Solution from:
// https://stackoverflow.com/a/16544330/8877655
fn angle_between_vectors(v1: (Length, Length), v2: (Length, Length)) -> Angle {
    let dot = v1.0 * v2.0 + v1.1 * v2.1;
    let det = v1.0 * v2.1 - v1.1 * v2.0;
    // If the implementation changes make sure that the returned angle is still
    // in [-pi, pi].
    det.atan2(dot)
}

impl Helix {
    fn at(&self, t: f64) -> Coordinate {
        let t = Angle::new::<radian>(t);

        Coordinate {
            x: self.r * (t + self.phi0).cos() + self.x0,
            y: self.r * (t + self.phi0).sin() + self.y0,
            z: (self.h / Angle::FULL_TURN) * t + self.z0,
        }
    }
    // Given a SpacePoint, return the value of t that corresponds to the
    // closest point on the helix.
    //
    // This implementation closely follows this paper:
    // https://www.sciencedirect.com/science/article/pii/S0168900208014836
    //
    // I only have a different relationship between E and t. For no reason in
    // particular other than that was what I first came up with when following
    // the paper's derivation.
    //
    // I solve Kepler's equation using Newton's method (which should converge
    // for any value of the eccentricity).
    // The first thing to try if you ever need this to be faster is to use e.g.
    // Markley's method for eccentricities less than 1 and Halleys's method for
    // eccentricities larger than 1.
    // Just re-verify the convergence properties of these methods.
    #[allow(non_snake_case)]
    fn closest_t(&self, p: SpacePoint, tolerance: f64, max_num_iter: usize) -> f64 {
        // If h is zero (i.e. a circle), the following method produces NaNs
        // (which are bad because they will propagate to the minimizer).
        // Handle the circle case separately.
        if self.h == Length::new::<centimeter>(0.0) {
            let c = self.at(0.0);
            return angle_between_vectors(
                (c.x - self.x0, c.y - self.y0),
                (p.x() - self.x0, p.y() - self.y0),
            )
            // No need to clamp because atan2 is already in [-pi, pi].
            .get::<radian>();
        }
        // Because E is linear in t, then a tolerance on E corresponds to a
        // tolerance on t.
        let tolerance = Angle::new::<radian>(tolerance.abs());
        // Basically just Algorithm 1 from page 3:
        let u = p.x();
        let v = p.y();
        let r = (u - self.x0).hypot(v - self.y0);
        let delta = (v - self.y0).atan2(u - self.x0);

        let temp = self.phi0 + Angle::from(2.0 * PI * (p.z - self.z0) / self.h) - delta;
        let n = (temp / Angle::FULL_TURN).floor::<ratio>();

        let M = Angle::HALF_TURN + Angle::from(2.0 * PI * n) - temp;
        let e = 4.0 * PI.powi(2) * r * self.r / self.h.powi(P2::new());
        // Need to solve for E in the equation:
        //    M = E - e * sin(E)
        // Newton method converges monotonically for any value of e.
        fn f(E: Angle, e: Ratio, M: Angle) -> Angle {
            E - Angle::from(e * E.sin()) - M
        }
        fn df(E: Angle, e: Ratio) -> Ratio {
            Ratio::new::<ratio>(1.0) - e * E.cos()
        }

        let mut E = if M < Angle::new::<radian>(0.0) {
            -Angle::HALF_TURN
        } else {
            Angle::HALF_TURN
        };
        for _ in 0..max_num_iter {
            E -= Angle::from(f(E, e, M) / df(E, e));

            if f(E, e, M).abs() < tolerance {
                break;
            }
        }

        let t = Angle::HALF_TURN - E + Angle::from(2.0 * PI * n) - self.phi0 + delta;
        // Our helix model is a single revolution i.e. t in [-pi, pi].
        // Clamping doesn't really give you the closest point when it is outside
        // the range (there is more likely other local minima), but it is good
        // enough to penalize the helix.
        // If t is within the range, then it is the actual global minimum.
        t.get::<radian>().clamp(-PI, PI)
    }
    // Return the coordinate of the closest point on the helix to the beamline.
    fn closest_to_beamline(&self) -> Coordinate {
        let c = self.at(0.0);
        let t = angle_between_vectors((c.x - self.x0, c.y - self.y0), (-self.x0, -self.y0))
            .get::<radian>();

        self.at(t)
    }
}

/// Trajectory of a charged particle through the detector volume.
///
/// The [`Coordinate`]s of a track are parametrized by a single variable `t`.
/// The values returned by [`Track::t_inner`] and [`Track::t_outer`] give an
/// approximate range of `t` for which the track is close to the inner and outer
/// cathodes of the rTPC respectively.
///
/// It is important to note that `t_inner` is not necessarily smaller than
/// `t_outer` (`t` is an arbitrary parametrization).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    // Don't expose the helix. It is just an internal implementation detail that
    // is bound to change at any time.
    helix: Helix,
    // These `t_inner` and `t_outer` are useful to "draw" the actual trajectory
    // of a particle through the detector volume. They tell us what is an
    // approximate range of `t`, and also in which direction it has to change if
    // we want to extrapolate the track in a particular direction (e.g. towards
    // the penning trap).
    // Note that `t_inner` can be greater than `t_outer`. It is not a `time`.
    // It is just an arbitrary parametrization.
    t_inner: f64,
    t_outer: f64,
}

impl Track {
    /// Return the [`Coordinate`] of the track at a given `t`.
    pub fn at(&self, t: f64) -> Coordinate {
        self.helix.at(t)
    }
    /// Return a value of `t` for which the track is close to the inner cathode
    /// of the detector.
    pub fn t_inner(&self) -> f64 {
        self.t_inner
    }
    /// Return a value of `t` for which the track is close to the outer cathode
    /// of the detector.
    pub fn t_outer(&self) -> f64 {
        self.t_outer
    }
}

/// The error type returned when conversion from a [`Cluster`] to a [`Track`]
/// fails.
#[derive(Debug, Error)]
pub enum TryTrackFromClusterError {
    /// Unable to produce initial fit parameters.
    #[error("unable to produce initial fit parameters")]
    NoInitialParameters,
}

impl TryFrom<Cluster> for Track {
    type Error = TryTrackFromClusterError;

    fn try_from(cluster: Cluster) -> Result<Self, Self::Error> {
        track_fitting::fit_cluster_to_helix(
            cluster,
            // Maximum number of Nelder-Mead iterations.
            100,
            // Nelder-Mead standard deviation tolerance.
            f64::EPSILON,
            // Delta from the initial guess for each simplex vertex.
            // I just stuck to the default value used by scipy's implementation
            // of Nelder-Mead. It has worked well.
            // See:
            // https://github.com/scipy/scipy/blob/v1.11.2/scipy/optimize/_optimize.py#L833
            0.05,
            // Maximum number of iterations to find the closest point on the
            // helix given a SpacePoint.
            20,
            // Tolerance for finding the `t` parameter of the closest point on
            // the helix given a SpacePoint.
            f64::EPSILON,
        )
    }
}

/// Information about a reconstructed vertex.
#[derive(Clone, Debug)]
pub struct VertexInfo {
    /// Position of the vertex.
    pub position: Coordinate,
    /// [`Track`]s associated to the vertex. Each track is paired with the value
    /// of `t` at which it is closest to the vertex.
    pub tracks: Vec<(Track, f64)>,
}

/// Result of reconstructing the vertices of an event from a set of [`Track`]s.
#[derive(Clone, Debug)]
pub struct VertexingResult {
    /// Primary signal vertex.
    pub primary: Option<VertexInfo>,
    /// Secondary vertices.
    pub secondaries: Vec<VertexInfo>,
    /// Remaining [`Track`]s that were not associated to any vertex.
    pub remainder: Vec<Track>,
}

/// Given a collection of [`Track`]s, reconstruct the vertices of an event.
pub fn fit_vertices(tracks: Vec<Track>) -> VertexingResult {
    vertex_fitting::fit_vertices(
        tracks,
        // Maximum distance of closest approach to the beamline to be considered
        // for primary vertex seed.
        Length::new::<centimeter>(5.0),
        // Maximum clustering distance along the beamline for primary vertex
        // cluster.
        Length::new::<centimeter>(5.0),
        // Delta from the initial guess for each initial simplex vertex.
        // I just stuck to the default value used by scipy's implementation
        // of Nelder-Mead. It has worked well.
        // See:
        // https://github.com/scipy/scipy/blob/v1.11.2/scipy/optimize/_optimize.py#L833
        0.05,
        // Maximum number of iterations to find the closest point on the
        // helix given a SpacePoint.
        20,
        // Tolerance for finding the `t` parameter of the closest point on
        // the helix given a SpacePoint.
        f64::EPSILON,
        // Maximum number of Nelder-Mead iterations per vertex distance
        // minimization.
        100,
        // Nelder-Mead standard deviation tolerance.
        f64::EPSILON,
    )
}

#[cfg(test)]
mod tests;
