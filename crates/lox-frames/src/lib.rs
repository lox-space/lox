// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Reference frames, rotations, and coordinate transformations.

pub mod frames;
pub mod iers;
pub mod providers;
pub mod rotations;
pub mod traits;

pub use frames::{Cirf, Frame, Iau, Icrf, Itrf, J2000, Teme, Tirf, UnknownFrameError};
pub use traits::{
    BodyFixed, NonBodyFixedFrameError, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame,
    TryBodyFixed, TryQuasiInertial,
};
