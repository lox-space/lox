// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;
use std::sync::OnceLock;

use lox_itur::ItuProvider;

/// Returns the shared ItuProvider for integration tests.
///
/// Bundle resolution order:
///   1. `LOX_ITUR_BUNDLE` env var
///   2. `target/lox-itur-data.npz` (workspace target/)
///
/// Panics with a remediation message if no bundle is found. Skipping silently
/// hides regressions, so we fail loudly.
pub fn provider() -> &'static ItuProvider {
    static P: OnceLock<ItuProvider> = OnceLock::new();
    P.get_or_init(|| {
        let path = std::env::var("LOX_ITUR_BUNDLE")
            .ok()
            .map(PathBuf::from)
            .or_else(default_bundle_path)
            .filter(|p| p.exists())
            .unwrap_or_else(|| {
                panic!(
                    "lox-itur integration tests need lox-itur-data.npz. \
                     Run `pip download --no-deps itur==0.4.0 && \
                     just lox-itur-pack itur-0.4.0-py2.py3-none-any.whl`, \
                     or set LOX_ITUR_BUNDLE to a bundle path."
                )
            });
        ItuProvider::open(path).expect("failed to open lox-itur-data.npz")
    })
}

fn default_bundle_path() -> Option<PathBuf> {
    // CARGO_MANIFEST_DIR = .../crates/lox-itur ; workspace target/ is two levels up.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent()?.parent()?;
    Some(workspace.join("target").join("lox-itur-data.npz"))
}
