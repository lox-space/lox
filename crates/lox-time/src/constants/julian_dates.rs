/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::continuous::BaseTime;
use crate::Subsecond;

pub const SECONDS_BETWEEN_JD_AND_J2000: i64 = 211813488000;
pub const SECONDS_BETWEEN_MJD_AND_J2000: i64 = 4453444800;
pub const SECONDS_BETWEEN_J1950_AND_J2000: i64 = 1577880000;

pub const J0: BaseTime = BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000, Subsecond(0.0));
