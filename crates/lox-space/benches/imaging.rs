// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! AOI imaging-access benchmarks: optical (nadir and off-nadir) and SAR
//! payloads, plus spacecraft- and AOI-count scaling.
//!
//! The whole suite is gated on the `imaging` feature (a default lox-space
//! feature). With the feature off the bench compiles to an empty divan run, so
//! `cargo bench -p lox-space --no-default-features --features analysis` still
//! succeeds. Run the full suite with `cargo bench -p lox-space --bench imaging`.

#[path = "common/mod.rs"]
mod common;

fn main() {
    divan::main();
}

#[cfg(feature = "imaging")]
mod imaging_benches {
    use divan::Bencher;
    use geo::{LineString, Polygon};
    use lox_space::analysis::assets::{DynScenario, Spacecraft};
    use lox_space::analysis::imaging::{
        Aoi, AoiId, LookSide, OpticalAccessAnalysis, OpticalPayload, SarAccessAnalysis, SarPayload,
    };
    use lox_space::core::units::{Angle, Distance};
    use lox_space::orbits::DynTrajectory;
    use lox_space::orbits::propagators::sgp4::{Elements, Sgp4};
    use lox_space::orbits::propagators::{OrbitSource, Propagator};
    use lox_space::time::deltas::TimeDelta;
    use lox_space::time::intervals::Interval;

    use super::common::{self, DynEnsemble};

    fn sentinel2_trajectory(name: &str, line1: &[u8], line2: &[u8]) -> DynTrajectory {
        let tle = Elements::from_tle(Some(name.to_string()), line1, line2).unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_hours(6);
        sgp4.with_step(TimeDelta::from_seconds(10))
            .propagate(Interval::new(t0, t1))
            .unwrap()
            .into_dyn()
    }

    fn sentinel2a_trajectory() -> DynTrajectory {
        sentinel2_trajectory(
            "SENTINEL-2A",
            b"1 40697U 15028A   26079.19377485 -.00000072  00000+0 -11026-4 0  9994",
            b"2 40697  98.5642 155.3327 0001269  98.1407 261.9920 14.30816376561005",
        )
    }

    /// A large AOI covering most of Western Europe — a sun-synchronous LEO
    /// satellite will overfly this within the 6 h window.
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

    /// `i` small distinct AOIs spread along a band of longitudes.
    fn aoi_band(count: usize) -> Vec<(AoiId, Aoi)> {
        (0..count)
            .map(|i| {
                let lon = -10.0 + (i as f64) * 2.0;
                let aoi = Aoi::new(Polygon::new(
                    LineString::from(vec![
                        (lon, 35.0),
                        (lon + 1.5, 35.0),
                        (lon + 1.5, 60.0),
                        (lon, 60.0),
                        (lon, 35.0),
                    ]),
                    vec![],
                ));
                (AoiId::new(format!("aoi-{i}")), aoi)
            })
            .collect()
    }

    /// `n` OneWeb spacecraft each carrying the given optical payload, plus a
    /// scenario/ensemble assembled via the shared fixtures.
    fn oneweb_optical_scenario(n: usize, payload: OpticalPayload) -> (DynScenario, DynEnsemble) {
        let trajs = common::propagate_oneweb_trajectories(n, 6);
        let spacecraft: Vec<Spacecraft> = trajs
            .iter()
            .map(|(name, traj)| {
                Spacecraft::new(name.clone(), OrbitSource::Trajectory(traj.clone()))
                    .with_optical_payload(payload)
            })
            .collect();
        let trajectories: Vec<DynTrajectory> = trajs.into_iter().map(|(_, t)| t).collect();
        common::assemble_scenario(spacecraft, trajectories, &[])
    }

    fn optical_setup(payload: OpticalPayload) -> (DynScenario, DynEnsemble) {
        let traj = sentinel2a_trajectory();
        let sc = Spacecraft::new("s2a", OrbitSource::Trajectory(traj.clone()))
            .with_optical_payload(payload);
        common::assemble_scenario(vec![sc], vec![traj], &[])
    }

    #[divan::bench]
    fn optical_nadir(bencher: Bencher) {
        let payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));
        let (scenario, ensemble) = optical_setup(payload);
        bencher.bench(|| {
            let aois = vec![(AoiId::new("europe"), western_europe_aoi())];
            OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
                .with_step(TimeDelta::from_seconds(30))
                .compute()
                .unwrap()
        });
    }

    #[divan::bench]
    fn optical_off_nadir(bencher: Bencher) {
        let payload = OpticalPayload::off_nadir(Distance::kilometers(290.0), Angle::degrees(30.0));
        let (scenario, ensemble) = optical_setup(payload);
        bencher.bench(|| {
            let aois = vec![(AoiId::new("europe"), western_europe_aoi())];
            OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
                .with_step(TimeDelta::from_seconds(30))
                .compute()
                .unwrap()
        });
    }

    #[divan::bench]
    fn sar_access(bencher: Bencher) {
        let traj = sentinel2a_trajectory();
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap();
        let sc =
            Spacecraft::new("s2a", OrbitSource::Trajectory(traj.clone())).with_sar_payload(payload);
        let (scenario, ensemble) = common::assemble_scenario(vec![sc], vec![traj], &[]);
        bencher.bench(|| {
            let aois = vec![(AoiId::new("europe"), western_europe_aoi())];
            SarAccessAnalysis::new(&scenario, &ensemble, aois)
                .with_step(TimeDelta::from_seconds(30))
                .compute()
                .unwrap()
        });
    }

    // Spacecraft-count scaling (single AOI).
    #[divan::bench(args = [1, 2, 4])]
    fn imaging_scaling_spacecraft(bencher: Bencher, n: usize) {
        let payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));
        bencher
            .with_inputs(|| oneweb_optical_scenario(n, payload))
            .bench_values(|(scenario, ensemble)| {
                let aois = vec![(AoiId::new("europe"), western_europe_aoi())];
                OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
                    .with_step(TimeDelta::from_seconds(30))
                    .compute()
                    .unwrap()
            });
    }

    // AOI-count scaling (single spacecraft).
    #[divan::bench(args = [1, 4, 16])]
    fn imaging_scaling_aois(bencher: Bencher, n_aoi: usize) {
        let payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));
        let (scenario, ensemble) = optical_setup(payload);
        bencher.bench(|| {
            let aois = aoi_band(n_aoi);
            OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
                .with_step(TimeDelta::from_seconds(30))
                .compute()
                .unwrap()
        });
    }
}
