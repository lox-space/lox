// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Reference frames, rotations, and coordinate transformations.

/// Reference frame types: the zero-sized marker types and the [`frames::Frame`]
/// enum covering the same set at runtime.
pub mod frames;
/// IAU body-fixed frame rotation calculations.
pub mod iau;
/// IERS reference systems, conventions, and Earth orientation sub-models.
pub mod iers;
/// Default rotation provider (no EOP data).
pub mod providers;
/// Rotation matrices, the [`rotations::TryRotation`] trait, and [`rotations::RotationProvider`].
pub mod rotations;
/// Core reference frame traits.
pub mod traits;

pub use frames::{Cirf, Frame, Iau, Icrf, Itrf, J2000, Teme, Tirf, UnknownFrameError};
pub use traits::{
    BodyFixed, NonBodyFixedFrameError, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame,
    TryBodyFixed, TryQuasiInertial,
};
