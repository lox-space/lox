// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use lox_core::units::AngularRate;

use crate::analysis::ElevationMask;
use crate::ground::DynGroundLocation;
use crate::orbits::DynTrajectory;

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

#[derive(Debug, Clone)]
pub struct GroundAsset {
    id: AssetId,
    location: DynGroundLocation,
    mask: ElevationMask,
}

impl GroundAsset {
    pub fn new(id: impl Into<String>, location: DynGroundLocation, mask: ElevationMask) -> Self {
        Self {
            id: AssetId::new(id),
            location,
            mask,
        }
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
}

#[derive(Debug, Clone)]
pub struct SpaceAsset {
    id: AssetId,
    trajectory: DynTrajectory,
    max_slew_rate: Option<AngularRate>,
}

impl SpaceAsset {
    pub fn new(id: impl Into<String>, trajectory: DynTrajectory) -> Self {
        Self {
            id: AssetId::new(id),
            trajectory,
            max_slew_rate: None,
        }
    }

    pub fn with_max_slew_rate(mut self, rate: AngularRate) -> Self {
        self.max_slew_rate = Some(rate);
        self
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn trajectory(&self) -> &DynTrajectory {
        &self.trajectory
    }

    pub fn max_slew_rate(&self) -> Option<AngularRate> {
        self.max_slew_rate
    }
}
