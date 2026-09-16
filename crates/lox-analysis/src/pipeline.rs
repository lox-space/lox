use std::{convert::Infallible, marker::PhantomData};

use futures::StreamExt;
use futures::TryStreamExt;
use futures::task::Spawn;
use futures::{Stream, channel::mpsc::unbounded};
use futures_scopes::relay::{RelayScope, new_relay_scope};
use futures_scopes::{ScopedSpawnExt, SpawnScope};
use lox_time::intervals::TimeInterval;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::events::DetectError;
use crate::{
    assets::{AssetId, GroundStation, Spacecraft},
    visibility::{IntervalMap, VisibilityError},
};

type PipelineItem<'a, T1, T2, E> = Result<(&'a T1, &'a T2, Vec<TimeInterval>), E>;

impl From<Infallible> for DetectError {
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

impl<'a, T1, T2> Pipeline<'a, T1, T2> for Analysis<'a, T1, T2>
where
    T1: AssetIdExt,
    T2: AssetIdExt,
{
    type Error = Infallible;

    fn iter(&self) -> impl Iterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_ {
        self.t1.iter().flat_map(move |item1| {
            self.t2
                .iter()
                .map(move |item2| Ok((item1, item2, vec![self.interval])))
        })
    }

    fn par_iter(&self) -> impl ParallelIterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_
    where
        T1: Sync,
        T2: Sync,
    {
        self.t1.par_iter().flat_map(move |item1| {
            self.t2
                .par_iter()
                .map(move |item2| Ok((item1, item2, vec![self.interval])))
        })
    }

    async fn stream<'sc>(
        &'sc self,
        _scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<'a, T1, T2, Self::Error>> + 'sc
    where
        T1: Sync,
        T2: Sync,
        'a: 'sc,
    {
        futures::stream::iter(self.iter())
    }
}

trait Pipeline<'a, T1, T2>: Sized
where
    T1: AssetIdExt + 'a,
    T2: AssetIdExt + 'a,
{
    type Error: Send;

    fn refine<R, I, E>(self, refinement: R) -> Refine<T1, T2, Self, R, I, E>
    where
        R: Fn(&'a T1, &'a T2, TimeInterval) -> I,
        I: Iterator<Item = Result<TimeInterval, E>> + 'a,
    {
        Refine {
            wrapped: self,
            refinement,
            __: PhantomData,
        }
    }

    fn filter<F>(self, filter: F) -> Filter<T1, T2, Self, F>
    where
        F: Fn(&T1, &T2, TimeInterval) -> bool,
    {
        Filter {
            wrapped: self,
            filter,
            __: PhantomData,
        }
    }

    fn iter(&self) -> impl Iterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_;

    fn collect(self) -> Result<IntervalMap, Self::Error> {
        self.iter()
            .map(|item| item.map(|(t1, t2, interval)| ((t1.asset_id(), t2.asset_id()), interval)))
            .collect()
    }

    fn par_iter(&self) -> impl ParallelIterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_
    where
        T1: Sync,
        T2: Sync;

    fn par_collect(self) -> Result<IntervalMap, Self::Error>
    where
        T1: Sync,
        T2: Sync,
    {
        self.par_iter()
            .map(|item| item.map(|(t1, t2, interval)| ((t1.asset_id(), t2.asset_id()), interval)))
            .collect()
    }

    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<'a, T1, T2, Self::Error>> + 'sc
    where
        T1: Sync,
        T2: Sync,
        'a: 'sc;

    async fn stream_collect<S>(self, spawner: &'a S) -> Result<IntervalMap, Self::Error>
    where
        T1: Sync,
        T2: Sync,
        S: Spawn + Clone + Send,
    {
        let scope = new_relay_scope!(spawner);
        self.stream(scope)
            .await
            .map(|item| item.map(|(t1, t2, intervals)| ((t1.asset_id(), t2.asset_id()), intervals)))
            .try_collect()
            .await
    }
}

struct Refine<T1, T2, W, R, I, E> {
    wrapped: W,
    refinement: R,
    __: PhantomData<(T1, T2, I, E)>,
}

impl<'a, T1, T2, W, R, I, E> Refine<T1, T2, W, R, I, E>
where
    T1: AssetIdExt + 'a,
    T2: AssetIdExt + 'a,
    W: Pipeline<'a, T1, T2>,
    R: Fn(&'a T1, &'a T2, TimeInterval) -> I,
    I: Iterator<Item = Result<TimeInterval, E>> + 'a,
    E: From<W::Error>,
{
    fn map_item(f: &R, item: PipelineItem<'a, T1, T2, W::Error>) -> PipelineItem<'a, T1, T2, E> {
        let (t1, t2, intervals) = item?;

        let mut sub_intervals = Vec::new();
        for interval in intervals {
            for sub_interval in f(&t1, &t2, interval) {
                sub_intervals.push(sub_interval?);
            }
        }

        Ok::<_, E>((t1, t2, sub_intervals))
    }
}

impl<'a, T1, T2, W, R, I, E> Pipeline<'a, T1, T2> for Refine<T1, T2, W, R, I, E>
where
    T1: AssetIdExt + 'a,
    T2: AssetIdExt + 'a,
    W: Pipeline<'a, T1, T2> + Sync,
    R: Fn(&'a T1, &'a T2, TimeInterval) -> I + Sync,
    I: Iterator<Item = Result<TimeInterval, E>> + Sync + 'a,
    E: From<W::Error> + Send + Sync,
{
    type Error = E;

    fn iter(&self) -> impl Iterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_ {
        self.wrapped
            .iter()
            .map(move |item| Self::map_item(&self.refinement, item))
    }

    fn par_iter(&self) -> impl ParallelIterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_
    where
        T1: Sync,
        T2: Sync,
    {
        self.wrapped
            .par_iter()
            .map(move |item| Self::map_item(&self.refinement, item))
    }

    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<'a, T1, T2, Self::Error>> + 'sc
    where
        T1: Sync,
        T2: Sync,
        'a: 'sc,
    {
        let (tx, rx) = unbounded();

        self.wrapped
            .stream(scope)
            .await
            .for_each(async |item| {
                let tx = tx.clone();

                scope
                    .spawner()
                    .spawn_scoped(async move {
                        tx.unbounded_send(Self::map_item(&self.refinement, item))
                            .unwrap();
                    })
                    .unwrap();
            })
            .await;

        rx
    }
}

struct Filter<T1, T2, W, F> {
    wrapped: W,
    filter: F,
    __: PhantomData<(T1, T2)>,
}

impl<'a, T1, T2, W, F> Filter<T1, T2, W, F>
where
    T1: AssetIdExt + 'a,
    T2: AssetIdExt + 'a,
    F: Fn(&'a T1, &'a T2, TimeInterval) -> bool,
{
    fn filter_item<E>(
        f: &F,
        item: PipelineItem<'a, T1, T2, E>,
    ) -> Option<PipelineItem<'a, T1, T2, E>> {
        match item {
            Ok((t1, t2, intervals)) => {
                let intervals = intervals
                    .into_iter()
                    .filter(|interval| f(&t1, &t2, *interval))
                    .collect::<Vec<_>>();

                if intervals.len() > 0 {
                    Some(Ok((t1, t2, intervals)))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a, T1, T2, W, F> Pipeline<'a, T1, T2> for Filter<T1, T2, W, F>
where
    T1: AssetIdExt + 'a,
    T2: AssetIdExt + 'a,
    W: Pipeline<'a, T1, T2> + Sync,
    F: Fn(&'a T1, &'a T2, TimeInterval) -> bool + Sync,
{
    type Error = W::Error;

    fn iter(&self) -> impl Iterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_ {
        self.wrapped
            .iter()
            .filter_map(|item| Self::filter_item(&self.filter, item))
    }

    fn par_iter(&self) -> impl ParallelIterator<Item = PipelineItem<'a, T1, T2, Self::Error>> + '_
    where
        T1: Sync,
        T2: Sync,
    {
        self.wrapped
            .par_iter()
            .filter_map(|item| Self::filter_item(&self.filter, item))
    }

    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<'a, T1, T2, Self::Error>> + 'sc
    where
        T1: Sync,
        T2: Sync,
        'a: 'sc,
    {
        let (tx, rx) = unbounded();

        self.wrapped
            .stream(scope)
            .await
            .for_each(async |item| {
                let tx = tx.clone();
                scope
                    .spawner()
                    .spawn_scoped(async move {
                        tx.unbounded_send(Self::filter_item(&self.filter, item))
                            .unwrap();
                    })
                    .unwrap();
            })
            .await;

        rx.filter_map(async |item| item)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::visibility::VisibilityError;
    use crate::{assets::Scenario, events::DetectFnExt};
    use futures::executor::{ThreadPool, block_on};
    use lox_bodies::Origin;
    use lox_core::{coords::LonLatAlt, units::Angle};
    use lox_frames::Frame;
    use lox_orbits::orbits::Ensemble;
    use lox_orbits::{ground::EllipsoidLocation, orbits::Trajectory, propagators::OrbitSource};
    use lox_test_utils::read_data_file;
    use lox_time::deltas::TimeDelta;
    use lox_time::time_scales::TimeScale;

    use crate::{
        events::UniformSampler,
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
                elev.into_intervals(UniformSampler::new(step), interval)
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

		elev.into_intervals(UniformSampler::new(step), interval)
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
    }

    #[test]
    fn refinement_stream() {
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
                elev.into_intervals(UniformSampler::new(step), interval)
            })
            .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000))
            .collect()
            .unwrap();

        let result_stream = {
            let pool = ThreadPool::new().unwrap();

            let filter = Analysis::new(&ground_assets, &space_assets, interval)
                .refine(|gs, sc, interval| {
                    let sc_traj = ensemble.get(sc.id()).expect(
                        "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
                    );
                    let elev = ElevationDetectFn {
                        gs: gs.location(),
                        mask: gs.mask(),
                        sc: sc_traj,
                    };

                    elev.into_intervals(UniformSampler::new(step), interval)

                })
                .filter(|_, _, interval| interval.duration() > TimeDelta::from_seconds(40000));
            let pipeline = filter;

            block_on(pipeline.stream_collect(&pool)).unwrap()
        };

        assert_eq!(result, result_stream);
        // dbg!(result);
    }
}
