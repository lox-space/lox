// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{CoordinateOrigin, NaifId, UndefinedOriginPropertyError};
use lox_core::coords::Ellipsoid;
use thiserror::Error;

use crate::iers::ReferenceSystem;

pub(crate) mod private {
    /// Internal token to seal `frame_key`.
    pub struct Internal;
}

/// Structural identity of a reference frame, used to detect when two frames —
/// whether expressed concretely or as a [`Frame`](crate::Frame) — are the
/// same, without rotating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKey {
    /// International Celestial Reference Frame.
    Icrf,
    /// J2000 Mean Equator and Equinox.
    J2000,
    /// Celestial Intermediate Reference Frame.
    Cirf,
    /// Terrestrial Intermediate Reference Frame.
    Tirf,
    /// International Terrestrial Reference Frame.
    Itrf,
    /// True Equator Mean Equinox.
    Teme,
    /// Mean of Date for the given IERS convention.
    Mod(ReferenceSystem),
    /// True of Date for the given IERS convention.
    Tod(ReferenceSystem),
    /// Pseudo-Earth Fixed for the given IERS convention.
    Pef(ReferenceSystem),
    /// IAU body-fixed frame for the given body.
    Iau(NaifId),
}

/// A reference frame with a human-readable name and abbreviation.
pub trait ReferenceFrame {
    /// Returns the full name of the frame (e.g. "International Celestial Reference Frame").
    fn name(&self) -> String;
    /// Returns the abbreviated name (e.g. "ICRF").
    fn abbreviation(&self) -> String;
    #[doc(hidden)]
    fn frame_key(&self, _: private::Internal) -> Option<FrameKey> {
        None
    }
}

/// Returns the frame's identity key, if it has one.
pub fn frame_key(frame: &impl ReferenceFrame) -> Option<FrameKey> {
    frame.frame_key(private::Internal)
}

/// Marker trait for quasi-inertial reference frames.
pub trait QuasiInertial: ReferenceFrame {}

/// The frame is not quasi-inertial.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{0} is not a quasi-inertial frame")]
pub struct NonQuasiInertialFrameError(pub String);

/// Fallible check for quasi-inertial frames (used by dynamic dispatch).
pub trait TryQuasiInertial: ReferenceFrame {
    /// Returns `Ok(())` if the frame is quasi-inertial.
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError>;
}

impl<T: QuasiInertial> TryQuasiInertial for T {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        Ok(())
    }
}

/// Marker trait for body-fixed reference frames.
pub trait BodyFixed: ReferenceFrame {
    /// The coordinate origin (central body) of the body-fixed frame.
    type Origin: CoordinateOrigin + Copy;

    /// Returns the coordinate origin (central body) of the body-fixed frame.
    fn origin(&self) -> Self::Origin;
}

/// The frame is not body-fixed.
#[derive(Clone, Debug, Error)]
#[error("{0} is not a body-fixed frame")]
pub struct NonBodyFixedFrameError(pub String);

/// Fallible check for body-fixed frames (used by dynamic dispatch).
pub trait TryBodyFixed: ReferenceFrame {
    /// The coordinate origin (central body) of the body-fixed frame.
    type Origin: CoordinateOrigin + Copy;

    /// Returns `Ok(())` if the frame is body-fixed.
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError>;

    /// Returns the coordinate origin (central body) of the body-fixed frame.
    fn try_origin(&self) -> Result<Self::Origin, NonBodyFixedFrameError>;
}

impl<T: BodyFixed> TryBodyFixed for T {
    type Origin = T::Origin;

    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        Ok(())
    }

    fn try_origin(&self) -> Result<Self::Origin, NonBodyFixedFrameError> {
        Ok(self.origin())
    }
}

/// A body-fixed frame with a conventional reference ellipsoid.
///
/// The ellipsoid is the one conventionally paired with the frame rather than an
/// intrinsic property of its origin: the terrestrial frames
/// ([`Itrf`](crate::Itrf), [`Tirf`](crate::Tirf) and [`Pef`](crate::Pef)) are
/// paired with GRS80, while [`Iau`](crate::Iau) frames use their body's own
/// spheroid. Callers needing a different datum supply the ellipsoid explicitly.
pub trait ReferenceEllipsoid: BodyFixed {
    /// Returns the reference ellipsoid conventionally paired with this frame.
    fn reference_ellipsoid(&self) -> Ellipsoid;
}

/// Fallible accessor for a frame's reference ellipsoid (used by dynamic dispatch).
pub trait TryReferenceEllipsoid: TryBodyFixed {
    /// Returns the reference ellipsoid conventionally paired with this frame,
    /// or an error if the frame is not body-fixed or its origin is not a spheroid.
    fn try_reference_ellipsoid(&self) -> Result<Ellipsoid, UndefinedReferenceEllipsoidError>;
}

impl<T> TryReferenceEllipsoid for T
where
    T: ReferenceEllipsoid,
{
    fn try_reference_ellipsoid(&self) -> Result<Ellipsoid, UndefinedReferenceEllipsoidError> {
        Ok(self.reference_ellipsoid())
    }
}

/// Error returned when a frame has no reference ellipsoid.
#[derive(Debug, thiserror::Error)]
pub enum UndefinedReferenceEllipsoidError {
    /// The frame is not body-fixed, so no datum is associated with it.
    #[error(transparent)]
    NotBodyFixed(#[from] NonBodyFixedFrameError),
    /// The frame's origin has no spheroid, as for a triaxial body.
    #[error(transparent)]
    UndefinedSpheroid(#[from] UndefinedOriginPropertyError),
}
