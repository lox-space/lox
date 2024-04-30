/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_time::utc::leap_seconds::LeapSecondsKernel;

#[test]
fn test_lsk_from_file() {
    assert!(LeapSecondsKernel::from_file("../../data/naif0012.tls").is_ok());
}
