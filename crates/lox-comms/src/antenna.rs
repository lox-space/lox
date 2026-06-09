// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Antenna types and the [`AntennaGain`] trait.

use lox_core::glam::DVec3;
use lox_core::units::{Angle, Decibel, Frequency};
use thiserror::Error;

use crate::pattern::AntennaPattern;

/// Right-handed antenna coordinate frame expressed in a parent frame.
///
/// The frame follows the same polar convention used by common antenna-pattern
/// formats: `z` is the antenna boresight (`theta = 0`), `x` is the `phi = 0`
/// reference direction, and `y` is the positive-`phi` direction. The three axes
/// are always stored as an orthonormal right-handed basis.
///
/// Use [`Self::from_boresight_and_reference`] when the antenna mounting is known
/// by a boresight direction and a reference direction for the `phi = 0` plane.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(into = "AntennaFrameRepr", try_from = "AntennaFrameRepr")
)]
pub struct AntennaFrame {
    x: DVec3,
    y: DVec3,
    z: DVec3,
}

/// Serde wire format for [`AntennaFrame`]: stores the inputs to
/// [`AntennaFrame::from_boresight_and_reference`] so deserialization is forced
/// through the validated constructor and the orthonormal/right-handed
/// invariant cannot be violated.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct AntennaFrameRepr {
    boresight: DVec3,
    reference: DVec3,
}

#[cfg(feature = "serde")]
impl From<AntennaFrame> for AntennaFrameRepr {
    fn from(frame: AntennaFrame) -> Self {
        // `x` is already perpendicular to `z` and unit length, so it is a
        // valid reference that reconstructs the same basis.
        Self {
            boresight: frame.z,
            reference: frame.x,
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<AntennaFrameRepr> for AntennaFrame {
    type Error = AntennaFrameError;

    fn try_from(repr: AntennaFrameRepr) -> Result<Self, Self::Error> {
        AntennaFrame::from_boresight_and_reference(repr.boresight, repr.reference)
    }
}

/// Minimum value of `|reference_perp| / |reference|` accepted by
/// [`AntennaFrame::from_boresight_and_reference`]. References whose angle to
/// the boresight is smaller than `asin(MIN_REFERENCE_SIN)` are rejected as
/// numerically degenerate, even though their perpendicular component would
/// pass `try_normalize`.
const MIN_REFERENCE_SIN: f64 = 1e-6;

impl AntennaFrame {
    /// Creates an identity antenna frame that is aligned with the parent frame.
    pub fn identity() -> Self {
        Self {
            x: DVec3::X,
            y: DVec3::Y,
            z: DVec3::Z,
        }
    }

    /// Creates an antenna frame from boresight and reference direction.
    ///
    /// `boresight` defines the antenna-frame `+Z` axis. `reference` defines the
    /// `+X` axis after projection into the plane perpendicular to `boresight`;
    /// it therefore fixes the `phi = 0` cut of the antenna pattern. The `+Y`
    /// axis is chosen so that `x × y = z`.
    ///
    /// Returns [`AntennaFrameError::InvalidBoresight`] when `boresight` cannot
    /// be normalized, and [`AntennaFrameError::InvalidReference`] when
    /// `reference` is zero, non-finite, or parallel to `boresight`.
    pub fn from_boresight_and_reference(
        boresight: DVec3,
        reference: DVec3,
    ) -> Result<Self, AntennaFrameError> {
        let z = boresight
            .try_normalize()
            .ok_or(AntennaFrameError::InvalidBoresight(boresight))?;

        let ref_length_sq = reference.length_squared();
        if !ref_length_sq.is_finite() || ref_length_sq == 0.0 {
            return Err(AntennaFrameError::InvalidReference(reference));
        }

        // Remove the component of reference along boresight.
        let x_raw = reference - z * reference.dot(z);
        // Require the perpendicular component to be a meaningful fraction of
        // the reference itself; otherwise the phi=0 direction is dominated by
        // floating-point noise.
        if x_raw.length_squared() < MIN_REFERENCE_SIN * MIN_REFERENCE_SIN * ref_length_sq {
            return Err(AntennaFrameError::InvalidReference(reference));
        }
        let x = x_raw
            .try_normalize()
            .ok_or(AntennaFrameError::InvalidReference(reference))?;

        let y = z.cross(x);

        Ok(Self { x, y, z })
    }

    /// Transforms a vector from the parent frame into the antenna frame.
    ///
    /// The result is the vector's components along the antenna `+X`, `+Y`,
    /// and `+Z` axes. Pattern code can call this on a line-of-sight vector
    /// to obtain `(theta, phi)` without depending on the axis convention.
    pub fn to_local(&self, v: DVec3) -> DVec3 {
        DVec3::new(v.dot(self.x), v.dot(self.y), v.dot(self.z))
    }

    /// Transforms a vector from the antenna frame back into the parent frame.
    pub fn from_local(&self, v: DVec3) -> DVec3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// Returns the antenna-frame `+X` axis in the parent frame.
    ///
    /// This axis defines the `phi = 0` reference direction.
    pub fn x(&self) -> DVec3 {
        self.x
    }

    /// Returns the antenna-frame `+Y` axis in the parent frame.
    ///
    /// This axis defines the positive-`phi` direction.
    pub fn y(&self) -> DVec3 {
        self.y
    }

    /// Returns the antenna-frame `+Z` axis in the parent frame.
    ///
    /// This axis is the antenna boresight.
    pub fn z(&self) -> DVec3 {
        self.z
    }
}

impl Default for AntennaFrame {
    fn default() -> Self {
        Self::identity()
    }
}

/// Errors produced while constructing an [`AntennaFrame`].
#[derive(Debug, Error)]
pub enum AntennaFrameError {
    /// The boresight vector is zero-length or non-finite.
    #[error("invalid boresight {0:?}: vector must be nonzero and finite")]
    InvalidBoresight(DVec3),
    /// The reference vector is zero-length, non-finite, or parallel to boresight.
    #[error(
        "invalid reference {0:?}: vector must be nonzero, finite, and not parallel to boresight"
    )]
    InvalidReference(DVec3),
}

/// Trait for types that can compute antenna gain.
pub trait AntennaGain {
    /// Returns the antenna gain in dBi at the given frequency and pattern angles.
    ///
    /// `theta` is the polar angle from boresight and `phi` is the azimuth about
    /// boresight measured from the antenna-frame `+X` axis toward `+Y`.
    fn gain(&self, frequency: Frequency, theta: Angle, phi: Angle) -> Decibel;
}

/// Antenna with angle-independent gain.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstantAntenna {
    /// Peak gain in dBi.
    pub gain: Decibel,
}

impl AntennaGain for ConstantAntenna {
    fn gain(&self, _frequency: Frequency, _theta: Angle, _phi: Angle) -> Decibel {
        self.gain
    }
}

/// An antenna with a physics-based gain pattern and a boresight vector.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternedAntenna {
    /// The gain pattern model.
    pub pattern: AntennaPattern,
    /// Boresight direction vector (unit vector in the antenna's local frame).
    pub boresight: DVec3,
}

impl AntennaGain for PatternedAntenna {
    fn gain(&self, frequency: Frequency, theta: Angle, phi: Angle) -> Decibel {
        self.pattern.gain(frequency, theta, phi)
    }
}

impl PatternedAntenna {
    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        self.pattern.peak_gain(frequency)
    }
}

/// An antenna, either constant-gain or pattern-based.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum Antenna {
    /// Constant-gain antenna.
    Constant(ConstantAntenna),
    /// Pattern-based antenna with boresight direction.
    Patterned(PatternedAntenna),
}

impl AntennaGain for Antenna {
    fn gain(&self, frequency: Frequency, theta: Angle, phi: Angle) -> Decibel {
        match self {
            Antenna::Constant(a) => a.gain(frequency, theta, phi),
            Antenna::Patterned(a) => a.gain(frequency, theta, phi),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::glam::DVec3;
    use lox_core::units::{Angle, Decibel, DecibelUnits, Distance, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::pattern::{AntennaPattern, ParabolicPattern};

    use super::*;

    fn parabolic() -> PatternedAntenna {
        PatternedAntenna {
            pattern: AntennaPattern::Parabolic(ParabolicPattern::new(Distance::meters(0.98), 0.45)),
            boresight: DVec3::Z,
        }
    }

    #[test]
    fn test_antenna_frame_identity_axes() {
        let frame = AntennaFrame::identity();

        assert_approx_eq!(frame.x(), DVec3::X, atol <= 1e-12);
        assert_approx_eq!(frame.y(), DVec3::Y, atol <= 1e-12);
        assert_approx_eq!(frame.z(), DVec3::Z, atol <= 1e-12);
        assert_approx_eq!(frame.x().cross(frame.y()), frame.z(), atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_default_is_identity() {
        assert_eq!(AntennaFrame::default(), AntennaFrame::identity());
    }

    #[test]
    fn test_antenna_frame_from_boresight_and_reference_identity() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::Z, DVec3::X).unwrap();

        assert_approx_eq!(frame.x(), DVec3::X, atol <= 1e-12);
        assert_approx_eq!(frame.y(), DVec3::Y, atol <= 1e-12);
        assert_approx_eq!(frame.z(), DVec3::Z, atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_projects_reference_onto_normal_plane() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::Z, DVec3::new(2.0, 0.0, 4.0))
            .unwrap();

        assert_approx_eq!(frame.x(), DVec3::X, atol <= 1e-12);
        assert_approx_eq!(frame.y(), DVec3::Y, atol <= 1e-12);
        assert_approx_eq!(frame.z(), DVec3::Z, atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_rotated_mount_is_right_handed_and_orthonormal() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();

        assert_approx_eq!(frame.x(), DVec3::Z, atol <= 1e-12);
        assert_approx_eq!(frame.y(), -DVec3::Y, atol <= 1e-12);
        assert_approx_eq!(frame.z(), DVec3::X, atol <= 1e-12);

        assert_approx_eq!(frame.x().length(), 1.0, atol <= 1e-12);
        assert_approx_eq!(frame.y().length(), 1.0, atol <= 1e-12);
        assert_approx_eq!(frame.z().length(), 1.0, atol <= 1e-12);
        assert_approx_eq!(frame.x().dot(frame.y()), 0.0, atol <= 1e-12);
        assert_approx_eq!(frame.x().dot(frame.z()), 0.0, atol <= 1e-12);
        assert_approx_eq!(frame.y().dot(frame.z()), 0.0, atol <= 1e-12);
        assert_approx_eq!(frame.x().cross(frame.y()), frame.z(), atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_rejects_zero_boresight() {
        let err = AntennaFrame::from_boresight_and_reference(DVec3::ZERO, DVec3::X).unwrap_err();

        assert!(matches!(err, AntennaFrameError::InvalidBoresight(_)));
        assert!(err.to_string().contains("invalid boresight"));
    }

    #[test]
    fn test_antenna_frame_rejects_parallel_reference() {
        let err = AntennaFrame::from_boresight_and_reference(DVec3::Z, DVec3::Z).unwrap_err();

        assert!(matches!(err, AntennaFrameError::InvalidReference(_)));
        assert!(err.to_string().contains("invalid reference"));
    }

    #[test]
    fn test_antenna_frame_rejects_non_finite_reference() {
        let err =
            AntennaFrame::from_boresight_and_reference(DVec3::Z, DVec3::new(f64::NAN, 0.0, 0.0))
                .unwrap_err();

        assert!(matches!(err, AntennaFrameError::InvalidReference(_)));
    }

    #[test]
    fn test_antenna_frame_rejects_near_parallel_reference() {
        // Reference within 1e-9 rad of boresight: try_normalize would accept
        // the projected component, but it is dominated by floating-point noise.
        let reference = DVec3::Z + DVec3::X * 1e-9;
        let err = AntennaFrame::from_boresight_and_reference(DVec3::Z, reference).unwrap_err();

        assert!(matches!(err, AntennaFrameError::InvalidReference(_)));
    }

    #[test]
    fn test_antenna_frame_to_local_round_trip() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();
        let v = DVec3::new(0.3, -0.7, 0.5);

        let round_trip = frame.from_local(frame.to_local(v));

        assert_approx_eq!(round_trip, v, atol <= 1e-12);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_antenna_frame_serde_round_trip_preserves_basis() {
        let frame = AntennaFrame::from_boresight_and_reference(
            DVec3::new(1.0, 2.0, 3.0),
            DVec3::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let json = serde_json::to_string(&frame).unwrap();
        let round_trip: AntennaFrame = serde_json::from_str(&json).unwrap();

        assert_approx_eq!(round_trip.x(), frame.x(), atol <= 1e-12);
        assert_approx_eq!(round_trip.y(), frame.y(), atol <= 1e-12);
        assert_approx_eq!(round_trip.z(), frame.z(), atol <= 1e-12);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_antenna_frame_serde_rejects_invalid_payload() {
        // A payload with a zero boresight must be rejected at deserialization
        // time, not silently produce a degenerate frame.
        let bad = r#"{"boresight":[0.0,0.0,0.0],"reference":[1.0,0.0,0.0]}"#;
        assert!(serde_json::from_str::<AntennaFrame>(bad).is_err());
    }

    #[test]
    fn test_antenna_frame_to_local_maps_boresight_to_z() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();

        // The parent-frame boresight vector should land on +Z in the antenna
        // frame, regardless of how the antenna is mounted.
        assert_approx_eq!(frame.to_local(DVec3::X), DVec3::Z, atol <= 1e-12);
    }

    #[test]
    fn test_constant_antenna_gain_dispatch() {
        let a = ConstantAntenna { gain: 10.0.db() };
        let g = a.gain(29.0.ghz(), Angle::radians(0.0), Angle::radians(0.0));
        assert_approx_eq!(g.as_f64(), 10.0, atol <= 1e-10);
    }

    #[test]
    fn test_patterned_antenna_gain() {
        let a = parabolic();
        let f = 29.0.ghz();
        let peak = a.peak_gain(f);
        let on_axis = a.gain(f, Angle::radians(0.0), Angle::radians(0.0));
        // On-axis gain equals peak gain
        assert_approx_eq!(on_axis.as_f64(), peak.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_antenna_enum_constant_dispatch() {
        let a = Antenna::Constant(ConstantAntenna {
            gain: Decibel::new(20.0),
        });
        let g = a.gain(29.0.ghz(), Angle::radians(0.0), Angle::radians(0.0));
        assert_approx_eq!(g.as_f64(), 20.0, atol <= 1e-10);
    }

    #[test]
    fn test_antenna_enum_patterned_dispatch() {
        let a = Antenna::Patterned(parabolic());
        let f = 29.0.ghz();
        let on_axis = a.gain(f, Angle::radians(0.0), Angle::radians(0.0));
        assert!(on_axis.as_f64() > 40.0);
    }
}
