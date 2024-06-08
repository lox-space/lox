/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![cfg(test)]

use std::{path::PathBuf, sync::OnceLock};

use crate::{ut1::DeltaUt1Tai, utc::leap_seconds::BuiltinLeapSeconds};

/// Returns a [PathBuf] to the test fixture directory.
pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}

/// Returns a [DeltaUt1Tai] loaded from the default IERS finals CSV located in the text fixture
/// directory.
pub fn delta_ut1_tai() -> &'static DeltaUt1Tai {
    static PROVIDER: OnceLock<DeltaUt1Tai> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        DeltaUt1Tai::new(data_dir().join("finals2000A.all.csv"), &BuiltinLeapSeconds).unwrap()
    })
}
