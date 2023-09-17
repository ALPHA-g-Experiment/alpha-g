use crate::SpacePoint;
use core::slice::Iter;
use thiserror::Error;
use uom::si::f64::Length;
use uom::si::length::centimeter;

// Identify groups of SpacePoints that belong together to potential tracks.
mod track_finding;
// Fit a group of SpacePoints to a track.
mod track_fitting;

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

/// Trajectory of a charged particle through the detector volume.
///
/// The [`Coordinate`]s of a track are parametrized by a single variable `t`.
/// The values returned by [`Track::t_inner`] and [`Track::t_outer`] give an
/// approximate range of `t` for which the track is close to the inner and outer
/// cathodes of the rTPC respectively.
///
/// It is important to note that `t_inner` is not necessarily smaller than
/// `t_outer` (`t` is an arbitrary parametrization).
#[derive(Clone, Copy, Debug)]
pub struct Track {
    helix: track_fitting::Helix,
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

#[cfg(test)]
mod tests;
