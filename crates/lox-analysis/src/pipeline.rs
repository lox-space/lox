use std::{collections::HashMap, marker::PhantomData, ops::Deref};

use lox_bodies::CoordinateOrigin;
use lox_frames::ReferenceFrame;
use lox_orbits::orbits::Ensemble;
use lox_time::intervals::TimeInterval;
use rayon::iter::{
    FromParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};

use crate::{
    assets::{AssetId, GroundStation, Scenario, Spacecraft},
    visibility::NoEphemeris,
};

trait AssetIdExt {
    fn asset_id(&self) -> AssetId;
}

impl AssetIdExt for Spacecraft {
    fn asset_id(&self) -> AssetId {
        self.id().clone()
    }
}

impl AssetIdExt for GroundStation {
    fn asset_id(&self) -> AssetId {
        self.id().clone()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct IntervalMap(HashMap<(AssetId, AssetId), Vec<TimeInterval>>);

impl Deref for IntervalMap {
    type Target = HashMap<(AssetId, AssetId), Vec<TimeInterval>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<(AssetId, AssetId, TimeInterval)> for IntervalMap {
    fn from_iter<T: IntoIterator<Item = (AssetId, AssetId, TimeInterval)>>(iter: T) -> Self {
        IntervalMap(
            iter.into_iter()
                .fold(HashMap::new(), |mut map, (t1, t2, interval)| {
                    let key = (t1, t2);
                    map.entry(key).or_insert_with(Vec::new).push(interval);
                    map
                }),
        )
    }
}

impl FromParallelIterator<(AssetId, AssetId, TimeInterval)> for IntervalMap {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (AssetId, AssetId, TimeInterval)>,
    {
        IntervalMap(
            par_iter
                .into_par_iter()
                .fold(HashMap::new, |mut map, (t1, t2, interval)| {
                    let key = (t1, t2);
                    map.entry(key).or_insert_with(Vec::new).push(interval);

                    map
                })
                .reduce(HashMap::new, |mut a, b| {
                    for (k, v) in b {
                        a.entry(k).or_insert_with(Vec::new).extend(v);
                    }
                    a
                }),
        )
    }
}

struct Analysis<'a, O: CoordinateOrigin, R: ReferenceFrame> {
    scenario: &'a Scenario<O, R>,
}

impl<'a, O: CoordinateOrigin, R: ReferenceFrame> Analysis<'a, O, R> {
    fn new(scenario: &'a Scenario<O, R>) -> Self {
        Self { scenario }
    }
}

impl<O, R> Refine<GroundStation, Spacecraft> for Analysis<'_, O, R>
where
    O: CoordinateOrigin + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a GroundStation, &'a Spacecraft, TimeInterval)>
    where
        Spacecraft: 'a,
        GroundStation: 'a,
    {
        self.scenario.ground_stations().iter().flat_map(move |gs| {
            self.scenario
                .spacecraft()
                .iter()
                .map(move |sc| (gs, sc, *self.scenario.interval()))
        })
    }

    fn par_iter<'a>(
        &'a self,
    ) -> impl ParallelIterator<Item = (&'a GroundStation, &'a Spacecraft, TimeInterval)>
    where
        GroundStation: 'a,
        Spacecraft: 'a,
    {
        self.scenario
            .ground_stations()
            .par_iter()
            .flat_map(move |gs| {
                self.scenario
                    .spacecraft()
                    .par_iter()
                    .map(move |sc| (gs, sc, *self.scenario.interval()))
            })
    }
}

trait Refine<T1, T2>: Sized
where
    T1: AssetIdExt,
    T2: AssetIdExt,
{
    fn refine<R, I>(self, refinement: R) -> Refined<T1, T2, Self, R, I>
    where
        R: Fn(&T1, &T2, TimeInterval) -> I,
        I: Iterator<Item = TimeInterval>,
    {
        Refined {
            wrapped: self,
            refinement,
            __: PhantomData,
        }
    }

    fn filter<F>(self, filter: F) -> Filtered<T1, T2, Self, F>
    where
        F: Fn(&T1, &T2, TimeInterval) -> bool,
    {
        Filtered {
            wrapped: self,
            filter,
            __: PhantomData,
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a,
        T2: 'a;

    fn par_iter<'a>(&'a self) -> impl ParallelIterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync;

    fn collect(self) -> IntervalMap {
        self.iter()
            .map(|(t1, t2, interval)| (t1.asset_id(), t2.asset_id(), interval))
            .collect()
    }

    fn par_collect(self) -> IntervalMap
    where
        T1: Send + Sync,
        T2: Send + Sync,
    {
        self.par_iter()
            .map(|(t1, t2, interval)| (t1.asset_id(), t2.asset_id(), interval))
            .collect()
    }
}

struct Refined<T1, T2, W, R, I> {
    wrapped: W,
    refinement: R,
    __: PhantomData<(T1, T2, I)>,
}

impl<T1, T2, W, R, I> Refine<T1, T2> for Refined<T1, T2, W, R, I>
where
    T1: AssetIdExt,
    T2: AssetIdExt,
    W: Refine<T1, T2> + Send + Sync,
    R: Fn(&T1, &T2, TimeInterval) -> I + Send + Sync,
    I: Iterator<Item = TimeInterval> + Send + Sync,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a,
        T2: 'a,
    {
        self.wrapped.iter().flat_map(move |(t1, t2, interval)| {
            (self.refinement)(t1, t2, interval).map(move |sub_interval| (t1, t2, sub_interval))
        })
    }

    fn par_iter<'a>(&'a self) -> impl ParallelIterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync,
    {
        self.wrapped.par_iter().flat_map(move |(t1, t2, interval)| {
            let refined = (self.refinement)(t1, t2, interval).collect::<Vec<_>>();

            refined
                .into_par_iter()
                .map(move |sub_interval| (t1, t2, sub_interval))
        })
    }
}

struct Filtered<T1, T2, W, F> {
    wrapped: W,
    filter: F,
    __: PhantomData<(T1, T2)>,
}

impl<T1, T2, W, F> Refine<T1, T2> for Filtered<T1, T2, W, F>
where
    T1: AssetIdExt,
    T2: AssetIdExt,
    W: Refine<T1, T2> + Send + Sync,
    F: Fn(&T1, &T2, TimeInterval) -> bool + Send + Sync,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a,
        T2: 'a,
    {
        self.wrapped
            .iter()
            .filter(|(t1, t2, interval)| (self.filter)(t1, t2, *interval))
    }

    fn par_iter<'a>(&'a self) -> impl ParallelIterator<Item = (&'a T1, &'a T2, TimeInterval)>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync,
    {
        self.wrapped
            .par_iter()
            .filter(|(t1, t2, interval)| (self.filter)(t1, t2, *interval))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::events::DetectFnExt;
    use lox_bodies::Origin;
    use lox_core::{coords::LonLatAlt, units::Angle};
    use lox_frames::Frame;
    use lox_orbits::{ground::EllipsoidLocation, orbits::Trajectory, propagators::OrbitSource};
    use lox_test_utils::read_data_file;
    use lox_time::time_scales::TimeScale;
    use lox_time::{deltas::TimeDelta, TimeBuilder};

    use crate::{
        events::{DetectFn, UniformSampler},
        visibility::{ElevationDetectFn, ElevationMask},
    };

    use super::*;

    fn make_scenario_and_ensemble(
        ground_assets: &[GroundStation],
        space_assets: &[Spacecraft],
        interval: TimeInterval<TimeScale>,
    ) -> (Scenario<Origin, Frame>, Ensemble<AssetId, Origin, Frame>) {
        let scenario_interval = TimeInterval::new(interval.start(), interval.end());
        let scenario = Scenario::with_interval(scenario_interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(ground_assets)
            .with_spacecraft(space_assets);
        // Build ensemble from OrbitSource::Trajectory entries
        let mut map = HashMap::new();
        for sc in space_assets {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                // Re-tag Trajectory as Ensemble<Origin, Frame>
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                let typed =
                    Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
                map.insert(sc.id().clone(), typed);
            }
        }
        let ensemble = Ensemble::new(map);
        (scenario, ensemble)
    }

    fn spacecraft_trajectory_dynamic() -> Trajectory {
        Trajectory::from_csv_dynamic(
            &read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap()
    }

    fn location_dynamic() -> EllipsoidLocation {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap()
    }

    #[test]
    fn refinement() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(Angle::ZERO);
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let step = TimeDelta::from_seconds(60);

        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let result = Analysis::new(&scenario)
            .refine(|gs, sc, interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                    "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
		);
                let elev = ElevationDetectFn {
                    gs: gs.location(),
                    mask: gs.mask(),
                    sc: sc_traj,
                };
                let windows = elev.intervals(UniformSampler::new(step), interval).unwrap();
                windows.into_iter()
            })
            // .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000))
            .collect();

        let result_par = Analysis::new(&scenario)
            .refine(|gs, sc, interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                    "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
		);
                let elev = ElevationDetectFn {
                    gs: gs.location(),
                    mask: gs.mask(),
                    sc: sc_traj,
                };
                let windows = elev.intervals(UniformSampler::new(step), interval).unwrap();
                windows.into_iter()
            })
            // .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000))
            .par_collect();

        dbg!(
            result
                .get(&(gs.id().clone(), sc.id().clone()))
                .unwrap()
                .len(),
            result_par
                .get(&(gs.id().clone(), sc.id().clone()))
                .unwrap()
                .len()
        );
        assert_eq!(result, result_par);
        dbg!(result);
        panic!();
    }
}
