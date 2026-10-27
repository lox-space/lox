// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Convenience re-exports of the most commonly used types.
//!
//! ```
//! use lox_space::prelude::*;
//! ```

// Time
#[cfg(feature = "time")]
pub use crate::time::Time;
#[cfg(feature = "time")]
pub use crate::time::deltas::TimeDelta;
#[cfg(feature = "time")]
pub use crate::time::intervals::Interval;
#[cfg(feature = "time")]
pub use crate::time::time_scales::{Tai, Tdb, TimeScale, Ut1};
#[cfg(feature = "time")]
pub use crate::time::utc::Utc;

// Units (types + extension traits for 800.0.km() syntax)
pub use crate::core::units::{Angle, AngleUnits, Distance, DistanceUnits, Velocity, VelocityUnits};

// Orbital elements & state
pub use crate::core::coords::Cartesian;
pub use crate::core::elements::Keplerian;
#[cfg(feature = "orbits")]
pub use crate::orbits::sso::SsoBuilder;
#[cfg(feature = "orbits")]
pub use crate::orbits::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};

// Propagation
#[cfg(feature = "orbits")]
pub use crate::orbits::propagators::Propagator;
#[cfg(feature = "orbits")]
pub use crate::orbits::propagators::j2::J2Propagator;
#[cfg(feature = "orbits")]
pub use crate::orbits::propagators::numerical::NumericalPropagator;
#[cfg(feature = "orbits")]
pub use crate::orbits::propagators::semi_analytical::Vallado;

// Bodies & frames
#[cfg(feature = "bodies")]
pub use crate::bodies::{Earth, Moon, Origin, Sun};
#[cfg(feature = "frames")]
pub use crate::frames::{Frame, Icrf, Itrf};

// Earth orientation
#[cfg(feature = "earth")]
pub use crate::earth::eop::{EopParser, EopProvider};

// Ground
#[cfg(feature = "orbits")]
pub use crate::orbits::ground::EllipsoidLocation;

// Analysis
#[cfg(feature = "analysis")]
pub use crate::analysis::assets::{GroundStation, Scenario, Spacecraft};
#[cfg(feature = "analysis")]
pub use crate::analysis::visibility::{Pass, VisibilityAnalysis, VisibilityResults};
