// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Kerolox service implementation. Real `compute_access` lands in Task 7.
//!
//! When wiring streaming responses, verify that `cors::dev_cors` exposes
//! the headers Connect-Web needs to read (`Content-Type`, `Grpc-Status`).
//! `Any` for `expose_headers` covers it; the current layer uses `Any`
//! everywhere except expose_headers — add `.expose_headers(Any)` to
//! `dev_cors()` if the browser sees opaque streaming responses.
