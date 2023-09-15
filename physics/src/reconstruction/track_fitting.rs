use crate::reconstruction::{Cluster, Coordinate, Track, TryTrackFromClusterError};
use crate::SpacePoint;
use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::neldermead::NelderMead;
use num_complex::Complex;
use std::f64::consts::PI;
use uom::si::angle::radian;
use uom::si::area::square_meter;
use uom::si::f64::{Angle, Area, Length, Ratio};
use uom::si::length::meter;
use uom::si::ratio::ratio;
use uom::typenum::P2;

// To first order, the full track from the vertex to outside of the rTPC gas
// volume is a helix with axis parallel to the z-axis.
// Minimize the orthogonal distance between the track and the SpacePoints.
pub(crate) fn fit_cluster_to_helix(
    mut cluster: Cluster,
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
    // In theory, we would expect our tracks to originate from (near) the origin
    // and travel outwards.
    // Sort the SpacePoints by radius to get them in their "natural" order.
    cluster
        .0
        .sort_unstable_by(|a, b| a.r.partial_cmp(&b.r).unwrap());
    let sp = cluster.0;
    // The first, middle, and last SpacePoints are enough to give us a good
    // initial guess for the helix parameters.
    // This assert is here just to make sure we don't accidentally change the
    // minimum number of points required in a cluster.
    assert!(sp.len() >= 3);
    let first = sp[0];
    let last = sp[sp.len() - 1];
    let middle = sp[sp.len() / 2];

    let (x0, y0, r) = circle_through_three_points(
        (first.r * first.phi.cos(), first.r * first.phi.sin()),
        (middle.r * middle.phi.cos(), middle.r * middle.phi.sin()),
        (last.r * last.phi.cos(), last.r * last.phi.sin()),
    );

    let cm = center_of_mass(&sp);
    let phi0 = (cm.y - y0).atan2(cm.x - x0);
    let z0 = cm.z;

    let theta = angle_between_vectors(
        (last.r * last.phi.cos() - x0, last.r * last.phi.sin() - y0),
        (
            first.r * first.phi.cos() - x0,
            first.r * first.phi.sin() - y0,
        ),
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
        new_point[i] *= 1.0 + initial_simplex_delta;
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
#[derive(Clone, Copy, Debug)]
pub(crate) struct Helix {
    x0: Length,
    y0: Length,
    z0: Length,
    r: Length,
    phi0: Angle,
    h: Length,
}

impl Helix {
    pub(crate) fn at(&self, t: f64) -> Coordinate {
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
        // Because E is linear in t, then a tolerance on E corresponds to a
        // tolerance on t.
        let tolerance = Angle::new::<radian>(tolerance.abs());
        // Basically just Algorithm 1 from page 3:
        let u = p.r * p.phi.cos();
        let v = p.r * p.phi.sin();
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
        x += p.r * p.phi.cos();
        y += p.r * p.phi.sin();
        z += p.z;
    }

    let n = points.len() as f64;
    Coordinate {
        x: x / n,
        y: y / n,
        z: z / n,
    }
}

// Return the (signed) angle from v1 to v2.
// Solution from:
// https://stackoverflow.com/a/16544330/8877655
fn angle_between_vectors(v1: (Length, Length), v2: (Length, Length)) -> Angle {
    let dot = v1.0 * v2.0 + v1.1 * v2.1;
    let det = v1.0 * v2.1 - v1.1 * v2.0;
    det.atan2(dot)
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
    let x = c.x - sp.r * sp.phi.cos();
    let y = c.y - sp.r * sp.phi.sin();
    let z = c.z - sp.z;

    x.powi(P2::new()) + y.powi(P2::new()) + z.powi(P2::new())
}

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

                norm_sqr(p, closest_point)
            })
            .sum::<Area>()
            .get::<square_meter>())
    }
}
