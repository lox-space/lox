// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use thiserror::Error;

pub(crate) mod private {
    pub struct Internal;
}

pub trait ReferenceFrame {
    fn name(&self) -> String;
    fn abbreviation(&self) -> String;
    fn is_rotating(&self) -> bool;
    #[doc(hidden)]
    fn frame_id(&self, _: private::Internal) -> Option<i32> {
        None
    }
}

pub fn frame_id(frame: impl ReferenceFrame) -> Option<i32> {
    frame.frame_id(private::Internal)
}

pub trait QuasiInertial: ReferenceFrame {}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{0} is not a quasi-inertial frame")]
pub struct NonQuasiInertialFrameError(pub String);

pub trait TryQuasiInertial: ReferenceFrame {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError>;
}

impl<T: QuasiInertial> TryQuasiInertial for T {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        Ok(())
    }
}

pub trait BodyFixed: ReferenceFrame {}

#[derive(Clone, Debug, Error)]
#[error("{0} is not a body-fixed frame")]
pub struct NonBodyFixedFrameError(pub String);

pub trait TryBodyFixed: ReferenceFrame {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError>;
}

impl<T: BodyFixed> TryBodyFixed for T {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        Ok(())
    }
}
