/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod dynamic;
pub mod frames;
pub mod traits;
pub mod transformations;

pub use dynamic::{DynFrame, UnknownFrameError};
pub use frames::{Cirf, Iau, Icrf, Itrf, Tirf};
pub use traits::{
    BodyFixed, NonBodyFixedFrameError, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame,
    TryBodyFixed, TryQuasiInertial, TryRotateTo,
};
