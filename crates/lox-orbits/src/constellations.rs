// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Constellation design algorithms.
//!
//! This module provides builders for generating satellite constellations:
//!
//! - [`WalkerDeltaBuilder`] — Walker Delta (RAAN spread = 360°)
//! - [`WalkerStarBuilder`] — Walker Star (RAAN spread = 180°)
//! - [`StreetOfCoverageBuilder`] — Street-of-Coverage optimized constellations
//! - [`FlowerBuilder`] — Flower constellations (repeating ground tracks)

mod flower;
mod street_of_coverage;
mod walker;

use lox_bodies::{DynOrigin, Origin};
use lox_core::elements::{Keplerian, KeplerianError};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::Time;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

pub use flower::FlowerBuilder;
pub use street_of_coverage::StreetOfCoverageBuilder;
pub use walker::{WalkerDeltaBuilder, WalkerStarBuilder};

/// A single satellite in a constellation, described by its plane index,
/// position within the plane, and Keplerian orbital elements.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstellationSatellite {
    /// Index of the orbital plane this satellite belongs to.
    pub plane: usize,
    /// Zero-based index of the satellite within its plane.
    pub index_in_plane: usize,
    /// Keplerian orbital elements of the satellite.
    pub elements: Keplerian,
}

/// Errors that can occur when building a constellation.
#[derive(Debug, Clone, Error)]
pub enum ConstellationError {
    /// The number of satellites is not evenly divisible by the number of planes.
    #[error("number of satellites ({nsats}) is not divisible by number of planes ({nplanes})")]
    SatellitePlaneMismatch {
        /// Total satellite count.
        nsats: usize,
        /// Number of orbital planes.
        nplanes: usize,
    },
    /// Zero orbital planes were specified.
    #[error("number of planes must be greater than zero")]
    ZeroPlanes,
    /// Zero satellites were specified.
    #[error("number of satellites must be greater than zero")]
    ZeroSatellites,
    /// Too few satellites to satisfy the requested coverage fold.
    #[error("too few satellites for the requested coverage fold")]
    SocConstraint,
    /// Street-of-coverage requires a non-equatorial inclination.
    #[error("street-of-coverage requires non-equatorial inclination")]
    SocEquatorialInclination,
    /// The street-of-coverage optimizer failed to converge.
    #[error("street-of-coverage optimization did not converge")]
    SocNotConverged,
    /// Neither perigee altitude nor semi-major axis was provided.
    #[error("neither perigee altitude nor semi-major axis was provided")]
    MissingShape,
    /// A required body property is not defined for the chosen origin.
    #[error("required body property is not defined: {0}")]
    UndefinedProperty(String),
    /// An error occurred while constructing Keplerian elements.
    #[error(transparent)]
    Keplerian(#[from] KeplerianError),
}

/// The propagator to use when converting constellation satellites into
/// propagatable spacecraft.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConstellationPropagator {
    /// Vallado two-body propagator (default).
    #[default]
    Vallado,
    /// J2-perturbed analytical propagator.
    J2,
}

/// A named collection of satellites produced by a constellation design algorithm,
/// combined with the epoch, origin, and frame needed to create propagatable orbits.
#[derive(Debug, Clone)]
pub struct Constellation<T: TimeScale, O: Origin, R: ReferenceFrame> {
    name: String,
    epoch: Time<T>,
    origin: O,
    frame: R,
    satellites: Vec<ConstellationSatellite>,
    propagator: ConstellationPropagator,
}

/// Type alias for a constellation with fully dynamic time scale, origin, and frame.
pub type DynConstellation = Constellation<DynTimeScale, DynOrigin, DynFrame>;

impl<T: TimeScale, O: Origin, R: ReferenceFrame> Constellation<T, O, R> {
    /// Creates a new constellation from precomputed satellites.
    pub fn new(
        name: impl Into<String>,
        epoch: Time<T>,
        origin: O,
        frame: R,
        satellites: Vec<ConstellationSatellite>,
    ) -> Self {
        Self {
            name: name.into(),
            epoch,
            origin,
            frame,
            satellites,
            propagator: ConstellationPropagator::default(),
        }
    }

    /// Sets the propagator to use for this constellation.
    pub fn with_propagator(mut self, propagator: ConstellationPropagator) -> Self {
        self.propagator = propagator;
        self
    }

    /// Returns the constellation name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the reference epoch.
    pub fn epoch(&self) -> Time<T>
    where
        T: Copy,
    {
        self.epoch
    }

    /// Returns the central body.
    pub fn origin(&self) -> O
    where
        O: Copy,
    {
        self.origin
    }

    /// Returns the reference frame.
    pub fn frame(&self) -> R
    where
        R: Copy,
    {
        self.frame
    }

    /// Returns a slice of all satellites in the constellation.
    pub fn satellites(&self) -> &[ConstellationSatellite] {
        &self.satellites
    }

    /// Returns the selected propagator type.
    pub fn propagator(&self) -> ConstellationPropagator {
        self.propagator
    }

    /// Returns the total number of satellites.
    pub fn len(&self) -> usize {
        self.satellites.len()
    }

    /// Returns `true` if the constellation contains no satellites.
    pub fn is_empty(&self) -> bool {
        self.satellites.is_empty()
    }
}

impl<T, O, R> Constellation<T, O, R>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
{
    /// Converts the constellation into a fully dynamic representation.
    pub fn into_dyn(self) -> DynConstellation {
        Constellation {
            name: self.name,
            epoch: self.epoch.into_dyn(),
            origin: self.origin.into(),
            frame: self.frame.into(),
            satellites: self.satellites,
            propagator: self.propagator,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::Earth;
    use lox_frames::Icrf;
    use lox_time::Time;
    use lox_time::time_scales::Tai;
    use lox_units::{AngleUnits, DistanceUnits};

    fn make_constellation() -> Constellation<Tai, Earth, Icrf> {
        WalkerDeltaBuilder::new(6, 3)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build_constellation("test", Time::j2000(Tai), Earth, Icrf)
            .unwrap()
    }

    #[test]
    fn test_constellation_new() {
        let epoch = Time::j2000(Tai);
        let sats = vec![];
        let c = Constellation::new("empty", epoch, Earth, Icrf, sats);
        assert_eq!(c.name(), "empty");
        assert_eq!(c.len(), 0);
        assert!(c.is_empty());
    }

    #[test]
    fn test_constellation_getters() {
        let c = make_constellation();
        assert_eq!(c.name(), "test");
        assert_eq!(c.epoch(), Time::j2000(Tai));
        assert_eq!(c.origin(), Earth);
        assert_eq!(c.frame(), Icrf);
        assert_eq!(c.len(), 6);
        assert!(!c.is_empty());
        assert_eq!(c.satellites().len(), 6);
        assert_eq!(c.propagator(), ConstellationPropagator::Vallado);
    }

    #[test]
    fn test_constellation_with_propagator() {
        let c = make_constellation().with_propagator(ConstellationPropagator::J2);
        assert_eq!(c.propagator(), ConstellationPropagator::J2);
    }

    #[test]
    fn test_constellation_propagator_default() {
        assert_eq!(
            ConstellationPropagator::default(),
            ConstellationPropagator::Vallado
        );
    }

    #[test]
    fn test_constellation_into_dyn() {
        let c = make_constellation().with_propagator(ConstellationPropagator::J2);
        let dyn_c = c.into_dyn();
        assert_eq!(dyn_c.name(), "test");
        assert_eq!(dyn_c.len(), 6);
        assert_eq!(dyn_c.origin(), DynOrigin::Earth);
        assert_eq!(dyn_c.frame(), DynFrame::Icrf);
        assert_eq!(dyn_c.propagator(), ConstellationPropagator::J2);
    }

    #[test]
    fn test_constellation_error_display() {
        let e = ConstellationError::SatellitePlaneMismatch {
            nsats: 7,
            nplanes: 3,
        };
        assert!(e.to_string().contains("not divisible"));

        assert!(ConstellationError::ZeroPlanes.to_string().contains("zero"));
        assert!(
            ConstellationError::ZeroSatellites
                .to_string()
                .contains("zero")
        );
        assert!(
            ConstellationError::SocConstraint
                .to_string()
                .contains("coverage fold")
        );
        assert!(
            ConstellationError::SocEquatorialInclination
                .to_string()
                .contains("non-equatorial")
        );
        assert!(
            ConstellationError::SocNotConverged
                .to_string()
                .contains("converge")
        );
        assert!(
            ConstellationError::MissingShape
                .to_string()
                .contains("neither")
        );
        assert!(
            ConstellationError::UndefinedProperty("mu".into())
                .to_string()
                .contains("mu")
        );
    }
}
