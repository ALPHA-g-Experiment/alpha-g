use crate::reconstruction::{Coordinate, Track, VertexInfo, VertexingResult};
use crate::SpacePoint;
use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::neldermead::NelderMead;
use uom::si::area::square_meter;
use uom::si::f64::{Area, Length};
use uom::si::length::meter;
use uom::typenum::P2;

#[allow(clippy::too_many_arguments)]
pub(crate) fn fit_vertices(
    mut tracks: Vec<Track>,
    // If a track has a distance of closest approach to the beamline larger than
    // this value, remove it from the primary vertex seed.
    max_track_beamline_dca: Length,
    max_beamline_clustering_distance: Length,
    // Following scipy's way of defining initial simplex vertices from the
    // initial guess.
    initial_simplex_delta: f64,
    // See Helix::closest_t for details on these parameters.
    max_num_closest_t_iter: usize,
    closest_t_tolerance: f64,
    // Maximum number of Nelder-Mead iterations.
    max_num_solver_iter: u64,
    // Nelder-Mead stops whenever the standard deviation between the cost at all
    // simplex vertices is below this threshold.
    nelder_mead_sd_tolerance: f64,
) -> VertexingResult {
    let primary_tracks = tracks
        .iter()
        .filter(|track| {
            (track.helix.r - track.helix.x0.hypot(track.helix.y0)).abs() < max_track_beamline_dca
        })
        .copied()
        .collect();

    let vertex = beamline_clusters(primary_tracks, max_beamline_clustering_distance)
        .into_iter()
        .filter(|(cluster, _)| cluster.len() > 1)
        .max_by_key(|(cluster, _)| cluster.len())
        .map(|(tracks, mean_z)| {
            // Argmin needs all parameters to be same type. Work with internal
            // f64.
            // It has to be in `METER` because that is what the `cost_function`
            // expects internally.
            let initial_guess = vec![0.0, 0.0, mean_z.get::<meter>()];
            let mut initial_simplex = vec![initial_guess.clone()];
            for i in 0..initial_guess.len() {
                let mut new_point = initial_guess.clone();
                if new_point[i] == 0.0 {
                    // Default value from scipy's implementation.
                    // I don't think this is important enough to make it a parameter.
                    new_point[i] = 0.00025;
                } else {
                    new_point[i] *= 1.0 + initial_simplex_delta;
                }
                initial_simplex.push(new_point);
            }

            let problem = Problem {
                tracks: tracks.clone(),
                tolerance: closest_t_tolerance,
                max_num_iter: max_num_closest_t_iter,
            };
            let solver = NelderMead::new(initial_simplex)
                .with_sd_tolerance(nelder_mead_sd_tolerance)
                .unwrap();
            let res = Executor::new(problem, solver)
                .configure(|state| state.max_iters(max_num_solver_iter))
                .run()
                .unwrap();
            let best_params = res.state.best_param.unwrap();
            // Again, remember that the f64s in the `cost_function` are in
            // `METER`
            let position = Coordinate {
                x: Length::new::<meter>(best_params[0]),
                y: Length::new::<meter>(best_params[1]),
                z: Length::new::<meter>(best_params[2]),
            };

            VertexInfo {
                position,
                tracks: tracks
                    .into_iter()
                    .map(|track| {
                        // There is already a method in `Track` to calculate the
                        // closest_t to a SpacePoint. Just use that.
                        let sp = SpacePoint {
                            r: position.x.hypot(position.y),
                            phi: position.y.atan2(position.x),
                            z: position.z,
                        };

                        let t =
                            track
                                .helix
                                .closest_t(sp, closest_t_tolerance, max_num_closest_t_iter);

                        (track, t)
                    })
                    .collect(),
            }
        });
    // The remainder is the set of tracks that are not associated with the
    // primary vertex.
    for (track, _) in vertex.iter().flat_map(|v| v.tracks.iter()) {
        // All tracks in the primary vertex are guaranteed to come from the
        // original set of tracks.
        let index = tracks.iter().position(|t| t == track).unwrap();
        tracks.swap_remove(index);
    }

    VertexingResult {
        primary: vertex,
        secondaries: Vec::new(),
        remainder: tracks,
    }
}

// Cluster tracks by the `z` coordinate of their closest approach to the
// beamline.
fn beamline_clusters(
    mut tracks: Vec<Track>,
    max_beamline_clustering_distance: Length,
    // Return all clusters and the average `z` coordinate of the closest
    // approach to the beamline.
) -> Vec<(Vec<Track>, Length)> {
    if tracks.is_empty() {
        return Vec::new();
    }

    tracks.sort_unstable_by(|a, b| {
        a.helix
            .closest_to_beamline()
            .z
            .partial_cmp(&b.helix.closest_to_beamline().z)
            .unwrap()
    });

    let mut clusters = vec![vec![tracks[0]]];
    for track in tracks.into_iter().skip(1) {
        let current_z = track.helix.closest_to_beamline().z;
        let last_z = clusters
            .last()
            .unwrap()
            .last()
            .unwrap()
            .helix
            .closest_to_beamline()
            .z;

        if (current_z - last_z).abs() < max_beamline_clustering_distance {
            clusters.last_mut().unwrap().push(track);
        } else {
            clusters.push(vec![track]);
        }
    }

    clusters
        .into_iter()
        .map(|tracks| {
            let z = tracks
                .iter()
                .map(|track| track.helix.closest_to_beamline().z)
                .sum::<Length>()
                / tracks.len() as f64;
            (tracks, z)
        })
        .collect()
}

// The actual minimization problem is to find the coordinate that minimizes the
// sum of squared distances to all tracks.
struct Problem {
    tracks: Vec<Track>,
    // Parameters required to calculate the distance between a point and a helix.
    tolerance: f64,
    max_num_iter: usize,
}

// Calculate the squared distance between a SpacePoint and a Coordinate.
fn norm_sqr(sp: SpacePoint, c: Coordinate) -> Area {
    let x = c.x - sp.x();
    let y = c.y - sp.y();
    let z = c.z - sp.z;

    x.powi(P2::new()) + y.powi(P2::new()) + z.powi(P2::new())
}
// Argmin parameters need to be a single type, so we just use the internal f64
// representation of the coordinate.
// This interface is the only place where we could make a mistake in the units.
// The f64s are in METERS.
impl CostFunction for Problem {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let x = Length::new::<meter>(p[0]);
        let y = Length::new::<meter>(p[1]);
        let z = Length::new::<meter>(p[2]);
        // There is already a method in Track to calculate the closest point to
        // a SpacePoint. So just use that.
        let sp = SpacePoint {
            r: x.hypot(y),
            phi: y.atan2(x),
            z,
        };

        Ok(self
            .tracks
            .iter()
            .map(|track| {
                let t = track.helix.closest_t(sp, self.tolerance, self.max_num_iter);
                let closest_point = track.at(t);

                let val = norm_sqr(sp, closest_point);
                // Argmin needs non-NaN values to work properly.
                // If we got NaN at this point, there is a bug somewhere that
                // needs to be fixed.
                assert!(!val.is_nan(), "found NaN in vertex_fitting::cost_function");

                val
            })
            .sum::<Area>()
            .get::<square_meter>())
    }
}
