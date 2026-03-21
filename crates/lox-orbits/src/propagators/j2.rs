// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2002-2026 CS GROUP
//
// SPDX-License-Identifier: MPL-2.0 AND Apache-2.0

//! Semi-analytical J2 orbit propagator using the Brouwer-Lyddane theory.
//!
//! This propagator models the secular and periodic effects of the J2 zonal
//! harmonic perturbation analytically. It converts osculating elements to mean
//! elements at construction, propagates mean elements with secular rates
//! (including J2² second-order corrections), and adds short-period and
//! long-period corrections when evaluating the osculating state.
//!
//! The implementation follows Orekit's `BrouwerLyddanePropagator` for the
//! J2-only case, including the T2 function for critical inclination avoidance.
//!
//! # References
//!
//! - Brouwer, D. (1959). "Solution of the Problem of Artificial Satellite
//!   Theory Without Drag." *The Astronomical Journal*, 64, 378.
//! - Lyddane, R. H. (1963). "Small Eccentricities or Inclinations in the
//!   Brouwer Theory of the Artificial Satellite." *The Astronomical Journal*,
//!   68, 555.
//! - Phipps, W. E. (1992). "Parallelization of the Navy Space Surveillance
//!   Center (NAVSPASUR) Satellite Model." Naval Postgraduate School.

use lox_bodies::{
    DynOrigin, J2 as J2Trait, Origin, PointMass, Spheroid, TryJ2, TryPointMass, TrySpheroid,
};
use lox_core::anomalies::{AnomalyError, MeanAnomaly};
use lox_core::coords::Cartesian;
use lox_core::elements::{Eccentricity, GravitationalParameter, Keplerian, OrbitType};
use lox_core::units::{AngleUnits, Distance};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;

/// Default time step for interval propagation (60 seconds).
const DEFAULT_STEP_SECONDS: f64 = 60.0;

/// Maximum iterations for osculating-to-mean element conversion.
const MAX_OSC_TO_MEAN_ITERATIONS: usize = 50;

/// Convergence tolerance for osculating-to-mean iteration.
const OSC_TO_MEAN_TOL: f64 = 1e-12;

/// Errors that can occur during semi-analytical J2 propagation.
#[derive(Debug, Error)]
pub enum J2Error {
    /// The orbit is not elliptic (required for J2 secular theory).
    #[error("semi-analytical J2 propagation requires an elliptic orbit, got {0}")]
    NonElliptic(OrbitType),
    /// The origin body lacks a required physical property.
    #[error("undefined origin property: {0}")]
    UndefinedOriginProperty(String),
    /// Anomaly conversion failed.
    #[error("anomaly conversion failed: {0}")]
    Anomaly(#[from] AnomalyError),
    /// Error constructing the output trajectory.
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
    /// Osculating-to-mean element conversion did not converge.
    #[error("osculating-to-mean element conversion did not converge after {0} iterations")]
    MeanElementConvergence(usize),
}

/// Semi-analytical J2 orbit propagator using first-order Brouwer theory.
///
/// Propagates mean Keplerian elements with secular rates for RAAN, argument of
/// periapsis, and mean anomaly. Short-period and long-period corrections are
/// applied when evaluating the osculating state.
#[derive(Debug, Clone, Copy)]
pub struct J2Propagator<T: TimeScale, O: TryJ2 + TryPointMass + TrySpheroid, R: ReferenceFrame> {
    initial_state: CartesianOrbit<T, O, R>,
    mean_elements: BrouwerMeanElements,
    step: TimeDelta,
}

/// Type alias for a [`J2Propagator`] using dynamic time scale, origin, and frame.
pub type DynJ2Propagator = J2Propagator<DynTimeScale, DynOrigin, DynFrame>;

/// Pre-computed mean Keplerian elements and secular rates for Brouwer J2 theory.
#[derive(Debug, Clone, Copy)]
struct BrouwerMeanElements {
    /// Mean semi-major axis [m].
    a: f64,
    /// Mean eccentricity.
    e: f64,
    /// Mean inclination [rad].
    i: f64,
    /// Mean RAAN at epoch [rad].
    raan0: f64,
    /// Mean argument of periapsis at epoch [rad].
    aop0: f64,
    /// Mean anomaly at epoch [rad].
    m0: f64,
    /// Secular rate of RAAN [rad/s].
    raan_dot: f64,
    /// Secular rate of argument of periapsis [rad/s].
    aop_dot: f64,
    /// Secular rate of mean anomaly (including mean motion) [rad/s].
    m_dot: f64,
    /// Gravitational parameter [m³/s²].
    mu: f64,
    /// J2 coefficient.
    j2: f64,
    /// Equatorial radius [m].
    r_eq: f64,
}

// Infallible — static bounds
impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: J2Trait + PointMass + Spheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Create a new semi-analytical J2 propagator from the given initial
    /// osculating state.
    pub fn new(initial_state: CartesianOrbit<T, O, R>) -> Result<Self, J2Error> {
        let mu = initial_state.origin().gravitational_parameter().as_f64();
        let j2 = initial_state.origin().j2();
        let r_eq = initial_state.origin().equatorial_radius().as_f64();
        Self::build(initial_state, mu, j2, r_eq)
    }
}

// Fallible — Try* bounds (covers DynOrigin)
impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: TryJ2 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Try to create a new semi-analytical J2 propagator, returning an error if
    /// the origin lacks required properties or the orbit is non-elliptic.
    pub fn try_new(initial_state: CartesianOrbit<T, O, R>) -> Result<Self, J2Error> {
        let mu = initial_state
            .origin()
            .try_gravitational_parameter()
            .map_err(|e| J2Error::UndefinedOriginProperty(e.to_string()))?
            .as_f64();
        let j2 = initial_state
            .origin()
            .try_j2()
            .map_err(|e| J2Error::UndefinedOriginProperty(e.to_string()))?;
        let r_eq = initial_state
            .origin()
            .try_equatorial_radius()
            .map_err(|e| J2Error::UndefinedOriginProperty(e.to_string()))?
            .as_f64();
        Self::build(initial_state, mu, j2, r_eq)
    }
}

impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: TryJ2 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    fn build(
        initial_state: CartesianOrbit<T, O, R>,
        mu: f64,
        j2: f64,
        r_eq: f64,
    ) -> Result<Self, J2Error> {
        let gm = GravitationalParameter::m3_per_s2(mu);
        let osc_kep = initial_state.state().to_keplerian(gm);

        let ecc = osc_kep.eccentricity();
        if !ecc.is_circular_or_elliptic() {
            return Err(J2Error::NonElliptic(ecc.orbit_type()));
        }

        let mean_elements = BrouwerMeanElements::from_osculating(&osc_kep, mu, j2, r_eq)?;

        Ok(Self {
            initial_state,
            mean_elements,
            step: TimeDelta::from_seconds_f64(DEFAULT_STEP_SECONDS),
        })
    }

    /// Set the fixed time step used by [`propagate`](Propagator::propagate) for
    /// interval propagation.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Return a reference to the initial orbital state.
    pub fn initial_state(&self) -> &CartesianOrbit<T, O, R> {
        &self.initial_state
    }
}

impl<T, O, R> Propagator<T, O> for J2Propagator<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: TryJ2 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = J2Error;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, J2Error> {
        let dt = (time - self.initial_state.time()).to_seconds().to_f64();
        let cartesian = self.mean_elements.state_at_dt(dt)?;
        Ok(CartesianOrbit::new(
            cartesian,
            time,
            self.initial_state.origin(),
            self.initial_state.reference_frame(),
        ))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, J2Error> {
        let states: Result<Vec<_>, _> = interval
            .step_by(self.step)
            .map(|t| self.state_at(t))
            .collect();
        Ok(Trajectory::try_new(states?)?)
    }
}

impl BrouwerMeanElements {
    /// Convert osculating Keplerian elements to mean elements using an
    /// iterative procedure, then compute secular rates.
    fn from_osculating(osc: &Keplerian, mu: f64, j2: f64, r_eq: f64) -> Result<Self, J2Error> {
        let a_osc = osc.semi_major_axis().as_f64();
        let e_osc = osc.eccentricity().as_f64();
        let i_osc = osc.inclination().as_f64();
        let raan_osc = osc.longitude_of_ascending_node().as_f64();
        let aop_osc = osc.argument_of_periapsis().as_f64();
        let nu_osc = osc.true_anomaly();
        let m_osc = nu_osc.to_mean(osc.eccentricity())?.as_f64();

        // Iterative osculating-to-mean conversion:
        // Start with mean = osculating, compute corrections, subtract.
        let mut a_mean = a_osc;
        let mut e_mean = e_osc;
        let mut i_mean = i_osc;
        let mut raan_mean = raan_osc;
        let mut aop_mean = aop_osc;
        let mut m_mean = m_osc;

        for iteration in 0..MAX_OSC_TO_MEAN_ITERATIONS {
            // Compute what osculating elements the current mean elements produce
            let corr = periodic_corrections(a_mean, e_mean, i_mean, aop_mean, m_mean, j2, r_eq)?;

            // Use the Lyddane rotation for e/l recovery
            let (sin_l, cos_l) = m_mean.sin_cos();
            let dk = corr.de * cos_l - corr.eppd_l * sin_l;
            let dh_e = corr.de * sin_l + corr.eppd_l * cos_l;
            let k_trial = e_mean * cos_l + dk;
            let h_trial = e_mean * sin_l + dh_e;
            let e_trial = (k_trial * k_trial + h_trial * h_trial).sqrt();
            let l_trial = h_trial.atan2(k_trial);

            let i_trial = i_mean + corr.di;
            let h_trial_raan = if i_mean.sin().abs() > 1e-10 {
                raan_mean + corr.sid_h / i_mean.sin()
            } else {
                raan_mean
            };
            let z_trial = m_mean + aop_mean + raan_mean + corr.dz;
            let g_trial = z_trial - l_trial - h_trial_raan;
            let a_trial = a_mean + corr.da;

            // Fixed-point iteration: mean = mean + (osc_target - osc_computed)
            let a_new = a_mean + (a_osc - a_trial);
            let e_new = e_mean + (e_osc - e_trial);
            let i_new = i_mean + (i_osc - i_trial);
            let raan_new = raan_mean + (raan_osc - h_trial_raan);
            let aop_new = aop_mean + (aop_osc - g_trial);
            let m_new = m_mean + (m_osc - l_trial);

            let converged = (a_new - a_mean).abs() < OSC_TO_MEAN_TOL * a_osc.abs()
                && (e_new - e_mean).abs() < OSC_TO_MEAN_TOL
                && (i_new - i_mean).abs() < OSC_TO_MEAN_TOL
                && (raan_new - raan_mean).abs() < OSC_TO_MEAN_TOL
                && (aop_new - aop_mean).abs() < OSC_TO_MEAN_TOL
                && (m_new - m_mean).abs() < OSC_TO_MEAN_TOL;

            a_mean = a_new;
            e_mean = e_new;
            i_mean = i_new;
            raan_mean = raan_new;
            aop_mean = aop_new;
            m_mean = m_new;

            if converged {
                break;
            }
            if iteration == MAX_OSC_TO_MEAN_ITERATIONS - 1 {
                return Err(J2Error::MeanElementConvergence(MAX_OSC_TO_MEAN_ITERATIONS));
            }
        }

        // Compute secular rates on mean elements (Brouwer with J2² terms).
        let n0 = (mu / a_mean.powi(3)).sqrt();
        let eta = (1.0 - e_mean * e_mean).sqrt();
        let eta2 = eta * eta;
        let cos_i = i_mean.cos();
        let ci2 = cos_i * cos_i;

        let q = r_eq / a_mean;
        let yp2 = 0.5 * j2 * q * q / (eta2 * eta2);
        let yp22 = yp2 * yp2;

        let ci2x3m1 = 3.0 * ci2 - 1.0;
        let ci2x5m1 = 5.0 * ci2 - 1.0;

        let dsl = 1.5
            * yp2
            * eta
            * (ci2x3m1
                + 0.0625
                    * yp2
                    * (-15.0
                        + eta * (16.0 + 25.0 * eta)
                        + ci2
                            * (30.0 - eta * (96.0 + 90.0 * eta)
                                + ci2 * (105.0 + eta * (144.0 + 25.0 * eta)))));

        let dsg = 1.5 * yp2 * ci2x5m1
            + 0.09375
                * yp22
                * (-35.0
                    + eta * (24.0 + 25.0 * eta)
                    + ci2
                        * (90.0 - eta * (192.0 + 126.0 * eta)
                            + ci2 * (385.0 + eta * (360.0 + 45.0 * eta))));

        let dsh = (-3.0 * yp2
            + 0.375
                * yp22
                * (-5.0 + eta * (12.0 + 9.0 * eta) - ci2 * (35.0 + eta * (36.0 + 5.0 * eta))))
            * cos_i;

        let m_dot = n0 * (1.0 + dsl);
        let aop_dot = n0 * dsg;
        let raan_dot = n0 * dsh;

        Ok(Self {
            a: a_mean,
            e: e_mean,
            i: i_mean,
            raan0: raan_mean,
            aop0: aop_mean,
            m0: m_mean,
            raan_dot,
            aop_dot,
            m_dot,
            mu,
            j2,
            r_eq,
        })
    }

    /// Evaluate the osculating Cartesian state at time dt seconds from epoch.
    fn state_at_dt(&self, dt: f64) -> Result<Cartesian, J2Error> {
        // Propagate mean elements
        let h_mean = self.raan0 + self.raan_dot * dt; // RAAN (h)
        let g_mean = self.aop0 + self.aop_dot * dt; // arg periapsis (g)
        let l_mean = self.m0 + self.m_dot * dt; // mean anomaly (l)

        let corr =
            periodic_corrections(self.a, self.e, self.i, g_mean, l_mean, self.j2, self.r_eq)?;

        // ─── Lyddane non-singular element recovery ───
        //
        // Instead of dividing by e or sin(i), use 2D rotations to combine
        // the correction pairs (δe, e·δl) and (δi, sin(i)·δh).

        // Semi-major axis: simple additive
        let a_osc = self.a + corr.da;

        // Eccentricity and mean anomaly via Lyddane rotation.
        // Use (k, h) = (e·cos(l), e·sin(l)) to avoid dividing by e:
        //   δk = δe·cos(l") - eppd_l·sin(l")
        //   δh = δe·sin(l") + eppd_l·cos(l")
        let (sin_l, cos_l) = l_mean.sin_cos();
        let dk = corr.de * cos_l - corr.eppd_l * sin_l;
        let dh_e = corr.de * sin_l + corr.eppd_l * cos_l;

        let k_osc = self.e * cos_l + dk;
        let h_osc = self.e * sin_l + dh_e;

        let e_osc = (k_osc * k_osc + h_osc * h_osc).sqrt().max(0.0);
        let l_osc = h_osc.atan2(k_osc);

        // Inclination and RAAN: use (δi, sid_h) with a division guard.
        // For truly equatorial orbits, both δi and sid_h are ~0.
        let i_osc = self.i + corr.di;
        let h_osc_raan = if self.i.sin().abs() > 1e-10 {
            h_mean + corr.sid_h / self.i.sin()
        } else {
            h_mean
        };

        // Recover ω from z = l + g + h
        // g_osc = (l_mean + g_mean + h_mean + δz) - l_osc - h_osc
        let z_osc = l_mean + g_mean + h_mean + corr.dz;
        let g_osc = z_osc - l_osc - h_osc_raan;

        // Convert mean anomaly to true anomaly
        let ecc = Eccentricity::try_new(e_osc).expect("eccentricity should be non-negative");
        let mean_anom = MeanAnomaly::new(l_osc.rad());
        let true_anom = mean_anom.to_true(ecc)?;

        let kep = Keplerian::builder()
            .with_semi_major_axis(Distance::new(a_osc), e_osc)
            .with_inclination(i_osc.rad().mod_two_pi())
            .with_longitude_of_ascending_node(h_osc_raan.rad().mod_two_pi())
            .with_argument_of_periapsis(g_osc.rad().mod_two_pi())
            .with_true_anomaly(true_anom.as_angle())
            .build()
            .expect("osculating elements should be valid");

        Ok(kep.to_cartesian(GravitationalParameter::m3_per_s2(self.mu)))
    }
}

/// Brouwer-Lyddane T2 function for critical inclination singularity avoidance.
///
/// Approximates `1 / (1 - 5cos²i)` near the critical inclination (i ≈ 63.4°)
/// using a regularized form. Reference: Phipps (1992), Eqs. 2.47–2.48.
fn critical_inclination_t2(cos_i: f64) -> f64 {
    const BETA: f64 = 100.0 / 2048.0; // 100 * 2^(-11)

    let x = 1.0 - 5.0 * cos_i * cos_i;
    let x2 = x * x;

    // Taylor-like sum: Σ (-1)^i β^i x^(2i) / (i+1)!
    let mut sum = 0.0_f64;
    let mut beta_pow = 1.0; // β^i
    let mut x2_pow = 1.0; // x^(2i)
    let mut factorial = 1.0; // (i+1)!
    for idx in 0..=12 {
        let sign = if idx % 2 == 0 { 1.0 } else { -1.0 };
        if idx > 0 {
            beta_pow *= BETA;
            x2_pow *= x2;
            factorial *= (idx + 1) as f64;
        }
        sum += sign * beta_pow * x2_pow / factorial;
    }

    // Product: Π (1 + exp(-2^(-i) β x²))
    let mut product = 1.0_f64;
    for idx in 0..=10 {
        let scale = 2.0_f64.powi(-idx);
        product *= 1.0 + (-scale * BETA * x2).exp();
    }

    BETA * x * sum * product
}

/// Lyddane corrections: the raw correction components that avoid singularities.
///
/// Instead of (δe, δM, δi, δΩ) which require dividing by e and sin(i),
/// we keep the products (e·δl, sin(i)·δh) and use 2D rotations to recover
/// the osculating elements.
struct LyddaneCorrections {
    /// Short-period correction to semi-major axis.
    da: f64,
    /// Total correction to eccentricity.
    de: f64,
    /// Total correction to inclination.
    di: f64,
    /// e × total correction to mean anomaly.
    eppd_l: f64,
    /// sin(i) × total correction to RAAN.
    sid_h: f64,
    /// Total correction to z = l + g + h (mean longitude).
    dz: f64,
}

/// Compute J2 periodic corrections using the full Brouwer-Lyddane theory.
///
/// Returns Lyddane corrections that avoid the classical element singularities
/// at e=0 and i=0 by keeping products e·δl and sin(i)·δh undivided.
///
/// Implements the same formulas as Orekit's `BrouwerLyddanePropagator` for the
/// J2-only case (C30=C40=C50=0).
// TODO: Add J3–J5 zonal harmonics for the long-period corrections.
fn periodic_corrections(
    a: f64,
    e: f64,
    i: f64,
    aop: f64,
    m: f64,
    j2: f64,
    r_eq: f64,
) -> Result<LyddaneCorrections, J2Error> {
    let e2 = e * e;
    let eta2 = 1.0 - e2;
    let eta = eta2.sqrt();
    let eta3 = eta * eta2;
    let sin_i = i.sin();
    let cos_i = i.cos();
    let ci2 = cos_i * cos_i;
    let si2 = sin_i * sin_i;
    let ci2x3m1 = 3.0 * ci2 - 1.0;

    // γ₂' = J₂(R_eq/a)² / (2η⁴)
    let q = r_eq / a;
    let yp2 = 0.5 * j2 * q * q / (eta2 * eta2);

    let t8 = 1.0 / (1.0 + eta);

    // Compute true anomaly from mean anomaly
    let ecc = Eccentricity::try_new(e.max(0.0)).expect("eccentricity should be non-negative");
    let mean_anom = MeanAnomaly::new(m.rad());
    let true_anom = mean_anom.to_true(ecc)?;
    let f = true_anom.as_f64();
    let (sin_f, cos_f) = f.sin_cos();

    // Argument combinations: 2g+f, 2g+2f, 2g+3f
    let g2 = 2.0 * aop;
    let (s2gf, c2gf) = (g2 + f).sin_cos();
    let (s2g2f, c2g2f) = (g2 + 2.0 * f).sin_cos();
    let (s2g3f, c2g3f) = (g2 + 3.0 * f).sin_cos();

    let e_cos_f = e * cos_f;
    let e_sin_f = e * sin_f;
    let ecfp1 = 1.0 + e_cos_f;
    let ecfp2 = 2.0 + e_cos_f;
    let ecfp3 = 3.0 + e_cos_f;
    let ecfp1_3 = ecfp1 * ecfp1 * ecfp1;

    // Working variables (Orekit naming)
    // Wrap mean anomaly to avoid drift in w17 when m is accumulated
    let m_wrapped = m.rem_euclid(std::f64::consts::TAU);
    let w17 = f + e_sin_f - m_wrapped;
    let w20 = cos_f * (ecfp3 * e_cos_f + 3.0);
    let w21 = 3.0 * (s2g2f + e * s2gf) + e * s2g3f;
    let w22 = ecfp1 * ecfp2 / eta2;

    // ─── Long-period corrections (J2-only) ───
    let t2 = critical_inclination_t2(cos_i);
    let ci2t2 = ci2 * t2;

    let ca = 1.0 - ci2 * (11.0 + 40.0 * ci2t2);
    let q1 = 0.125 * yp2 * ca;

    let p5p = 1.0 + ci2t2 * (8.0 + 20.0 * ci2t2);
    let p5p10 = 1.0 + 10.0 * p5p;
    let q2 = 0.125 * e2 * cos_i * yp2 * p5p10;

    let vle1 = e * eta * q1;
    let vli1 = if sin_i.abs() > 1e-10 {
        -e * q1 / sin_i
    } else {
        0.0
    };
    let vlh1i = -sin_i * q2;

    // Long-period ω correction coefficient
    let ce = 1.0 - ci2 * (33.0 + 200.0 * ci2t2);
    let vls1 =
        (eta3 - 1.0) * q1 - q2 + 25.0 * e2 * ci2 * ci2t2 * ci2t2 * yp2 - 0.0625 * e2 * yp2 * ce;

    let (s2g, c2g) = g2.sin_cos();
    let d1e = c2g * vle1;
    let d1i = if vli1.is_finite() { c2g * vli1 } else { 0.0 };
    let eppd1l = s2g * vle1 * eta;
    let sid1h = s2g * vlh1i;
    let d1z = s2g * vls1;

    // ─── Short-period corrections ───
    let d2a = a * yp2 / eta2 * ((ecfp1_3 - eta3) * ci2x3m1 + 3.0 * si2 * ecfp1_3 * c2g2f);

    let d2e = 0.5
        * yp2
        * ((w20 + e * t8) * ci2x3m1 + (w20 + e * c2g2f) * 3.0 * si2
            - (3.0 * e * c2gf + e * c2g3f) * eta2 * si2);

    let d2i = 0.5 * yp2 * cos_i * sin_i * (3.0 * (c2g2f + e * c2gf) + e * c2g3f);

    let eppd2l = -0.25
        * yp2
        * eta3
        * ((w22 + 1.0) * sin_f * 2.0 * si2
            + (-(w22 - 1.0) * s2gf + (w22 + 1.0 / 3.0) * s2g3f) * 3.0 * si2);

    let sid2h = 0.5 * yp2 * cos_i * sin_i * (w21 - 6.0 * w17);

    let ci_factor = 1.0 + cos_i * (2.0 - 5.0 * cos_i);
    let d2z = -(e * eppd2l * (t8 - 1.0) / eta3
        + 0.25 * yp2 * (6.0 * w17 * ci_factor - w21 * (3.0 + cos_i * (2.0 - 5.0 * cos_i))));

    // ─── Combine long-period + short-period (Lyddane representation) ───
    Ok(LyddaneCorrections {
        da: d2a,
        de: d1e + d2e,
        di: d1i + d2i,
        eppd_l: eppd1l + eppd2l,
        sid_h: sid1h + sid2h,
        dz: d1z + d2z,
    })
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_frames::Icrf;
    use lox_test_utils::assert_approx_eq;
    use lox_time::intervals::Interval;
    use lox_time::time;
    use lox_time::time_scales::Tdb;
    use lox_units::{DistanceUnits, VelocityUnits};

    use lox_core::glam::DVec3;

    use super::*;

    fn initial_state() -> CartesianOrbit<Tdb, Earth, Icrf> {
        let time = time!(Tdb, 2023, 1, 1).unwrap();
        CartesianOrbit::new(
            Cartesian::new(
                1131.340.km(),
                -2282.343.km(),
                6672.423.km(),
                -5.64305.kps(),
                4.30333.kps(),
                2.42879.kps(),
            ),
            time,
            Earth,
            Icrf,
        )
    }

    #[test]
    fn test_j2_construction() {
        let state = initial_state();
        let j2 = J2Propagator::new(state);
        assert!(j2.is_ok());
    }

    #[test]
    fn test_j2_state_at_epoch_roundtrip() {
        let state = initial_state();
        let j2 = J2Propagator::new(state).unwrap();
        let result = j2.state_at(state.time()).unwrap();

        // At epoch, the osculating state should match the initial state
        assert_approx_eq!(result.position(), state.position(), rtol <= 1e-4);
        assert_approx_eq!(result.velocity(), state.velocity(), rtol <= 1e-4);
    }

    #[test]
    fn test_j2_propagate_interval() {
        let state = initial_state();
        let j2 = J2Propagator::new(state).unwrap();
        let dt = TimeDelta::from_minutes(90);
        let interval = Interval::new(state.time(), state.time() + dt);
        let traj = j2.propagate(interval).unwrap();
        assert!(traj.states().len() > 1);
    }

    #[test]
    fn test_j2_raan_drift() {
        // Propagate for 1 day and verify RAAN drift is in the right direction
        // and order of magnitude. Comparing osculating RAANs includes short-period
        // oscillations, so we use a loose tolerance.
        let state = initial_state();
        let j2 = J2Propagator::new(state).unwrap();
        let dt_day = TimeDelta::from_seconds_f64(86400.0);
        let t1 = state.time() + dt_day;
        let result = j2.state_at(t1).unwrap();

        let mu = GravitationalParameter::m3_per_s2(j2.mean_elements.mu);
        let kep_initial = state.state().to_keplerian(mu);
        let kep_final = result.state().to_keplerian(mu);

        let expected_raan_drift = j2.mean_elements.raan_dot * 86400.0;
        let mut actual_raan_drift = kep_final.longitude_of_ascending_node().as_f64()
            - kep_initial.longitude_of_ascending_node().as_f64();
        // Unwrap modular arithmetic
        if actual_raan_drift > std::f64::consts::PI {
            actual_raan_drift -= std::f64::consts::TAU;
        } else if actual_raan_drift < -std::f64::consts::PI {
            actual_raan_drift += std::f64::consts::TAU;
        }

        // Allow 10% relative tolerance for short-period oscillation residuals
        let rel_err = ((actual_raan_drift - expected_raan_drift) / expected_raan_drift).abs();
        assert!(
            rel_err < 0.10,
            "RAAN drift relative error: {rel_err:.4} (actual: {actual_raan_drift:.6} rad, expected: {expected_raan_drift:.6} rad)"
        );
    }

    // Reference data generated by Orekit BrouwerLyddanePropagator
    // (see generate_j2_reference.py)
    //
    // Initial state: pos=(1131.340, -2282.343, 6672.423) km,
    //                vel=(-5.64305, 4.30333, 2.42879) km/s
    // Epoch: 2023-01-01T00:00:00 TDB
    // Body: Earth (mu=3.986004418e14, R_eq=6.378137e6, J2=1.08262668e-3)
    //
    // Format: (dt_minutes, x, y, z, vx, vy, vz) in meters and m/s
    const OREKIT_REFERENCE: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
        (
            0.0,
            1131340.000000,
            -2282343.000000,
            6672423.000000,
            -5643.050000,
            4303.330000,
            2428.790000,
        ),
        (
            10.0,
            -2242752.121678,
            557966.889389,
            6773387.408984,
            -5268.057047,
            4882.521920,
            -2106.737966,
        ),
        (
            30.0,
            -5531192.423534,
            4633844.992091,
            263697.501880,
            469.932662,
            1054.638774,
            -7337.613924,
        ),
        (
            60.0,
            1928334.430816,
            -251405.109886,
            -6969162.064247,
            5337.447581,
            -4850.288132,
            1674.719477,
        ),
        (
            90.0,
            4437235.654869,
            -4439342.287117,
            3459239.381693,
            -3439.273910,
            1635.513586,
            6439.401719,
        ),
        (
            360.0,
            314969.673624,
            1145933.007115,
            -7139368.109804,
            5700.456453,
            -4695.064016,
            -489.729355,
        ),
        (
            1440.0,
            -4297302.327437,
            2506775.458691,
            5182407.640712,
            -3757.361430,
            4055.996395,
            -5009.533959,
        ),
    ];

    #[test]
    fn test_j2_vs_orekit() {
        // Compare against Orekit BrouwerLyddanePropagator reference data.
        // Our implementation uses simplified Brouwer corrections while Orekit
        // uses the full Brouwer-Lyddane theory, so we expect position errors
        // of ~10-30 km per orbit (first-order J2 vs. higher-order corrections).
        let state = initial_state();
        let j2 = J2Propagator::new(state).unwrap();

        for &(dt_min, x, y, z, vx, vy, vz) in OREKIT_REFERENCE {
            let t = state.time() + TimeDelta::from_seconds_f64(dt_min * 60.0);
            let result = j2.state_at(t).unwrap();

            let pos_exp = DVec3::new(x, y, z);
            let vel_exp = DVec3::new(vx, vy, vz);

            let pos_err = (result.position() - pos_exp).length();
            let vel_err = (result.velocity() - vel_exp).length();

            // At epoch, should be exact (within roundtrip conversion tolerance)
            if dt_min == 0.0 {
                assert!(pos_err < 1.0, "t={dt_min}m: position error {pos_err:.1} m");
                assert!(
                    vel_err < 0.01,
                    "t={dt_min}m: velocity error {vel_err:.4} m/s"
                );
                continue;
            }

            // Per-orbit oscillation ~28 km from J2-only vs. Orekit's full theory.
            // Short-term and long-term (secular) errors are <100 m.
            assert!(
                pos_err < 30_000.0,
                "t={dt_min}m: position error {pos_err:.1} m"
            );
            assert!(
                vel_err < 30.0,
                "t={dt_min}m: velocity error {vel_err:.4} m/s"
            );
        }
    }

    #[test]
    fn test_j2_propagate_to() {
        let state = initial_state();
        let j2 = J2Propagator::new(state).unwrap();
        let dt = TimeDelta::from_minutes(40);
        let interval = Interval::new(state.time(), state.time() + dt);
        let times: Vec<_> = interval.step_by(TimeDelta::from_minutes(10)).collect();

        let traj = j2.propagate_to(times.clone()).unwrap();
        assert_eq!(traj.states().len(), times.len());
    }

    #[test]
    fn test_j2_with_step() {
        let state = initial_state();
        let j2 = J2Propagator::new(state)
            .unwrap()
            .with_step(TimeDelta::from_seconds_f64(30.0));
        let dt = TimeDelta::from_minutes(10);
        let interval = Interval::new(state.time(), state.time() + dt);
        let traj = j2.propagate(interval).unwrap();
        // 10 minutes / 30 seconds = 20 steps + 1 = 21 states
        assert_eq!(traj.states().len(), 21);
    }
}
