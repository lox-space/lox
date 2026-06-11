// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Antenna types and the [`AntennaGain`] trait.

use lox_core::glam::DVec3;
use lox_core::units::{Angle, Decibel, Distance, Frequency};
use thiserror::Error;

use crate::pattern::{AntennaPattern, DipolePattern, GaussianPattern, ParabolicPattern};

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

    /// Returns the pattern angles for a parent-frame direction vector.
    ///
    /// The input direction is normalized before conversion. The returned
    /// `theta` is the polar angle from boresight, and `phi` is the azimuth
    /// from the antenna-frame `+X` axis toward `+Y`.
    pub fn angles_for(&self, direction: DVec3) -> Result<(Angle, Angle), AntennaFrameError> {
        let direction = direction
            .try_normalize()
            .ok_or(AntennaFrameError::InvalidDirection(direction))?;
        let local = self.to_local(direction);
        let theta = Angle::radians(local.z.clamp(-1.0, 1.0).acos());
        let phi = Angle::radians(local.y.atan2(local.x));

        Ok((theta, phi))
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
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AntennaFrameError {
    /// The boresight vector is zero-length or non-finite.
    #[error("invalid boresight {0:?}: vector must be nonzero and finite")]
    InvalidBoresight(DVec3),
    /// The reference vector is zero-length, non-finite, or parallel to boresight.
    #[error(
        "invalid reference {0:?}: vector must be nonzero, finite, and not parallel to boresight"
    )]
    InvalidReference(DVec3),
    /// The direction vector is zero-length or non-finite.
    #[error("invalid direction {0:?}: vector must be nonzero and finite")]
    InvalidDirection(DVec3),
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ConstantAntennaRepr")
)]
pub struct ConstantAntenna {
    gain: Decibel,
}

/// Serde wire format for [`ConstantAntenna`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ConstantAntennaRepr {
    gain: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<ConstantAntennaRepr> for ConstantAntenna {
    type Error = crate::error::NonPhysicalError;

    fn try_from(repr: ConstantAntennaRepr) -> Result<Self, Self::Error> {
        ConstantAntenna::new(repr.gain)
    }
}

impl ConstantAntenna {
    /// Creates a new constant-gain antenna.
    ///
    /// Rejects a non-finite gain.
    pub fn new(gain: Decibel) -> Result<Self, crate::error::NonPhysicalError> {
        crate::error::NonPhysicalError::check_finite("antenna gain [dBi]", gain.as_f64())?;
        Ok(Self { gain })
    }

    /// Returns the peak gain in dBi.
    pub fn peak_gain(&self) -> Decibel {
        self.gain
    }
}

impl AntennaGain for ConstantAntenna {
    fn gain(&self, _frequency: Frequency, _theta: Angle, _phi: Angle) -> Decibel {
        self.gain
    }
}

/// An antenna with a physics-based gain pattern and antenna frame.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternedAntenna {
    /// The gain pattern model.
    pub pattern: AntennaPattern,
    /// Orientation of the antenna pattern in its parent frame.
    pub frame: AntennaFrame,
}

impl PatternedAntenna {
    /// Creates a patterned antenna with an identity frame.
    pub fn new(pattern: impl Into<AntennaPattern>) -> Self {
        Self {
            pattern: pattern.into(),
            frame: AntennaFrame::identity(),
        }
    }

    /// Sets the orientation of the antenna pattern in its parent frame.
    pub fn with_frame(mut self, frame: AntennaFrame) -> Self {
        self.frame = frame;
        self
    }
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

    /// Returns the gain in dBi toward a parent-frame direction vector.
    ///
    /// The direction is converted into pattern angles using [`Self::frame`].
    pub fn gain_toward(
        &self,
        frequency: Frequency,
        direction: DVec3,
    ) -> Result<Decibel, AntennaFrameError> {
        let (theta, phi) = self.frame.angles_for(direction)?;
        Ok(self.gain(frequency, theta, phi))
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
    /// Pattern-based antenna with antenna frame.
    Patterned(PatternedAntenna),
}

impl From<ConstantAntenna> for Antenna {
    fn from(antenna: ConstantAntenna) -> Self {
        Antenna::Constant(antenna)
    }
}

impl From<PatternedAntenna> for Antenna {
    fn from(antenna: PatternedAntenna) -> Self {
        Antenna::Patterned(antenna)
    }
}

impl Antenna {
    /// Creates a constant-gain antenna.
    ///
    /// Rejects a non-finite gain.
    pub fn constant(gain: Decibel) -> Result<Self, crate::error::NonPhysicalError> {
        Ok(ConstantAntenna::new(gain)?.into())
    }

    /// Creates a parabolic-dish antenna with an identity frame.
    ///
    /// Rejects a non-finite or non-positive diameter and an aperture
    /// efficiency outside (0, 1]. Use
    /// [`PatternedAntenna::with_frame`] to orient the pattern.
    pub fn parabolic(
        diameter: Distance,
        efficiency: f64,
    ) -> Result<Self, crate::error::NonPhysicalError> {
        Ok(PatternedAntenna::new(ParabolicPattern::new(diameter, efficiency)?).into())
    }

    /// Creates a Gaussian-pattern antenna with an identity frame.
    ///
    /// Rejects a non-finite or non-positive diameter and an aperture
    /// efficiency outside (0, 1].
    pub fn gaussian(
        diameter: Distance,
        efficiency: f64,
    ) -> Result<Self, crate::error::NonPhysicalError> {
        Ok(PatternedAntenna::new(GaussianPattern::new(diameter, efficiency)?).into())
    }

    /// Creates a dipole antenna with an identity frame.
    ///
    /// Rejects a non-finite or non-positive length.
    pub fn dipole(length: Distance) -> Result<Self, crate::error::NonPhysicalError> {
        Ok(PatternedAntenna::new(DipolePattern::new(length)?).into())
    }
}

impl AntennaGain for Antenna {
    fn gain(&self, frequency: Frequency, theta: Angle, phi: Angle) -> Decibel {
        match self {
            Antenna::Constant(a) => a.gain(frequency, theta, phi),
            Antenna::Patterned(a) => a.gain(frequency, theta, phi),
        }
    }
}

impl Antenna {
    /// Returns the gain in dBi toward a parent-frame direction vector.
    ///
    /// The direction is converted into pattern angles via
    /// [`Self::pattern_angles`]; for [`Antenna::Constant`] the resulting
    /// angles do not affect the gain but the direction is still validated.
    pub fn gain_toward(
        &self,
        frequency: Frequency,
        direction: DVec3,
    ) -> Result<Decibel, AntennaFrameError> {
        let (theta, phi) = self.pattern_angles(direction)?;
        Ok(self.gain(frequency, theta, phi))
    }

    /// Returns the pattern angles for a parent-frame direction vector.
    ///
    /// For [`Antenna::Patterned`] the direction is converted using the antenna
    /// frame. [`Antenna::Constant`] has no frame and its gain is
    /// direction-independent, so zero angles are returned; the direction is
    /// still validated.
    pub fn pattern_angles(&self, direction: DVec3) -> Result<(Angle, Angle), AntennaFrameError> {
        match self {
            Antenna::Constant(_) => {
                direction
                    .try_normalize()
                    .ok_or(AntennaFrameError::InvalidDirection(direction))?;
                Ok((Angle::ZERO, Angle::ZERO))
            }
            Antenna::Patterned(a) => a.frame.angles_for(direction),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::glam::DVec3;
    use lox_core::units::{Angle, Decibel, DecibelUnits, Distance, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::pattern::{AntennaPattern, DipolePattern, GaussianPattern, ParabolicPattern};

    use super::*;

    fn parabolic() -> PatternedAntenna {
        PatternedAntenna {
            pattern: AntennaPattern::Parabolic(
                ParabolicPattern::new(Distance::meters(0.98), 0.45).unwrap(),
            ),
            frame: AntennaFrame::identity(),
        }
    }

    #[test]
    fn test_frame_local_round_trip() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();
        let v = DVec3::new(0.3, -0.4, 0.5);
        // to_local/from_local are inverse rotations.
        assert_approx_eq!(frame.from_local(frame.to_local(v)), v, atol <= 1e-12);
        // The boresight maps onto the local +Z axis.
        assert_approx_eq!(frame.to_local(DVec3::X), DVec3::Z, atol <= 1e-12);
        // The default frame is the identity.
        assert_approx_eq!(AntennaFrame::default().z(), DVec3::Z, atol <= 1e-12);
    }

    #[test]
    fn test_antenna_gain_toward_constant_validates_direction() {
        let antenna = Antenna::constant(Decibel::new(30.0)).unwrap();
        // Constant gain is direction-independent...
        let gain = antenna
            .gain_toward(Frequency::hertz(29e9), DVec3::new(0.0, 1.0, 0.0))
            .unwrap();
        assert_approx_eq!(gain.as_f64(), 30.0, atol <= 1e-12);
        // ...but the direction is still validated.
        assert!(
            antenna
                .gain_toward(Frequency::hertz(29e9), DVec3::ZERO)
                .is_err()
        );
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
    fn test_antenna_frame_angles_for_identity() {
        let frame = AntennaFrame::identity();

        let (theta, phi) = frame.angles_for(DVec3::X).unwrap();
        assert_approx_eq!(
            theta.to_radians(),
            std::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(phi.to_radians(), 0.0, atol <= 1e-12);

        let (theta, phi) = frame.angles_for(DVec3::Y).unwrap();
        assert_approx_eq!(
            theta.to_radians(),
            std::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(phi.to_radians(), std::f64::consts::FRAC_PI_2, atol <= 1e-12);

        let (theta, phi) = frame.angles_for(DVec3::Z).unwrap();
        assert_approx_eq!(theta.to_radians(), 0.0, atol <= 1e-12);
        assert_approx_eq!(phi.to_radians(), 0.0, atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_angles_for_rotated_mount() {
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();

        let (theta, phi) = frame.angles_for(DVec3::X).unwrap();
        assert_approx_eq!(theta.to_radians(), 0.0, atol <= 1e-12);
        assert_approx_eq!(phi.to_radians(), 0.0, atol <= 1e-12);

        let (theta, phi) = frame.angles_for(DVec3::Z).unwrap();
        assert_approx_eq!(
            theta.to_radians(),
            std::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(phi.to_radians(), 0.0, atol <= 1e-12);
    }

    #[test]
    fn test_antenna_frame_angles_for_rejects_invalid_direction() {
        let err = AntennaFrame::identity()
            .angles_for(DVec3::ZERO)
            .unwrap_err();

        assert!(matches!(err, AntennaFrameError::InvalidDirection(_)));
        assert!(err.to_string().contains("invalid direction"));
    }

    #[test]
    fn test_antenna_smart_constructors() {
        // One-liners replace the nested enum/struct construction.
        let dish = Antenna::parabolic(Distance::meters(0.98), 0.45).unwrap();
        assert!(matches!(dish, Antenna::Patterned(_)));
        let constant = Antenna::constant(46.0.db()).unwrap();
        assert!(matches!(constant, Antenna::Constant(_)));
        assert!(Antenna::gaussian(Distance::meters(0.98), 0.45).is_ok());
        assert!(Antenna::dipole(Distance::meters(0.0185)).is_ok());
        // Validation flows through unchanged.
        assert!(Antenna::parabolic(Distance::meters(-1.0), 0.45).is_err());

        // Oriented patterns: builder + From conversions.
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();
        let oriented: Antenna =
            PatternedAntenna::new(ParabolicPattern::new(Distance::meters(0.98), 0.45).unwrap())
                .with_frame(frame)
                .into();
        let Antenna::Patterned(antenna) = oriented else {
            panic!("expected patterned antenna");
        };
        assert_eq!(antenna.frame, frame);
    }

    #[test]
    fn test_antenna_from_conversions_and_constant_paths() {
        let constant: Antenna = ConstantAntenna::new(20.0.db()).unwrap().into();
        assert!(matches!(constant, Antenna::Constant(_)));
        // gain_toward ignores the direction for constant antennas...
        let gain = constant.gain_toward(29.0.ghz(), DVec3::Y).unwrap();
        assert_approx_eq!(gain.as_f64(), 20.0, atol <= 1e-12);
        // ...but still validates it.
        assert!(constant.gain_toward(29.0.ghz(), DVec3::ZERO).is_err());
        assert!(constant.pattern_angles(DVec3::ZERO).is_err());

        let patterned: Antenna =
            PatternedAntenna::new(GaussianPattern::new(Distance::meters(0.98), 0.45).unwrap())
                .into();
        assert!(matches!(patterned, Antenna::Patterned(_)));
        let dipole: AntennaPattern = DipolePattern::new(Distance::meters(0.0185)).unwrap().into();
        assert!(matches!(dipole, AntennaPattern::Dipole(_)));

        // Smart constructors evaluate to working gain models.
        let gaussian = Antenna::gaussian(Distance::meters(0.98), 0.45).unwrap();
        let on_axis = gaussian.gain_toward(29.0.ghz(), DVec3::Z).unwrap();
        assert!(on_axis.as_f64() > 40.0);
        assert!(Antenna::constant(Decibel::new(f64::NAN)).is_err());
        assert!(Antenna::gaussian(Distance::meters(0.98), 1.5).is_err());
        assert!(Antenna::dipole(Distance::meters(-1.0)).is_err());
    }

    #[test]
    fn test_constant_antenna_gain_dispatch() {
        let a = ConstantAntenna::new(10.0.db()).unwrap();
        let g = a.gain(29.0.ghz(), Angle::ZERO, Angle::ZERO);
        assert_approx_eq!(g.as_f64(), 10.0, atol <= 1e-10);
    }

    #[test]
    fn test_patterned_antenna_gain() {
        let a = parabolic();
        let f = 29.0.ghz();
        let peak = a.peak_gain(f);
        let on_axis = a.gain(f, Angle::ZERO, Angle::ZERO);
        // On-axis gain equals peak gain
        assert_approx_eq!(on_axis.as_f64(), peak.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_patterned_antenna_gain_toward_uses_frame() {
        let a = PatternedAntenna {
            pattern: AntennaPattern::Parabolic(
                ParabolicPattern::new(Distance::meters(0.98), 0.45).unwrap(),
            ),
            frame: AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap(),
        };
        let f = 29.0.ghz();

        let on_axis = a.gain_toward(f, DVec3::X).unwrap();
        let off_axis = a.gain_toward(f, DVec3::Y).unwrap();

        assert_approx_eq!(on_axis.as_f64(), a.peak_gain(f).as_f64(), atol <= 1e-10);
        assert!(off_axis.as_f64() < on_axis.as_f64());
    }

    #[test]
    fn test_antenna_enum_constant_dispatch() {
        let a = Antenna::Constant(ConstantAntenna::new(Decibel::new(20.0)).unwrap());
        let g = a.gain(29.0.ghz(), Angle::ZERO, Angle::ZERO);
        assert_approx_eq!(g.as_f64(), 20.0, atol <= 1e-10);
    }

    #[test]
    fn test_antenna_enum_patterned_dispatch() {
        let a = Antenna::Patterned(parabolic());
        let f = 29.0.ghz();
        let on_axis = a.gain(f, Angle::ZERO, Angle::ZERO);
        assert!(on_axis.as_f64() > 40.0);
    }

    #[test]
    fn test_antenna_enum_gain_toward_constant_ignores_direction() {
        let a = Antenna::Constant(ConstantAntenna::new(20.0.db()).unwrap());
        let g = a.gain_toward(29.0.ghz(), DVec3::Y).unwrap();
        assert_approx_eq!(g.as_f64(), 20.0, atol <= 1e-10);
    }

    #[test]
    fn test_antenna_enum_gain_toward_patterned_uses_frame() {
        let a = Antenna::Patterned(parabolic());
        let f = 29.0.ghz();
        let on_axis = a.gain_toward(f, DVec3::Z).unwrap();
        let off_axis = a.gain_toward(f, DVec3::X).unwrap();
        assert!(off_axis.as_f64() < on_axis.as_f64());

        let err = a.gain_toward(f, DVec3::ZERO).unwrap_err();
        assert!(matches!(err, AntennaFrameError::InvalidDirection(_)));
    }
}
