// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::fmt;

use lox_bodies::{DynOrigin, Origin};
use lox_core::units::AngularRate;
use lox_frames::rotations::TryRotation;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::Time;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, Tai};
use rayon::prelude::*;

#[cfg(feature = "comms")]
use lox_comms::system::CommunicationSystem;

use crate::visibility::ElevationMask;
use lox_orbits::ground::DynGroundLocation;
use lox_orbits::orbits::Ensemble;
use lox_orbits::propagators::{OrbitSource, PropagateError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(String);

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstellationId(String);

impl ConstellationId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConstellationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkId(String);

impl NetworkId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GroundStation {
    id: AssetId,
    location: DynGroundLocation,
    mask: ElevationMask,
    body_fixed_frame: DynFrame,
    network: Option<NetworkId>,
    #[cfg(feature = "comms")]
    communication_systems: Vec<CommunicationSystem>,
}

impl GroundStation {
    pub fn new(id: impl Into<String>, location: DynGroundLocation, mask: ElevationMask) -> Self {
        let body_fixed_frame = DynFrame::Iau(location.origin());
        Self {
            id: AssetId::new(id),
            location,
            mask,
            body_fixed_frame,
            network: None,
            #[cfg(feature = "comms")]
            communication_systems: Vec::new(),
        }
    }

    pub fn with_body_fixed_frame(mut self, frame: impl Into<DynFrame>) -> Self {
        self.body_fixed_frame = frame.into();
        self
    }

    pub fn with_network_id(mut self, id: impl Into<String>) -> Self {
        self.network = Some(NetworkId(id.into()));
        self
    }

    #[cfg(feature = "comms")]
    pub fn with_communication_system(mut self, system: CommunicationSystem) -> Self {
        self.communication_systems.push(system);
        self
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn location(&self) -> &DynGroundLocation {
        &self.location
    }

    pub fn mask(&self) -> &ElevationMask {
        &self.mask
    }

    pub fn body_fixed_frame(&self) -> DynFrame {
        self.body_fixed_frame
    }

    #[cfg(feature = "comms")]
    pub fn communication_systems(&self) -> &[CommunicationSystem] {
        &self.communication_systems
    }
}

#[derive(Debug, Clone)]
pub struct Spacecraft {
    id: AssetId,
    orbit: OrbitSource,
    max_slew_rate: Option<AngularRate>,
    constellation: Option<ConstellationId>,
    #[cfg(feature = "comms")]
    communication_systems: Vec<CommunicationSystem>,
}

impl Spacecraft {
    pub fn new(id: impl Into<String>, orbit: OrbitSource) -> Self {
        Self {
            id: AssetId::new(id),
            orbit,
            max_slew_rate: None,
            constellation: None,
            #[cfg(feature = "comms")]
            communication_systems: Vec::new(),
        }
    }

    pub fn with_max_slew_rate(mut self, rate: AngularRate) -> Self {
        self.max_slew_rate = Some(rate);
        self
    }

    pub fn with_constellation_id(mut self, id: impl Into<String>) -> Self {
        self.constellation = Some(ConstellationId(id.into()));
        self
    }

    #[cfg(feature = "comms")]
    pub fn with_communication_system(mut self, system: CommunicationSystem) -> Self {
        self.communication_systems.push(system);
        self
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn orbit(&self) -> &OrbitSource {
        &self.orbit
    }

    pub fn max_slew_rate(&self) -> Option<AngularRate> {
        self.max_slew_rate
    }

    #[cfg(feature = "comms")]
    pub fn communication_systems(&self) -> &[CommunicationSystem] {
        &self.communication_systems
    }
}

/// A scenario declaring the analysis origin, reference frame, time interval,
/// and the assets (ground stations and spacecraft) involved.
///
/// The type parameters `O` and `R` specify the "native" origin body and
/// reference frame. For dynamic dispatch (e.g. via Python), use `DynScenario`.
#[derive(Debug, Clone)]
pub struct Scenario<O: Origin, R: ReferenceFrame> {
    interval: TimeInterval<Tai>,
    origin: O,
    frame: R,
    ground_stations: Vec<GroundStation>,
    spacecraft: Vec<Spacecraft>,
}

/// Dynamic scenario — preserves backward compatibility and serves the Python API.
pub type DynScenario = Scenario<DynOrigin, DynFrame>;

#[derive(Debug, thiserror::Error)]
pub enum ScenarioPropagateError {
    #[error("propagation failed for spacecraft \"{0}\": {1}")]
    Propagate(AssetId, PropagateError),
    #[error("frame transformation failed for spacecraft \"{0}\": {1}")]
    FrameTransformation(AssetId, String),
}

impl<O: Origin + Copy + Send + Sync, R: ReferenceFrame + Copy + Send + Sync> Scenario<O, R> {
    pub fn new(start_time: Time<Tai>, end_time: Time<Tai>, origin: O, frame: R) -> Self {
        let interval = TimeInterval::new(start_time, end_time);
        Self::with_interval(interval, origin, frame)
    }

    pub fn with_interval(interval: TimeInterval<Tai>, origin: O, frame: R) -> Self {
        Self {
            interval,
            origin,
            frame,
            ground_stations: Vec::new(),
            spacecraft: Vec::new(),
        }
    }

    pub fn with_spacecraft(mut self, spacecraft: &[Spacecraft]) -> Self {
        self.spacecraft = spacecraft.into();
        self
    }

    pub fn with_ground_stations(mut self, ground_stations: &[GroundStation]) -> Self {
        self.ground_stations = ground_stations.into();
        self
    }

    pub fn interval(&self) -> &TimeInterval<Tai> {
        &self.interval
    }

    pub fn origin(&self) -> O {
        self.origin
    }

    pub fn frame(&self) -> R {
        self.frame
    }

    pub fn ground_stations(&self) -> &[GroundStation] {
        &self.ground_stations
    }

    pub fn spacecraft(&self) -> &[Spacecraft] {
        &self.spacecraft
    }

    /// Propagate all spacecraft over the scenario interval, transforming
    /// trajectories to the scenario's frame using the provided rotation
    /// `provider`.
    ///
    /// Internally, each spacecraft's `OrbitSource` produces a `DynTrajectory`
    /// which is then rotated into the concrete frame `R` via the mixed
    /// `TryRotation<DynFrame, R, T>` impls, and finally re-tagged to
    /// `Trajectory<Tai, O, R>`.
    pub fn propagate<P>(
        &self,
        provider: &P,
    ) -> Result<Ensemble<AssetId, Tai, O, R>, ScenarioPropagateError>
    where
        R: Into<DynFrame>,
        P: TryRotation<DynFrame, R, DynTimeScale> + Send + Sync,
        P::Error: std::fmt::Display,
    {
        let dyn_interval = TimeInterval::new(
            self.interval.start().into_dyn(),
            self.interval.end().into_dyn(),
        );
        let origin = self.origin;
        let frame = self.frame;
        let entries: Result<HashMap<_, _>, _> = self
            .spacecraft
            .par_iter()
            .map(|sc| {
                let traj = sc
                    .orbit
                    .propagate(dyn_interval)
                    .map_err(|e| ScenarioPropagateError::Propagate(sc.id.clone(), e))?;
                // Rotate DynTrajectory directly into concrete frame R
                // (uses mixed TryRotation<DynFrame, R, DynTimeScale>).
                let rotated = traj.into_frame(frame, provider).map_err(|e| {
                    ScenarioPropagateError::FrameTransformation(sc.id.clone(), e.to_string())
                })?;
                // Re-tag origin and time scale (data unchanged, just type markers).
                let (epoch, _origin, frame, data) = rotated.into_parts();
                let typed = lox_orbits::orbits::Trajectory::from_parts(
                    epoch.with_scale(Tai),
                    origin,
                    frame,
                    data,
                );
                Ok((sc.id.clone(), typed))
            })
            .collect();
        Ok(Ensemble::new(entries?))
    }

    pub fn filter_by_constellations(&self, constellations: &[ConstellationId]) -> Self {
        let spacecraft = self
            .spacecraft
            .clone()
            .into_iter()
            .filter(|s| s.constellation.is_some())
            .filter(|s| constellations.contains(s.constellation.as_ref().unwrap()))
            .collect();
        Scenario {
            spacecraft,
            ..self.clone()
        }
    }

    pub fn filter_by_networks(&self, networks: &[NetworkId]) -> Self {
        let ground_stations = self
            .ground_stations
            .clone()
            .into_iter()
            .filter(|s| s.network.is_some())
            .filter(|s| networks.contains(s.network.as_ref().unwrap()))
            .collect();
        Scenario {
            ground_stations,
            ..self.clone()
        }
    }
}
