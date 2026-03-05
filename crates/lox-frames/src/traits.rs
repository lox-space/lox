// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use thiserror::Error;

pub(crate) mod private {
    /// Internal token to seal frame_id.
    pub struct Internal;
}

/// A reference frame with a human-readable name and abbreviation.
pub trait ReferenceFrame {
    /// Returns the full name of the frame (e.g. "International Celestial Reference Frame").
    fn name(&self) -> String;
    /// Returns the abbreviated name (e.g. "ICRF").
    fn abbreviation(&self) -> String;
    #[doc(hidden)]
    fn frame_id(&self, _: private::Internal) -> Option<usize> {
        None
    }
}

/// Returns the internal numeric frame identifier, if assigned.
pub fn frame_id(frame: &impl ReferenceFrame) -> Option<usize> {
    frame.frame_id(private::Internal)
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
