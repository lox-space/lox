// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Kerolox SAR constellation sizing compute engine.
//!
//! Library crate exposing internal modules (mapping, aoi, service, bridge)
//! plus a thin start-server entry point that `main.rs` calls. Splitting
//! lib + bin lets the integration tests in `tests/` exercise the service
//! without going through the binary.

pub mod aoi;
pub mod bridge;
pub mod cors;
pub mod mapping;
pub mod service;
