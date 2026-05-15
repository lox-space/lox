// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fuzz the KVN grammar/parser at the AST level: no-panic property.
//! Any UTF-8 input should yield either an `Ok(KvnDocument)` or an
//! `Err(KvnError)` without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = lox_odm::kvn::parse(s);
    }
});
