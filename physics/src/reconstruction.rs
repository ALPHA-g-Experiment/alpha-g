use crate::SpacePoint;
use core::slice::Iter;
use uom::si::f64::Length;
use uom::si::length::centimeter;

// Identify groups of SpacePoints that belong together to potential tracks.
mod track_finding;

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
        1,
        // Number of bins along `rho` in Hough space.
        200,
        // Number of bins along `theta` in Hough space.
        200,
        // Maximum clustering distance in Euclidean space.
        Length::new::<centimeter>(1.0),
    )
}

#[cfg(test)]
mod tests;
