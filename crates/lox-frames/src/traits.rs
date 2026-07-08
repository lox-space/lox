// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::NaifId;
use thiserror::Error;

use crate::iers::ReferenceSystem;

pub(crate) mod private {
    /// Internal token to seal `frame_key`.
    pub struct Internal;
}

/// Structural identity of a reference frame, used to detect when two frames —
/// whether expressed concretely or as a [`DynFrame`](crate::DynFrame) — are the
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
pub trait BodyFixed: ReferenceFrame {}

/// The frame is not body-fixed.
#[derive(Clone, Debug, Error)]
#[error("{0} is not a body-fixed frame")]
pub struct NonBodyFixedFrameError(pub String);

/// Fallible check for body-fixed frames (used by dynamic dispatch).
pub trait TryBodyFixed: ReferenceFrame {
    /// Returns `Ok(())` if the frame is body-fixed.
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError>;
}

impl<T: BodyFixed> TryBodyFixed for T {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        Ok(())
    }
}
