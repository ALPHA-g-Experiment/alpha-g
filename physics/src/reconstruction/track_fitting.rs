use crate::reconstruction::{
    angle_between_vectors, Cluster, Coordinate, Helix, Track, TryTrackFromClusterError,
};
use crate::SpacePoint;
use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::neldermead::NelderMead;
use num_complex::Complex;
use std::f64::consts::PI;
use uom::si::angle::radian;
use uom::si::area::square_meter;
use uom::si::f64::{Angle, Area, Length};
use uom::si::length::meter;
use uom::typenum::P2;

// To first order, the full track from the vertex to outside of the rTPC gas
// volume is a helix with axis parallel to the z-axis.
// Minimize the orthogonal distance between the track and the SpacePoints.
pub(crate) fn fit_cluster_to_helix(
    cluster: Cluster,
    max_num_solver_iter: u64,
    // Nelder-Mead stops whenever the standard deviation between the cost at all
    // simplex vertices is below this threshold.
    nelder_mead_sd_tolerance: f64,
    // Just following scipy's way of defining the simplex vertices given an
    // initial guess.
    // Initial simplex is defined by changing each coordinate of the initial
    // guess by this percentage.
    initial_simplex_delta: f64,
    // See Helix::closest_t for details on these 2 parameters.
    max_num_closest_t_iter: usize,
    closest_t_tolerance: f64,
) -> Result<Track, TryTrackFromClusterError> {
    let sp = cluster.0;
    // This assert is here just to make sure we don't accidentally change the
    // minimum number of points required in a cluster.
    assert!(sp.len() >= 3);
    // Three points are enough to get a reasonable first guess for the helix
    // parameters.
    let (first, middle, last) = three_template_points(&sp)?;

    let (x0, y0, r) = circle_through_three_points(
        (first.x(), first.y()),
        (middle.x(), middle.y()),
        (last.x(), last.y()),
    );

    let cm = center_of_mass(&sp);
    let phi0 = (cm.y - y0).atan2(cm.x - x0);
    let z0 = cm.z;

    let theta = angle_between_vectors(
        (last.x() - x0, last.y() - y0),
        (first.x() - x0, first.y() - y0),
    );
    // The product of the sign of theta and sign of the change in height by
    // rotating from outer to inner gives you the correct sign of h.
    // Getting this sign right is tricky (and very important for the minimizer
    // to have a good starting point).
    // Check this thoroughly if you change any of the code for the initial
    // guess.
    let h = 2.0 * PI * (first.z - last.z) / theta;
    // Argmin needs the same type for all parameters. Work with f64; there is no
    // risk of messing up the units inside the minimizer.
    // BUT it has to be in `METER` and `RADIAN` because that is what the
    // `cost_function` expects the `f64`s to represent.
    let initial_guess = vec![
        x0.get::<meter>(),
        y0.get::<meter>(),
        z0.get::<meter>(),
        r.get::<meter>(),
        phi0.get::<radian>(),
        h.get::<meter>(),
    ];
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
        points: sp,
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
    // Again, remember that the f64s in the `cost_function` are in `METER` and
    // `RADIAN`.
    let helix = Helix {
        x0: Length::new::<meter>(best_params[0]),
        y0: Length::new::<meter>(best_params[1]),
        z0: Length::new::<meter>(best_params[2]),
        r: Length::new::<meter>(best_params[3]),
        phi0: Angle::new::<radian>(best_params[4]),
        h: Length::new::<meter>(best_params[5]),
    };
    Ok(Track {
        helix,
        t_inner: helix.closest_t(first, closest_t_tolerance, max_num_closest_t_iter),
        t_outer: helix.closest_t(last, closest_t_tolerance, max_num_closest_t_iter),
    })
}

// With 3 spread out points, we can get a decent first guess on the helix
// parameters.
fn three_template_points(
    points: &[SpacePoint],
    // In theory, we would expect our tracks to originate from (near) the origin
    // and travel outwards.
    // Sorting by `r` feels like a natural ordering.
    // Return the:
    // (Smallest r, Middle r, Largest r)
) -> Result<(SpacePoint, SpacePoint, SpacePoint), TryTrackFromClusterError> {
    let first = points
        .iter()
        .min_by(|a, b| a.r.partial_cmp(&b.r).unwrap())
        .copied()
        .unwrap();
    let last = points
        .iter()
        .max_by(|a, b| a.r.partial_cmp(&b.r).unwrap())
        .copied()
        .unwrap();

    let middle_r = (first.r + last.r) / 2.0;
    let middle = points
        .iter()
        .min_by(|a, b| {
            (a.r - middle_r)
                .abs()
                .partial_cmp(&(b.r - middle_r).abs())
                .unwrap()
        })
        .copied()
        .unwrap();

    // If the 3 points are collinear, then there is no circle containing the
    // three points( with finite radius).
    // Also, if any pair of points are the same, then there is no circle
    // because we effectively have only 2 points.
    // There are 3 possible comparisons to make between slopes. I just did this
    // one because it exactly matches a fail mode of
    // `circle_through_three_points`.
    // Any other comparison (or using this as a test of collinearity to e.g.
    // resample the points) would require some epsilon distance difference
    // instead of exact equality (i.e. the usual way of comparing floats).
    if (last.x() - first.x()) * (middle.y() - first.y())
        == (middle.x() - first.x()) * (last.y() - first.y())
    {
        return Err(TryTrackFromClusterError::NoInitialParameters);
    }

    Ok((first, middle, last))
}

// Return the center and radius of the circle that goes through three points.
// Solution from:
// https://math.stackexchange.com/a/3503338/485443
fn circle_through_three_points(
    // Input tuples are (x, y)
    p1: (Length, Length),
    p2: (Length, Length),
    p3: (Length, Length),
    // Output tuple is (x, y, r)
) -> (Length, Length, Length) {
    // Cannot use Length in Complex because it is not Num.
    // Just use f64 (same units for all) and convert back at the end.
    let z1 = Complex::new(p1.0.get::<meter>(), p1.1.get::<meter>());
    let z2 = Complex::new(p2.0.get::<meter>(), p2.1.get::<meter>());
    let z3 = Complex::new(p3.0.get::<meter>(), p3.1.get::<meter>());

    let w = (z3 - z1) / (z2 - z1);
    let c_prime = (w - w.norm_sqr()) / (w - w.conj());

    let c = (z2 - z1) * c_prime + z1;
    let r = (z1 - c).norm();

    (
        Length::new::<meter>(c.re),
        Length::new::<meter>(c.im),
        Length::new::<meter>(r),
    )
}

// Calculate the center of mass of a set of SpacePoints.
fn center_of_mass(points: &[SpacePoint]) -> Coordinate {
    let mut x = Length::new::<meter>(0.0);
    let mut y = Length::new::<meter>(0.0);
    let mut z = Length::new::<meter>(0.0);

    for p in points {
        x += p.x();
        y += p.y();
        z += p.z;
    }

    let n = points.len() as f64;
    Coordinate {
        x: x / n,
        y: y / n,
        z: z / n,
    }
}

// The actual minimization problem is to find the helix that minimizes the
// sum of squared distances to all points.
struct Problem {
    points: Vec<SpacePoint>,
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
// Argmin parameters need to be a single type, so we just use the internal
// f64 representation of the helix parameters.
// This interface is the only place where we could make a mistake in the units.
// The f64s in the `cost_function` are in `METER` and `RADIAN`.
impl CostFunction for Problem {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let helix = Helix {
            x0: Length::new::<meter>(p[0]),
            y0: Length::new::<meter>(p[1]),
            z0: Length::new::<meter>(p[2]),
            r: Length::new::<meter>(p[3]),
            phi0: Angle::new::<radian>(p[4]),
            h: Length::new::<meter>(p[5]),
        };

        Ok(self
            .points
            .iter()
            .map(|&p| {
                let t = helix.closest_t(p, self.tolerance, self.max_num_iter);
                let closest_point = helix.at(t);

                let val = norm_sqr(p, closest_point);
                // Argmin needs non-NaN values to work properly.
                // If we got NaN at this point, there is a bug somewhere that
                // needs to be fixed.
                assert!(!val.is_nan(), "found NaN in track_fitting::cost_function");

                val
            })
            .sum::<Area>()
            .get::<square_meter>())
    }
}
