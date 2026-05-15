// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fuzz the format detector + top-level auto-detecting `read_opm`.
//! No-panic property; format detection must never panic on any UTF-8.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = lox_odm::detect_format(s);
        let _ = lox_odm::read_opm(s);
        let _ = lox_odm::read_oem(s);
        let _ = lox_odm::read_omm(s);
    }
});
