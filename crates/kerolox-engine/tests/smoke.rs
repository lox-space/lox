// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end smoke test: spawn the engine on a free port, send a tiny
//! request via a Connect client, verify at least one AccessPairResult
//! comes back.

use connectrpc::client::{ClientConfig, HttpClient};
use kerolox_engine::{aoi::AoiLibrary, comparators::ComparatorLibrary, service::KeroloxImpl};
use kerolox_proto::kerolox::v1::{
    AccessRequest, KeroloxClient, KeroloxExt, LookSide, PropagateRequest, ResultSource, SarSensor,
    SatelliteOrbitalElements,
};
use std::path::PathBuf;
use std::sync::Arc;

fn aoi_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("aois")
}

fn comparator_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("comparators")
}

fn make_service() -> Arc<KeroloxImpl> {
    let aoi_lib = Arc::new(AoiLibrary::load_from_dir(&aoi_dir()).unwrap());
    let comp_lib = Arc::new(ComparatorLibrary::load_from_dir(&comparator_dir()).unwrap());
    Arc::new(KeroloxImpl::new(aoi_lib, comp_lib))
}

#[tokio::test(flavor = "multi_thread")]
async fn compute_access_streams_at_least_one_pair() {
    let service = make_service();
    let connect_router = service.register(connectrpc::Router::new());
    let app = axum::Router::new().fallback_service(connect_router.into_axum_service());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    // Single circular LEO satellite at 600 km altitude, 53° inclination.
    // Over 24 hours, it will overfly both Hormuz and Black Sea.
    let req = AccessRequest {
        start_time_iso: "2026-06-01T00:00:00.000".into(),
        duration_seconds: 24.0 * 3600.0,
        satellites: vec![SatelliteOrbitalElements {
            id: "sat-0".into(),
            sma_m: 6_978_137.0, // 600 km altitude
            ecc: 0.0,
            inc_rad: 53.0_f64.to_radians(),
            raan_rad: 0.0,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            __buffa_unknown_fields: Default::default(),
        }],
        sar: buffa::MessageField::some(SarSensor {
            look_side: LookSide::LOOK_SIDE_RIGHT.into(),
            min_incidence_deg: 20.0,
            max_incidence_deg: 45.0,
            __buffa_unknown_fields: Default::default(),
        }),
        aoi_ids: vec!["hormuz".into(), "black_sea".into()],
        comparators: vec![],
        step_seconds: 30.0,
        __buffa_unknown_fields: Default::default(),
    };

    let http = HttpClient::plaintext();
    let config = ClientConfig::new(format!("http://{addr}").parse().unwrap());
    let client = KeroloxClient::new(http, config);

    let mut stream = client.compute_access(req).await.unwrap();
    let mut count = 0usize;
    while let Some(_msg) = stream.message().await.unwrap() {
        count += 1;
    }
    eprintln!("smoke: received {count} AccessPairResult(s)");
    assert!(count >= 1, "expected at least one streamed pair, got {count}");
    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn propagate_trajectories_streams_at_least_one_message() {
    let service = make_service();
    let connect_router = service.register(connectrpc::Router::new());
    let app = axum::Router::new().fallback_service(connect_router.into_axum_service());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    // Single circular LEO satellite at 600 km, 1 hour scenario, 30 s step.
    let req = PropagateRequest {
        start_time_iso: "2026-06-01T00:00:00.000".into(),
        duration_seconds: 3600.0,
        step_seconds: 30.0,
        satellites: vec![SatelliteOrbitalElements {
            id: "sat-0".into(),
            sma_m: 6_978_137.0,
            ecc: 0.0,
            inc_rad: 53.0_f64.to_radians(),
            raan_rad: 0.0,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            __buffa_unknown_fields: Default::default(),
        }],
        __buffa_unknown_fields: Default::default(),
    };

    let http = HttpClient::plaintext();
    let config = ClientConfig::new(format!("http://{addr}").parse().unwrap());
    let client = KeroloxClient::new(http, config);

    let mut stream = client.propagate_trajectories(req).await.unwrap();
    let mut count = 0usize;
    while let Some(msg) = stream.message().await.unwrap() {
        eprintln!("smoke: trajectory sc_id={}, {} samples", msg.sc_id, msg.epochs_ms.len());
        count += 1;
    }
    eprintln!("smoke: received {count} SampledTrajectoryMessage(s)");
    assert!(count >= 1, "expected at least one trajectory message, got {count}");
    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn compute_access_comparators_returns_comparator_tagged_pairs() {
    let service = make_service();
    let connect_router = service.register(connectrpc::Router::new());
    let app = axum::Router::new().fallback_service(connect_router.into_axum_service());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    // Single user satellite + ICEYE comparator, 24 h scenario.
    let req = AccessRequest {
        start_time_iso: "2026-06-01T00:00:00.000".into(),
        duration_seconds: 24.0 * 3600.0,
        satellites: vec![SatelliteOrbitalElements {
            id: "sat-0".into(),
            sma_m: 6_978_137.0,
            ecc: 0.0,
            inc_rad: 53.0_f64.to_radians(),
            raan_rad: 0.0,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            __buffa_unknown_fields: Default::default(),
        }],
        sar: buffa::MessageField::some(SarSensor {
            look_side: LookSide::LOOK_SIDE_RIGHT.into(),
            min_incidence_deg: 20.0,
            max_incidence_deg: 45.0,
            __buffa_unknown_fields: Default::default(),
        }),
        aoi_ids: vec!["hormuz".into(), "black_sea".into()],
        comparators: vec!["iceye".into()],
        step_seconds: 30.0,
        __buffa_unknown_fields: Default::default(),
    };

    let http = HttpClient::plaintext();
    let config = ClientConfig::new(format!("http://{addr}").parse().unwrap());
    let client = KeroloxClient::new(http, config);

    let mut stream = client.compute_access(req).await.unwrap();
    let mut user_count = 0usize;
    let mut comparator_count = 0usize;
    while let Some(msg) = stream.message().await.unwrap() {
        if msg.source == ResultSource::RESULT_SOURCE_COMPARATOR {
            comparator_count += 1;
            assert_eq!(msg.comparator_id, "iceye", "comparator_id should be 'iceye'");
        } else {
            user_count += 1;
        }
    }
    eprintln!(
        "smoke: received {user_count} USER pair(s) and {comparator_count} COMPARATOR pair(s)"
    );
    assert!(comparator_count >= 1, "expected at least one COMPARATOR pair, got {comparator_count}");
    server.abort();
}
