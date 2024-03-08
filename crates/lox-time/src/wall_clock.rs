/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// `WallClock` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait WallClock {
    fn hour(&self) -> i64;
    fn minute(&self) -> i64;
    fn second(&self) -> i64;
    fn millisecond(&self) -> i64;
    fn microsecond(&self) -> i64;
    fn nanosecond(&self) -> i64;
    fn picosecond(&self) -> i64;
    fn femtosecond(&self) -> i64;
}
