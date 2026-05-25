// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Kerolox service implementation: server-streaming ComputeAccess.

use crate::aoi::AoiLibrary;
use crate::bridge::bridge;
use crate::mapping::{access_window_to_proto, parse_start_time, sar_sensor_to_payload, satellite_to_keplerian};
use buffa::view::{MessageView, OwnedView};
use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use futures::StreamExt;
use kerolox_proto::kerolox::v1::{
    AccessPairResult, AccessRequestView, Kerolox, ResultSource,
};
use lox_analysis::assets::{Scenario, Spacecraft};
use lox_analysis::imaging::aoi::AoiId;
use lox_analysis::imaging::analysis::SarAccessAnalysis;
use lox_bodies::{DynOrigin, Earth};
use lox_frames::{DynFrame, frames::Icrf, providers::DefaultRotationProvider};
use lox_orbits::orbits::KeplerianOrbit;
use lox_orbits::propagators::{OrbitSource, semi_analytical::DynVallado};
use lox_stream::OnError;
use lox_time::deltas::TimeDelta;
use std::sync::Arc;

pub struct KeroloxImpl {
    aoi_library: Arc<AoiLibrary>,
}

impl KeroloxImpl {
    pub fn new(aoi_library: Arc<AoiLibrary>) -> Self {
        Self { aoi_library }
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
        let duration = TimeDelta::from_seconds(request.duration_seconds as i64);
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
            let kep_orbit =
                KeplerianOrbit::try_from_keplerian(keplerian, start, Earth, Icrf)
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

        let step_secs = if request.step_seconds > 0.0 {
            request.step_seconds as i64
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
            Ok::<_, ConnectError>(AccessPairResult {
                sc_id: pair.sc_id.to_string(),
                aoi_id: pair.aoi_id.to_string(),
                source: ResultSource::RESULT_SOURCE_USER.into(),
                comparator_id: String::new(),
                windows,
                __buffa_unknown_fields: Default::default(),
            })
        });

        Response::stream_ok(futures_stream)
    }
}
