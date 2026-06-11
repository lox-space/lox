// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Line-of-sight pointing at link terminals.

use lox_core::glam::DVec3;
use lox_core::units::Angle;

/// Line-of-sight pointing at one end of a link.
///
/// Describes how the line of sight relates to the terminal's antenna so the
/// link budget can evaluate the gain pattern in the right direction.
/// [`Pointing::Direction`] is the primary form for trajectory-driven analyses;
/// the other variants are convenience entry points for ideal pointing and for
/// pre-computed pattern angles.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Pointing {
    /// The antenna boresight is aligned with the line of sight (θ = φ = 0).
    #[default]
    Boresight,
    /// Explicit pattern angles in the antenna frame.
    Angles {
        /// Polar angle from boresight.
        theta: Angle,
        /// Azimuth about boresight from the antenna-frame `+X` axis toward `+Y`.
        phi: Angle,
    },
    /// Line-of-sight direction vector in the antenna's parent frame.
    Direction(DVec3),
}

impl Pointing {
    /// Convenience for an off-boresight polar angle with `φ = 0`.
    pub fn off_boresight(theta: Angle) -> Self {
        Self::Angles {
            theta,
            phi: Angle::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointing_off_boresight_is_angles_with_zero_phi() {
        let theta = Angle::degrees(1.5);
        assert_eq!(
            Pointing::off_boresight(theta),
            Pointing::Angles {
                theta,
                phi: Angle::ZERO
            }
        );
        assert_eq!(Pointing::default(), Pointing::Boresight);
    }
}
