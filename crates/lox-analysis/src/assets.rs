// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::fmt;

use lox_core::units::AngularRate;
use lox_frames::DynFrame;
use lox_frames::rotations::TryRotation;
use lox_time::Time;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, Tai};
use rayon::prelude::*;

#[cfg(feature = "comms")]
use lox_comms::system::CommunicationSystem;

use crate::visibility::ElevationMask;
use lox_orbits::ground::DynGroundLocation;
use lox_orbits::orbits::DynEnsemble;
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
    network: Option<NetworkId>,
    #[cfg(feature = "comms")]
    communication_systems: Vec<CommunicationSystem>,
}

impl GroundStation {
    pub fn new(id: impl Into<String>, location: DynGroundLocation, mask: ElevationMask) -> Self {
        Self {
            id: AssetId::new(id),
            location,
            mask,
            network: None,
            #[cfg(feature = "comms")]
            communication_systems: Vec::new(),
        }
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

#[derive(Debug, Clone)]
pub struct Scenario {
    interval: TimeInterval<Tai>,
    ground_stations: Vec<GroundStation>,
    spacecraft: Vec<Spacecraft>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScenarioPropagateError {
    #[error("propagation failed for spacecraft \"{0}\": {1}")]
    Propagate(AssetId, PropagateError),
    #[error("frame transformation failed for spacecraft \"{0}\": {1}")]
    FrameTransformation(AssetId, String),
}

impl Scenario {
    pub fn new(start_time: Time<Tai>, end_time: Time<Tai>) -> Self {
        let interval = TimeInterval::new(start_time, end_time);
        Self::with_interval(interval)
    }

    pub fn with_interval(interval: TimeInterval<Tai>) -> Self {
        Self {
            interval,
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

    pub fn ground_stations(&self) -> &[GroundStation] {
        &self.ground_stations
    }

    pub fn spacecraft(&self) -> &[Spacecraft] {
        &self.spacecraft
    }

    /// Propagate all spacecraft over the scenario interval, transforming
    /// trajectories to the given target `frame` using the provided rotation
    /// `provider`.
    pub fn propagate<P>(
        &self,
        frame: DynFrame,
        provider: &P,
    ) -> Result<DynEnsemble<AssetId>, ScenarioPropagateError>
    where
        P: TryRotation<DynFrame, DynFrame, DynTimeScale> + Send + Sync,
        P::Error: std::fmt::Display,
    {
        let dyn_interval = TimeInterval::new(
            self.interval.start().into_dyn(),
            self.interval.end().into_dyn(),
        );
        let entries: Result<HashMap<_, _>, _> = self
            .spacecraft
            .par_iter()
            .map(|sc| {
                let traj = sc
                    .orbit
                    .propagate(dyn_interval)
                    .map_err(|e| ScenarioPropagateError::Propagate(sc.id.clone(), e))?;
                let traj = if traj.reference_frame() != frame {
                    traj.into_frame(frame, provider).map_err(|e| {
                        ScenarioPropagateError::FrameTransformation(sc.id.clone(), e.to_string())
                    })?
                } else {
                    traj
                };
                Ok((sc.id.clone(), traj))
            })
            .collect();
        Ok(DynEnsemble::new(entries?))
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
