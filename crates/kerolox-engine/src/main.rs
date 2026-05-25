// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use axum::{Router, routing::get};
use kerolox_engine::{aoi::AoiLibrary, comparators::ComparatorLibrary, cors::dev_cors, service::KeroloxImpl};
use kerolox_proto::kerolox::v1::KeroloxExt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let addr_str = std::env::var("KEROLOX_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| format!("KEROLOX_ADDR={addr_str:?} is not a valid socket address: {e}"))?;

    let aoi_dir: PathBuf = std::env::var("KEROLOX_AOI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("data")
                .join("aois")
        });
    let aoi_library = Arc::new(AoiLibrary::load_from_dir(&aoi_dir)?);
    tracing::info!("loaded {} AOIs", aoi_library.len());

    let comparator_dir = aoi_dir
        .parent()
        .map(|p| p.join("comparators"))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data").join("comparators"));
    let comparator_library = Arc::new(ComparatorLibrary::load_from_dir(&comparator_dir)?);
    tracing::info!(
        "loaded comparators: iceye={}",
        comparator_library.get("iceye").map(|c| c.satellites.len()).unwrap_or(0)
    );

    let service = Arc::new(KeroloxImpl::new(aoi_library, comparator_library));
    let connect = service.register(connectrpc::Router::new());

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service())
        .layer(dev_cors()); // TODO(prod): replace with an origin-allowlist before deployment

    tracing::info!("kerolox-engine listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
