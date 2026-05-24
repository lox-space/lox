// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! SAR (Synthetic Aperture Radar) payload: side-looking annular access geometry.

use thiserror::Error;

use lox_core::units::Angle;

/// Which side of the ground track a SAR payload can image.
///
/// `Left` and `Right` are defined relative to the spacecraft's instantaneous
/// body-fixed velocity direction at the sub-satellite point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LookSide {
    /// Image only on the left side of the ground track.
    Left,
    /// Image only on the right side of the ground track.
    Right,
    /// Image on either side (roll-agile platform).
    Either,
}

/// Angular envelope of the SAR field of regard, stored in the convention
/// the caller constructed it with.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum AngleEnvelope {
    /// Look angle (off-nadir, measured at the satellite) in radians.
    Look { min_rad: f64, max_rad: f64 },
    /// Incidence angle (off-vertical, measured at the ground point) in radians.
    Incidence { min_rad: f64, max_rad: f64 },
}

/// Errors from constructing a [`SarPayload`].
#[derive(Debug, Error)]
pub enum SarPayloadError {
    /// Returned when the `min` argument is not strictly less than `max`.
    #[error("invalid angle range: min ({min}°) must be less than max ({max}°)")]
    InvalidAngleRange {
        /// The offending minimum angle in degrees.
        min: f64,
        /// The offending maximum angle in degrees.
        max: f64,
    },
    /// Returned when an angle is outside the valid `[0°, 90°)` range.
    #[error("angle must lie in [0°, 90°), got {0}°")]
    AngleOutOfRange(f64),
}

/// A SAR payload describing a side-looking annular access region.
///
/// Construct via [`SarPayload::with_look_angles`] (look angle at the satellite)
/// or [`SarPayload::with_incidence_angles`] (incidence angle at the ground point).
/// The chosen convention is preserved internally and converted at evaluation
/// time using the actual instantaneous altitude.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SarPayload {
    envelope: AngleEnvelope,
    side: LookSide,
}

impl SarPayload {
    /// Constructs a SAR payload from a look-angle envelope (off-nadir, at the satellite).
    pub fn with_look_angles(
        min: Angle,
        max: Angle,
        side: LookSide,
    ) -> Result<Self, SarPayloadError> {
        let (min_rad, max_rad) = validate_range(min, max)?;
        Ok(Self {
            envelope: AngleEnvelope::Look { min_rad, max_rad },
            side,
        })
    }

    /// Constructs a SAR payload from an incidence-angle envelope (off-vertical, at the ground).
    pub fn with_incidence_angles(
        min: Angle,
        max: Angle,
        side: LookSide,
    ) -> Result<Self, SarPayloadError> {
        let (min_rad, max_rad) = validate_range(min, max)?;
        Ok(Self {
            envelope: AngleEnvelope::Incidence { min_rad, max_rad },
            side,
        })
    }

    /// Returns the configured looking side.
    pub fn side(&self) -> LookSide {
        self.side
    }
}

fn validate_range(min: Angle, max: Angle) -> Result<(f64, f64), SarPayloadError> {
    let min_deg = min.to_degrees();
    let max_deg = max.to_degrees();
    for &deg in &[min_deg, max_deg] {
        if !(0.0..90.0).contains(&deg) {
            return Err(SarPayloadError::AngleOutOfRange(deg));
        }
    }
    if min_deg >= max_deg {
        return Err(SarPayloadError::InvalidAngleRange {
            min: min_deg,
            max: max_deg,
        });
    }
    Ok((min.to_radians(), max.to_radians()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_look_angles_valid() {
        let p = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap();
        assert_eq!(p.side(), LookSide::Right);
    }

    #[test]
    fn with_incidence_angles_valid() {
        let p = SarPayload::with_incidence_angles(
            Angle::degrees(22.0),
            Angle::degrees(46.0),
            LookSide::Either,
        )
        .unwrap();
        assert_eq!(p.side(), LookSide::Either);
    }

    #[test]
    fn rejects_inverted_range() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(45.0),
            Angle::degrees(20.0),
            LookSide::Left,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::InvalidAngleRange { .. }));
    }

    #[test]
    fn rejects_negative_angle() {
        let err = SarPayload::with_incidence_angles(
            Angle::degrees(-5.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::AngleOutOfRange(_)));
    }

    #[test]
    fn rejects_at_or_above_90() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(90.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::AngleOutOfRange(_)));
    }

    #[test]
    fn rejects_equal_min_and_max() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(30.0),
            Angle::degrees(30.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::InvalidAngleRange { .. }));
    }
}
