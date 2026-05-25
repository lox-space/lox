// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! CORS tower-http layer for the SvelteKit dev server.

use tower_http::cors::{Any, CorsLayer};

/// Permissive CORS allowing any origin (dev-only).
///
/// Connect-Web uses `POST` for unary and server-streaming RPCs and reads
/// the `Connect-Protocol-Version` header, so the layer must allow that
/// custom header in addition to standard preflight headers.
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
