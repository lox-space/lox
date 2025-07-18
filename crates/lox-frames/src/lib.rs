/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Module declarations
pub mod dynamic;
pub mod frames;
pub mod traits;
pub mod transformations;

#[cfg(feature = "python")]
pub mod python;

// Re-export commonly used types
pub use dynamic::{DynFrame, UnknownFrameError};
pub use frames::{Icrf, Cirf, Tirf, Itrf, Iau};
pub use traits::{
    ReferenceFrame, QuasiInertial, BodyFixed, 
    TryQuasiInertial, TryBodyFixed, TryRotateTo,
    NonQuasiInertialFrameError, NonBodyFixedFrameError
};