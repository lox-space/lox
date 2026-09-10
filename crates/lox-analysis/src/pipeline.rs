use std::{collections::HashMap, convert::Infallible, marker::PhantomData, ops::Deref};

use lox_bodies::CoordinateOrigin;
use lox_frames::ReferenceFrame;
use lox_orbits::orbits::Ensemble;
use lox_time::intervals::TimeInterval;
use rayon::iter::{
    FromParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};

use crate::{
    assets::{AssetId, GroundStation, Scenario, Spacecraft},
    visibility::{NoEphemeris, VisibilityError},
};

impl From<Infallible> for VisibilityError {
    fn from(value: Infallible) -> Self {
        unreachable!()
    }
}

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

struct Analysis<'a, T1, T2> {
    t1: &'a [T1],
    t2: &'a [T2],
    interval: TimeInterval,
}

impl<'a, T1, T2> Analysis<'a, T1, T2> {
    fn new(t1: &'a [T1], t2: &'a [T2], interval: TimeInterval) -> Self {
        Self { t1, t2, interval }
    }
}

impl<T1, T2> Refine<T1, T2> for Analysis<'_, T1, T2>
where
    T1: AssetIdExt,
    T2: AssetIdExt,
{
    type Error = Infallible;

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a,
        T2: 'a,
    {
        self.t1.iter().flat_map(move |item1| {
            self.t2
                .iter()
                .map(move |item2| Ok((item1, item2, self.interval)))
        })
    }

    fn par_iter<'a>(
        &'a self,
    ) -> impl ParallelIterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync,
    {
        self.t1.par_iter().flat_map(move |item1| {
            self.t2
                .par_iter()
                .map(move |item2| Ok((item1, item2, self.interval)))
        })
    }
}

trait Refine<T1, T2>: Sized
where
    T1: AssetIdExt,
    T2: AssetIdExt,
{
    type Error: Send + Sync;

    fn refine<R, I, E>(self, refinement: R) -> Refined<T1, T2, Self, R, I, E>
    where
        R: Fn(&T1, &T2, TimeInterval) -> Result<I, E>,
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

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a,
        T2: 'a;

    fn collect(self) -> Result<IntervalMap, Self::Error> {
        self.iter()
            .map(|item| item.map(|(t1, t2, interval)| (t1.asset_id(), t2.asset_id(), interval)))
            .collect()
    }

    fn par_iter<'a>(
        &'a self,
    ) -> impl ParallelIterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync;

    fn par_collect(self) -> Result<IntervalMap, Self::Error>
    where
        T1: Send + Sync,
        T2: Send + Sync,
    {
        self.par_iter()
            .map(|item| item.map(|(t1, t2, interval)| (t1.asset_id(), t2.asset_id(), interval)))
            .collect()
    }
}

struct Refined<T1, T2, W, R, I, E> {
    wrapped: W,
    refinement: R,
    __: PhantomData<(T1, T2, I, E)>,
}

impl<T1, T2, W, R, I, E> Refine<T1, T2> for Refined<T1, T2, W, R, I, E>
where
    T1: AssetIdExt,
    T2: AssetIdExt,
    W: Refine<T1, T2> + Send + Sync,
    R: Fn(&T1, &T2, TimeInterval) -> Result<I, E> + Send + Sync,
    I: Iterator<Item = TimeInterval> + Send + Sync,
    E: From<W::Error> + Send + Sync,
{
    type Error = E;

    fn iter<'a>(&'a self) -> impl Iterator<Item = Result<(&'a T1, &'a T2, TimeInterval), E>>
    where
        T1: 'a,
        T2: 'a,
    {
        self.wrapped
            .iter()
            .flat_map(move |item| {
                let (t1, t2, interval) = item?;
                let intervals = (self.refinement)(t1, t2, interval)?;

                Ok::<_, E>(intervals.map(move |sub_interval| Ok((t1, t2, sub_interval))))
            })
            .flatten()
    }

    fn par_iter<'a>(
        &'a self,
    ) -> impl ParallelIterator<Item = Result<(&'a T1, &'a T2, TimeInterval), E>>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync,
    {
        self.wrapped
            .par_iter()
            .flat_map(move |item| {
                let (t1, t2, interval) = item?;

                let intervals = (self.refinement)(t1, t2, interval)?;
                let intervals = intervals.collect::<Vec<_>>();

                Ok::<_, E>(
                    intervals
                        .into_par_iter()
                        .map(move |sub_interval| Ok((t1, t2, sub_interval))),
                )
            })
            .flatten()
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
    type Error = W::Error;

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a,
        T2: 'a,
    {
        self.wrapped.iter().filter(|item| match item {
            Ok((t1, t2, interval)) => (self.filter)(t1, t2, *interval),
            Err(_) => true,
        })
    }

    fn par_iter<'a>(
        &'a self,
    ) -> impl ParallelIterator<Item = Result<(&'a T1, &'a T2, TimeInterval), Self::Error>>
    where
        T1: 'a + Send + Sync,
        T2: 'a + Send + Sync,
    {
        self.wrapped.par_iter().filter(|item| match item {
            Ok((t1, t2, interval)) => (self.filter)(t1, t2, *interval),
            Err(_) => true,
        })
        // (self.filter)(t1, t2, *interval))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::events::DetectFnExt;
    use crate::visibility::VisibilityError;
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

        let result = Analysis::new(&ground_assets, &space_assets, interval)
            .refine(|gs, sc, interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
                let elev = ElevationDetectFn {
                    gs: gs.location(),
                    mask: gs.mask(),
                    sc: sc_traj,
                };
                let windows = elev.intervals(UniformSampler::new(step), interval)?;
                Ok::<_, VisibilityError>(windows.into_iter())
            })
            .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000))
            .collect()
            .unwrap();

        let result_par = Analysis::new(&ground_assets, &space_assets, interval)
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
                Ok::<_, VisibilityError>(windows.into_iter())
            })
            .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000))
            .par_collect().unwrap();

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
