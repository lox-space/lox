// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Convenience re-exports of the most commonly used types.
//!
//! ```
//! use lox_space::prelude::*;
//! ```

// Time
pub use crate::time::Time;
pub use crate::time::deltas::TimeDelta;
pub use crate::time::intervals::Interval;
pub use crate::time::time_scales::{Tai, Tdb, Ut1};
pub use crate::time::utc::Utc;

// Units (types + extension traits for 800.0.km() syntax)
pub use crate::core::units::{Angle, AngleUnits, Distance, DistanceUnits, Velocity, VelocityUnits};

// Orbital elements & state
pub use crate::core::coords::Cartesian;
pub use crate::core::elements::Keplerian;
pub use crate::orbits::orbits::sso::SsoBuilder;
pub use crate::orbits::orbits::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};

// Propagation
pub use crate::orbits::propagators::Propagator;
pub use crate::orbits::propagators::numerical::J2Propagator;
pub use crate::orbits::propagators::semi_analytical::Vallado;

// Bodies & frames
pub use crate::bodies::{Earth, Moon, Sun};
pub use crate::frames::{Icrf, Itrf};

// Earth orientation
pub use crate::earth::eop::{EopParser, EopProvider};

// Ground
pub use crate::orbits::ground::GroundLocation;

// Analysis
pub use crate::analysis::assets::{GroundStation, Scenario, Spacecraft};
pub use crate::analysis::visibility::{Pass, VisibilityAnalysis, VisibilityResults};
