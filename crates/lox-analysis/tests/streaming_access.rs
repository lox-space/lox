// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(all(feature = "stream", feature = "imaging"))]

use std::collections::HashMap;
use std::sync::Arc;

use geo::{LineString, Polygon};
use lox_analysis::assets::{AssetId, DynScenario, Spacecraft};
use lox_analysis::imaging::analysis::{AccessAnalysis, AccessPairResult};
use lox_analysis::imaging::aoi::{Aoi, AoiId};
use lox_analysis::imaging::optical::OpticalPayload;
use lox_analysis::imaging::sar::{LookSide, SarPayload};
use lox_bodies::DynOrigin;
use lox_core::units::{Angle, Distance};
use lox_frames::DynFrame;
use lox_orbits::orbits::{DynTrajectory, Ensemble, Trajectory};
use lox_orbits::propagators::OrbitSource;
use lox_orbits::propagators::Propagator;
use lox_orbits::propagators::sgp4::{Elements, Sgp4};
use lox_stream::OnError;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::{Interval, TimeInterval};
use lox_time::time_scales::{DynTimeScale, Tai};

// Sentinel-1A TLE — epoch 2026-079 (20 March 2026), consistent with the
// SAR integration tests. Reused for optical tests too (any LEO orbit works
// for geometry verification; payload swath is wide enough to generate passes).
const S1A_NAME: &str = "SENTINEL-1A";
const S1A_LINE1: &[u8] = b"1 39634U 14016A   26079.20000000  .00000050  00000+0  37000-4 0  9991";
const S1A_LINE2: &[u8] = b"2 39634  98.1817 105.0000 0001300  90.0000 270.0000 14.59197557600008";

fn s1a_trajectory() -> DynTrajectory {
    let tle = Elements::from_tle(Some(S1A_NAME.to_string()), S1A_LINE1, S1A_LINE2).unwrap();
    let sgp4 = Sgp4::new(tle).unwrap();
    let t0 = sgp4.time();
    let t1 = t0 + TimeDelta::from_hours(6);
    sgp4.with_step(TimeDelta::from_seconds(10))
        .propagate(Interval::new(t0, t1))
        .unwrap()
        .into_dyn()
}

fn western_europe_aoi() -> Aoi {
    Aoi::new(Polygon::new(
        LineString::from(vec![
            (-10.0, 35.0),
            (20.0, 35.0),
            (20.0, 60.0),
            (-10.0, 60.0),
            (-10.0, 35.0),
        ]),
        vec![],
    ))
}

fn make_scenario(
    spacecraft: &[Spacecraft],
    interval: TimeInterval<DynTimeScale>,
) -> (DynScenario, Ensemble<AssetId, Tai, DynOrigin, DynFrame>) {
    let tai_interval =
        TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
    let scenario = DynScenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
        .with_spacecraft(spacecraft);
    let mut map = HashMap::new();
    for sc in spacecraft {
        if let OrbitSource::Trajectory(traj) = sc.orbit() {
            let (epoch, origin, frame, data) = traj.clone().into_parts();
            let typed = Trajectory::from_parts(epoch.with_scale(Tai), origin, frame, data);
            map.insert(sc.id().clone(), typed);
        }
    }
    (scenario, Ensemble::new(map))
}

fn optical_payload() -> OpticalPayload {
    OpticalPayload::off_nadir(Distance::kilometers(290.0), Angle::degrees(20.0))
}

fn sar_payload() -> SarPayload {
    SarPayload::with_incidence_angles(Angle::degrees(29.0), Angle::degrees(46.0), LookSide::Right)
        .unwrap()
}

#[test]
fn optical_streamed_results_equal_batch() {
    let traj = s1a_trajectory();
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    let sc = Spacecraft::new("s1a", OrbitSource::Trajectory(traj))
        .with_optical_payload(optical_payload());
    let (scenario, ensemble) = make_scenario(std::slice::from_ref(&sc), interval);
    let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);

    let batch_analysis = AccessAnalysis::<OpticalPayload, _, _>::new(
        arc_scenario.as_ref(),
        arc_ensemble.as_ref(),
        aois.clone(),
    )
    .with_step(TimeDelta::from_seconds(30));
    let batch = batch_analysis.compute().unwrap();

    let stream_analysis = AccessAnalysis::<OpticalPayload, _, _>::new(
        arc_scenario.as_ref(),
        arc_ensemble.as_ref(),
        aois,
    )
    .with_step(TimeDelta::from_seconds(30));
    let mut s = stream_analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        8,
        OnError::Continue,
    );

    let mut streamed: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    while let Some(item) = s.blocking_next() {
        let AccessPairResult {
            sc_id,
            aoi_id,
            windows,
        } = item.unwrap();
        streamed.insert((sc_id, aoi_id), windows);
    }

    assert_eq!(streamed.len(), batch.num_pairs());
    for ((sc_id, aoi_id), windows) in &streamed {
        let bw = batch.windows(sc_id, aoi_id);
        assert_eq!(windows.len(), bw.len());
    }
}

#[test]
fn sar_streamed_results_equal_batch() {
    let traj = s1a_trajectory();
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    let sc = Spacecraft::new("s1a", OrbitSource::Trajectory(traj)).with_sar_payload(sar_payload());
    let (scenario, ensemble) = make_scenario(std::slice::from_ref(&sc), interval);
    let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);

    use lox_analysis::imaging::sar::SarPayload;

    let batch_analysis = AccessAnalysis::<SarPayload, _, _>::new(
        arc_scenario.as_ref(),
        arc_ensemble.as_ref(),
        aois.clone(),
    )
    .with_step(TimeDelta::from_seconds(30));
    let batch = batch_analysis.compute().unwrap();

    let stream_analysis =
        AccessAnalysis::<SarPayload, _, _>::new(arc_scenario.as_ref(), arc_ensemble.as_ref(), aois)
            .with_step(TimeDelta::from_seconds(30));
    let mut s = stream_analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        8,
        OnError::Continue,
    );

    let mut count_streamed = 0usize;
    while let Some(item) = s.blocking_next() {
        item.unwrap();
        count_streamed += 1;
    }
    assert_eq!(count_streamed, batch.num_pairs());
}

#[test]
fn spacecraft_without_payload_skipped() {
    let traj = s1a_trajectory();
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    // sc_optical has an optical payload; sc_plain does not.
    let sc_optical = Spacecraft::new("sc_optical", OrbitSource::Trajectory(traj.clone()))
        .with_optical_payload(optical_payload());
    let sc_plain = Spacecraft::new("sc_plain", OrbitSource::Trajectory(traj));
    let spacecraft = [sc_optical, sc_plain];
    let (scenario, ensemble) = make_scenario(&spacecraft, interval);
    let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);

    let analysis = AccessAnalysis::<OpticalPayload, _, _>::new(
        arc_scenario.as_ref(),
        arc_ensemble.as_ref(),
        aois,
    )
    .with_step(TimeDelta::from_seconds(30));
    let mut s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        8,
        OnError::Continue,
    );

    let mut sc_ids_seen = std::collections::HashSet::new();
    while let Some(item) = s.blocking_next() {
        let pr = item.unwrap();
        sc_ids_seen.insert(pr.sc_id);
    }
    // Only the optically-equipped spacecraft should appear.
    assert_eq!(sc_ids_seen.len(), 1);
    assert!(sc_ids_seen.contains(&AssetId::new("sc_optical")));
}

#[test]
fn drop_stops_workers() {
    use std::time::Duration;

    let traj = s1a_trajectory();
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    // Build several spacecraft to give the worker pool something to do.
    let spacecraft: Vec<Spacecraft> = (0..8)
        .map(|i| {
            Spacecraft::new(format!("sc{i}"), OrbitSource::Trajectory(traj.clone()))
                .with_optical_payload(optical_payload())
        })
        .collect();
    let (scenario, ensemble) = make_scenario(&spacecraft, interval);
    // Many AOIs to create enough jobs that cancellation is observable.
    let aois: Vec<(AoiId, Aoi)> = (0..8)
        .map(|i| (AoiId::new(format!("aoi{i}")), western_europe_aoi()))
        .collect();

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);

    let analysis = AccessAnalysis::<OpticalPayload, _, _>::new(
        arc_scenario.as_ref(),
        arc_ensemble.as_ref(),
        aois,
    )
    .with_step(TimeDelta::from_seconds(30));
    let s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        4,
        OnError::Continue,
    );
    let token = s.token();
    drop(s);
    std::thread::sleep(Duration::from_millis(100));
    assert!(token.is_cancelled());
}
