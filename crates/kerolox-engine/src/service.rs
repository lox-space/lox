// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Kerolox service implementation: server-streaming ComputeAccess and
//! PropagateTrajectories.

use crate::aoi::AoiLibrary;
use crate::bridge::bridge;
use crate::comparators::ComparatorLibrary;
use crate::mapping::{
    access_window_to_proto, parse_start_time, sar_sensor_to_payload, satellite_to_keplerian,
    unix_epoch_ms_from_utc,
};
use buffa::view::{MessageView, OwnedView};
use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use futures::StreamExt;
use kerolox_proto::kerolox::v1::{
    AccessPairResult, AccessRequestView, Kerolox, PropagateRequestView, ResultSource,
    SampledTrajectoryMessage,
};
use lox_analysis::assets::{Scenario, Spacecraft};
use lox_analysis::imaging::analysis::SarAccessAnalysis;
use lox_analysis::imaging::aoi::AoiId;
use lox_bodies::{DynOrigin, Earth};
use lox_frames::{DynFrame, frames::Icrf, providers::DefaultRotationProvider};
use lox_orbits::orbits::KeplerianOrbit;
use lox_orbits::propagators::sgp4::Sgp4;
use lox_orbits::propagators::{OrbitSource, semi_analytical::DynVallado};
use lox_space::core::elements::Keplerian;
use lox_space::time::time_scales::Tai;
use lox_space::time::utc::transformations::ToUtc;
use lox_stream::OnError;
use lox_time::deltas::TimeDelta;
use std::sync::Arc;
use thiserror::Error;
use tokio_stream::wrappers::ReceiverStream;

/// Errors that can occur during single-satellite trajectory propagation.
#[derive(Debug, Error)]
pub enum PropagateError {
    #[error("invalid start time: {0}")]
    InvalidTime(String),
    #[error("propagator init error: {0}")]
    PropagatorInit(String),
    #[error("propagation error at step {step}: {msg}")]
    Propagation { step: usize, msg: String },
    #[error("frame transform error at step {step}: {msg}")]
    FrameTransform { step: usize, msg: String },
    #[error("ground location error at step {step}: {msg}")]
    GroundLocation { step: usize, msg: String },
}

/// Propagate one satellite over the scenario window and return a
/// [`SampledTrajectoryMessage`] with parallel ECI / ground-track / epoch buffers.
fn propagate_one(
    sc_id: String,
    kep: Keplerian,
    start: lox_time::Time<Tai>,
    duration_s: f64,
    step_s: f64,
) -> Result<SampledTrajectoryMessage, PropagateError> {
    // Build Keplerian orbit → Cartesian → DynVallado.
    let kep_orbit = KeplerianOrbit::try_from_keplerian(kep, start, Earth, Icrf)
        .map_err(|e| PropagateError::PropagatorInit(e.to_string()))?;
    let cartesian = kep_orbit
        .try_to_cartesian()
        .map_err(|e| PropagateError::PropagatorInit(e.to_string()))?;
    let vallado = DynVallado::try_new(cartesian.into_dyn())
        .map_err(|e| PropagateError::PropagatorInit(e.to_string()))?;

    let iau_earth = DynFrame::Iau(DynOrigin::Earth);
    let provider = DefaultRotationProvider;
    let n_steps = (duration_s / step_s) as usize;
    let total = n_steps + 1;

    let mut epochs_ms = Vec::with_capacity(total);
    let mut eci_km = Vec::with_capacity(total * 3);
    let mut ground_deg = Vec::with_capacity(total * 2);

    for i in 0..=n_steps {
        let dt_s = step_s * i as f64;
        // Compute DynTime for this sample by adding a float delta to the TAI start.
        let t_tai = start + TimeDelta::from_seconds_f64(dt_s);
        let t_dyn = t_tai.into_dyn();

        let state = vallado
            .state_at(t_dyn)
            .map_err(|e| PropagateError::Propagation {
                step: i,
                msg: e.to_string(),
            })?;

        // ECI → Three.js (Y-up): (x, z, -y), m → km.
        let pos = state.position();
        eci_km.push(pos.x / 1000.0);
        eci_km.push(pos.z / 1000.0);
        eci_km.push(-pos.y / 1000.0);

        // Body-fixed frame for ground track.
        let body_fixed = state.try_to_frame(iau_earth, &provider).map_err(|e| {
            PropagateError::FrameTransform {
                step: i,
                msg: e.to_string(),
            }
        })?;
        let ground =
            body_fixed
                .try_to_ground_location()
                .map_err(|e| PropagateError::GroundLocation {
                    step: i,
                    msg: e.to_string(),
                })?;
        ground_deg.push(ground.latitude().to_degrees());
        ground_deg.push(ground.longitude().to_degrees());

        // Unix epoch ms from TAI → UTC.
        let utc = t_tai.to_utc();
        epochs_ms.push(unix_epoch_ms_from_utc(&utc));
    }

    Ok(SampledTrajectoryMessage {
        sc_id,
        epochs_ms,
        eci_threejs_buffer_km: eci_km,
        ground_lat_lon_deg: ground_deg,
        comparator_id: String::new(),
        __buffa_unknown_fields: Default::default(),
    })
}

/// Propagate one comparator satellite (SGP4) over the scenario window.
/// Mirrors `propagate_one` but sources state from an `Sgp4` propagator
/// (TEME frame) and converts to ICRF for the ECI buffer.
fn propagate_one_sgp4(
    sc_id: String,
    comparator_id: String,
    sgp4: Sgp4,
    start: lox_time::Time<Tai>,
    duration_s: f64,
    step_s: f64,
) -> Result<SampledTrajectoryMessage, PropagateError> {
    let iau_earth = DynFrame::Iau(DynOrigin::Earth);
    let provider = DefaultRotationProvider;
    let n_steps = (duration_s / step_s) as usize;
    let total = n_steps + 1;

    let mut epochs_ms = Vec::with_capacity(total);
    let mut eci_km = Vec::with_capacity(total * 3);
    let mut ground_deg = Vec::with_capacity(total * 2);

    for i in 0..=n_steps {
        let dt_s = step_s * i as f64;
        let t_tai = start + TimeDelta::from_seconds_f64(dt_s);

        // SGP4 yields a TEME state at a concrete TAI epoch.
        let teme = sgp4
            .state_at(t_tai)
            .map_err(|e| PropagateError::Propagation {
                step: i,
                msg: e.to_string(),
            })?;
        let state = teme.into_dyn();

        // ECI (Three.js Y-up) buffer: convert TEME -> ICRF to match the user path.
        let icrf = state.try_to_frame(DynFrame::Icrf, &provider).map_err(|e| {
            PropagateError::FrameTransform {
                step: i,
                msg: e.to_string(),
            }
        })?;
        let pos = icrf.position();
        eci_km.push(pos.x / 1000.0);
        eci_km.push(pos.z / 1000.0);
        eci_km.push(-pos.y / 1000.0);

        // Ground track via body-fixed frame.
        let body_fixed = state.try_to_frame(iau_earth, &provider).map_err(|e| {
            PropagateError::FrameTransform {
                step: i,
                msg: e.to_string(),
            }
        })?;
        let ground =
            body_fixed
                .try_to_ground_location()
                .map_err(|e| PropagateError::GroundLocation {
                    step: i,
                    msg: e.to_string(),
                })?;
        ground_deg.push(ground.latitude().to_degrees());
        ground_deg.push(ground.longitude().to_degrees());

        let utc = t_tai.to_utc();
        epochs_ms.push(unix_epoch_ms_from_utc(&utc));
    }

    Ok(SampledTrajectoryMessage {
        sc_id,
        epochs_ms,
        eci_threejs_buffer_km: eci_km,
        ground_lat_lon_deg: ground_deg,
        comparator_id,
        __buffa_unknown_fields: Default::default(),
    })
}

pub struct KeroloxImpl {
    aoi_library: Arc<AoiLibrary>,
    comparator_library: Arc<ComparatorLibrary>,
}

impl KeroloxImpl {
    pub fn new(aoi_library: Arc<AoiLibrary>, comparator_library: Arc<ComparatorLibrary>) -> Self {
        Self {
            aoi_library,
            comparator_library,
        }
    }
}

impl Kerolox for KeroloxImpl {
    #[allow(refining_impl_trait)]
    async fn compute_access(
        &self,
        _ctx: RequestContext,
        request: OwnedView<AccessRequestView<'static>>,
    ) -> ServiceResult<ServiceStream<AccessPairResult>> {
        let start = parse_start_time(&request.start_time_iso)
            .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
        let duration_s = request.duration_seconds;
        if !duration_s.is_finite() || duration_s <= 0.0 {
            return Err(ConnectError::invalid_argument(format!(
                "duration_seconds must be positive and finite (got {duration_s})"
            )));
        }
        let duration = TimeDelta::from_seconds(duration_s as i64);
        let end = start + duration;

        let sar_view = request
            .sar
            .as_option()
            .ok_or_else(|| ConnectError::invalid_argument("missing sar sensor"))?;
        let sar_owned = sar_view.to_owned_message();
        let sar = sar_sensor_to_payload(&sar_owned)
            .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;

        let mut spacecraft: Vec<Spacecraft> = Vec::with_capacity(request.satellites.len());
        for s_view in request.satellites.iter() {
            let s_owned = s_view.to_owned_message();
            let keplerian = satellite_to_keplerian(&s_owned)
                .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
            let kep_orbit = KeplerianOrbit::try_from_keplerian(keplerian, start, Earth, Icrf)
                .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
            let cartesian = kep_orbit
                .try_to_cartesian()
                .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
            let cartesian_dyn = cartesian.into_dyn();
            let vallado = DynVallado::try_new(cartesian_dyn)
                .map_err(|e| ConnectError::internal(e.to_string()))?;
            let sc = Spacecraft::new(s_owned.id.as_str(), OrbitSource::Vallado(vallado))
                .with_sar_payload(sar);
            spacecraft.push(sc);
        }

        // Append comparator spacecraft (e.g. ICEYE) propagated via SGP4.
        // Their ids are namespaced "<comparator>/<name>" so the result
        // mapping can tag them as COMPARATOR.
        for comparator_id in request.comparators.iter() {
            let cid: &str = comparator_id.as_ref();
            let comp = self
                .comparator_library
                .get(cid)
                .ok_or_else(|| ConnectError::not_found(format!("unknown comparator: {cid}")))?;
            for (name, sgp4) in &comp.satellites {
                let sc_id = format!("{cid}/{name}");
                let sc = Spacecraft::new(sc_id.as_str(), OrbitSource::Sgp4(sgp4.clone()))
                    .with_sar_payload(sar);
                spacecraft.push(sc);
            }
        }

        let mut aois = Vec::with_capacity(request.aoi_ids.len());
        for id in request.aoi_ids.iter() {
            let id_str: &str = id.as_ref();
            let aoi = self
                .aoi_library
                .get(id_str)
                .cloned()
                .ok_or_else(|| ConnectError::not_found(format!("unknown aoi id: {id_str}")))?;
            aois.push((AoiId::new(id_str), aoi));
        }

        let step_s = request.step_seconds;
        let step_secs = if step_s.is_finite() && step_s > 0.0 {
            step_s as i64
        } else {
            30
        };

        let scenario = Arc::new(
            Scenario::new(start, end, DynOrigin::Earth, DynFrame::Icrf)
                .with_spacecraft(&spacecraft),
        );
        let ensemble = Arc::new(
            scenario
                .propagate(&DefaultRotationProvider)
                .map_err(|e| ConnectError::internal(e.to_string()))?,
        );

        let analysis = SarAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(step_secs));

        let lox_stream = analysis.compute_stream(
            Arc::clone(&scenario),
            Arc::clone(&ensemble),
            64,
            OnError::Abort,
        );

        let futures_stream = bridge(lox_stream).map(|res| {
            let pair = res?;
            let windows = pair.windows.iter().map(access_window_to_proto).collect();
            let sc_id_str = pair.sc_id.to_string();
            let (source, comparator_id) = match sc_id_str.split_once('/') {
                Some((cid, _)) => (ResultSource::RESULT_SOURCE_COMPARATOR, cid.to_string()),
                None => (ResultSource::RESULT_SOURCE_USER, String::new()),
            };
            Ok::<_, ConnectError>(AccessPairResult {
                sc_id: sc_id_str,
                aoi_id: pair.aoi_id.to_string(),
                source: source.into(),
                comparator_id,
                windows,
                __buffa_unknown_fields: Default::default(),
            })
        });

        Response::stream_ok(futures_stream)
    }

    #[allow(refining_impl_trait)]
    async fn propagate_trajectories(
        &self,
        _ctx: RequestContext,
        request: OwnedView<PropagateRequestView<'static>>,
    ) -> ServiceResult<ServiceStream<SampledTrajectoryMessage>> {
        let start = parse_start_time(&request.start_time_iso)
            .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
        let duration_s = request.duration_seconds;
        if !duration_s.is_finite() || duration_s <= 0.0 {
            return Err(ConnectError::invalid_argument(format!(
                "duration_seconds must be positive and finite (got {duration_s})"
            )));
        }
        let step_s = request.step_seconds;
        let step_s_final = if step_s.is_finite() && step_s > 0.0 {
            step_s
        } else {
            30.0
        };

        // Eagerly validate and collect (sc_id, Keplerian) pairs so we own them
        // before spawning the parallel stream.
        let sats: Vec<(String, Keplerian)> = request
            .satellites
            .iter()
            .map(|s_view| {
                let s_owned = s_view.to_owned_message();
                let kep = satellite_to_keplerian(&s_owned)
                    .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
                Ok::<_, ConnectError>((s_owned.id.to_string(), kep))
            })
            .collect::<Result<_, _>>()?;

        // Resolve comparator satellites (SGP4) to stream alongside the user
        // trajectories. Cloned into owned data before spawn_blocking.
        let mut comp_sats: Vec<(String, String, Sgp4)> = Vec::new();
        for comparator_id in request.comparators.iter() {
            let cid: &str = comparator_id.as_ref();
            let comp = self
                .comparator_library
                .get(cid)
                .ok_or_else(|| ConnectError::not_found(format!("unknown comparator: {cid}")))?;
            for (name, sgp4) in &comp.satellites {
                comp_sats.push((format!("{cid}/{name}"), cid.to_string(), sgp4.clone()));
            }
        }

        // Run propagation sequentially on the tokio blocking pool rather than
        // the rayon pool. This keeps the rayon pool free for compute_access's
        // parallel access analysis, so the two RPCs run concurrently instead
        // of access queuing behind propagation.
        let (tx, rx) =
            tokio::sync::mpsc::channel::<Result<SampledTrajectoryMessage, ConnectError>>(64);

        tokio::task::spawn_blocking(move || {
            for (sc_id, kep) in sats {
                let result = propagate_one(sc_id, kep, start, duration_s, step_s_final)
                    .map_err(|e| ConnectError::internal(e.to_string()));
                let is_err = result.is_err();
                if tx.blocking_send(result).is_err() {
                    // Receiver dropped (client disconnected / aborted).
                    break;
                }
                if is_err {
                    break;
                }
            }
            for (sc_id, comparator_id, sgp4) in comp_sats {
                let result =
                    propagate_one_sgp4(sc_id, comparator_id, sgp4, start, duration_s, step_s_final)
                        .map_err(|e| ConnectError::internal(e.to_string()));
                let is_err = result.is_err();
                if tx.blocking_send(result).is_err() {
                    break;
                }
                if is_err {
                    break;
                }
            }
        });

        let stream = ReceiverStream::new(rx);
        Response::stream_ok(stream)
    }
}
