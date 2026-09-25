use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;

use futures::StreamExt;
use futures::TryStreamExt;
use futures::task::Spawn;
use futures::{Stream, channel::mpsc::unbounded};
use futures_scopes::relay::{RelayScope, new_relay_scope};
use futures_scopes::{ScopedSpawnExt, SpawnScope};
use lox_time::intervals::TimeInterval;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::assets::{AssetId, GroundStation, Spacecraft};
use crate::events::DetectError;

type PipelineItem<T, W, E> = Result<(T, Vec<W>), E>;
type AnalysisMap<K, V> = HashMap<K, Vec<V>>;

impl From<Infallible> for DetectError {
    fn from(value: Infallible) -> Self {
        unreachable!()
    }
}

impl AsKey<AssetId> for Spacecraft {
    fn as_key(&self) -> AssetId {
        self.id().clone()
    }
}

impl AsKey<AssetId> for GroundStation {
    fn as_key(&self) -> AssetId {
        self.id().clone()
    }
}

impl<'a, T1, T2> AsKey<(AssetId, AssetId)> for (&'a T1, &'a T2)
where
    T1: AsKey<AssetId>,
    T2: AsKey<AssetId>,
{
    fn as_key(&self) -> (AssetId, AssetId) {
        let (t1, t2) = self;
        (t1.as_key(), t2.as_key())
    }
}

pub struct AssetPairAnalysis<'a, T1, T2> {
    t1: &'a [T1],
    t2: &'a [T2],
    interval: TimeInterval,
}

impl<'a, T1, T2> AssetPairAnalysis<'a, T1, T2> {
    pub fn new(t1: &'a [T1], t2: &'a [T2], interval: TimeInterval) -> Self {
        Self { t1, t2, interval }
    }
}

impl<'a, T1, T2> Pipeline<'a> for AssetPairAnalysis<'a, T1, T2>
where
    T1: AsKey<AssetId>,
    T2: AsKey<AssetId>,
    (&'a T1, &'a T2): AsKey<(AssetId, AssetId)>,
{
    type Asset = (&'a T1, &'a T2);
    type Key = (AssetId, AssetId);
    type Window = TimeInterval;
    type Error = Infallible;

    fn iter(
        &self,
    ) -> impl Iterator<Item = PipelineItem<(&'a T1, &'a T2), TimeInterval, Self::Error>> + '_ {
        self.t1.iter().flat_map(move |item1| {
            self.t2
                .iter()
                .map(move |item2| Ok(((item1, item2), vec![self.interval])))
        })
    }
}

impl<'a, T1, T2> ParallelPipeline<'a> for AssetPairAnalysis<'a, T1, T2>
where
    T1: AsKey<AssetId> + Sync,
    T2: AsKey<AssetId> + Sync,
    (&'a T1, &'a T2): AsKey<(AssetId, AssetId)>,
{
    fn par_iter(
        &self,
    ) -> impl ParallelIterator<Item = PipelineItem<(&'a T1, &'a T2), Self::Window, Self::Error>> + '_
    {
        self.t1.par_iter().flat_map(move |item1| {
            self.t2
                .par_iter()
                .map(move |item2| Ok(((item1, item2), vec![self.interval])))
        })
    }
}

impl<'a, T1, T2> StreamPipeline<'a> for AssetPairAnalysis<'a, T1, T2>
where
    T1: AsKey<AssetId> + Sync,
    T2: AsKey<AssetId> + Sync,
    (&'a T1, &'a T2): AsKey<(AssetId, AssetId)>,
{
    async fn stream<'sc>(
        &'sc self,
        _scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<(&'a T1, &'a T2), Self::Window, Self::Error>> + 'sc
    where
        'a: 'sc,
    {
        futures::stream::iter(self.iter())
    }
}

pub trait AsKey<K> {
    fn as_key(&self) -> K;
}

pub trait Pipeline<'a>: Sized {
    type Asset: Copy + 'a;
    type Key: Eq + Hash + core::fmt::Debug;
    type Window: Copy;
    type Error;

    fn refine<R, I>(self, refinement: R) -> Refine<Self, R>
    where
        R: RefineFn<Self::Asset, Self::Window> + Fn(Self::Asset, Self::Window) -> I,
    {
        Refine {
            wrapped: self,
            refinement,
        }
    }

    fn filter<F>(self, filter: F) -> Filter<Self, F>
    where
        F: Fn(Self::Asset, Self::Window) -> bool,
    {
        Filter {
            wrapped: self,
            filter,
        }
    }

    fn filter_asset<F>(self, f: F) -> Filter<Self, impl Fn(Self::Asset, Self::Window) -> bool>
    where
        F: Fn(Self::Asset) -> bool + Sync + 'a,
    {
        self.filter(move |t, _| f(t))
    }

    fn filter_window<F>(self, f: F) -> Filter<Self, impl Fn(Self::Asset, Self::Window) -> bool>
    where
        F: Fn(Self::Window) -> bool + Sync + 'a,
    {
        self.filter(move |_, window| f(window))
    }

    fn iter(
        &self,
    ) -> impl Iterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_;

    fn collect(self) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey<Self::Key>,
    {
        self.iter()
            .map(|item| item.map(|(t, window)| (t.as_key(), window)))
            .collect()
    }
}

pub trait ParallelPipeline<'a>: Pipeline<'a>
where
    Self::Asset: Sync + Send,
    Self::Window: Send,
    Self::Error: Send,
{
    fn par_iter(
        &self,
    ) -> impl ParallelIterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_;

    fn par_collect(self) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey<Self::Key>,
        Self::Key: Send,
    {
        self.par_iter()
            .map(|item| item.map(|(t, window)| (t.as_key(), window)))
            .collect()
    }
}

pub trait StreamPipeline<'a>: Pipeline<'a>
where
    Self::Asset: Sync + Send,
    Self::Window: Send,
    Self::Error: Send,
{
    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + 'sc
    where
        'a: 'sc;

    async fn stream_collect<S>(
        self,
        spawner: &'a S,
    ) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey<Self::Key>,
        S: Spawn + Clone + Send,
    {
        let scope = new_relay_scope!(spawner);
        self.stream(scope)
            .await
            .map(|item| item.map(|(t, window)| (t.as_key(), window)))
            .try_collect()
            .await
    }
}

pub trait RefineFn<A, W> {
    type Error;
    type Iter: Iterator<Item = Result<W, Self::Error>>;

    fn refine(&self, asset: A, window: W) -> Self::Iter;
}

impl<T, A, W, I, E> RefineFn<A, W> for T
where
    T: Fn(A, W) -> I,
    I: Iterator<Item = Result<W, E>>,
{
    type Iter = I;
    type Error = E;

    fn refine(&self, asset: A, window: W) -> Self::Iter {
        self(asset, window)
    }
}

pub struct Refine<W, R> {
    wrapped: W,
    refinement: R,
}

impl<'a, W, R> Refine<W, R>
where
    W: Pipeline<'a>,
    R: RefineFn<W::Asset, W::Window>,
    R::Error: From<W::Error>,
{
    fn map_item(
        f: &R,
        item: PipelineItem<W::Asset, W::Window, W::Error>,
    ) -> PipelineItem<W::Asset, W::Window, R::Error> {
        let (t, windows) = item?;

        Ok((
            t,
            windows
                .into_iter()
                .flat_map(|window| f.refine(t, window))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl<'a, W, R> Pipeline<'a> for Refine<W, R>
where
    W: Pipeline<'a>,
    R: RefineFn<W::Asset, W::Window>,
    R::Error: From<W::Error>,
{
    type Asset = W::Asset;
    type Key = W::Key;
    type Window = W::Window;
    type Error = R::Error;

    fn iter(
        &self,
    ) -> impl Iterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_ {
        self.wrapped
            .iter()
            .map(move |item| Self::map_item(&self.refinement, item))
    }
}

impl<'a, W, R> ParallelPipeline<'a> for Refine<W, R>
where
    W: ParallelPipeline<'a> + Sync,
    W::Asset: Sync + Send,
    W::Window: Send,
    W::Error: Send,
    R: RefineFn<W::Asset, W::Window> + Sync,
    R::Error: From<W::Error> + Send,
    R::Iter: Send + Sync + 'a,
{
    fn par_iter(
        &self,
    ) -> impl ParallelIterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_
    {
        self.wrapped
            .par_iter()
            .map(move |item| Self::map_item(&self.refinement, item))
    }
}

impl<'a, W, R> StreamPipeline<'a> for Refine<W, R>
where
    W: StreamPipeline<'a> + Sync,
    W::Asset: Sync + Send,
    W::Window: Send,
    W::Error: Send,
    R: RefineFn<W::Asset, W::Window> + Sync,
    R::Error: From<W::Error> + Send,
    R::Iter: Send + Sync + 'a,
{
    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + 'sc
    where
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

pub struct Filter<W, F> {
    wrapped: W,
    filter: F,
}

impl<'a, W, F> Filter<W, F>
where
    W: Pipeline<'a>,
    F: Fn(W::Asset, W::Window) -> bool,
{
    fn filter_item<E>(
        f: &F,
        item: PipelineItem<W::Asset, W::Window, E>,
    ) -> Option<PipelineItem<W::Asset, W::Window, E>> {
        match item {
            Ok((t, windows)) => {
                let windows = windows
                    .into_iter()
                    .filter(|window| f(t, *window))
                    .collect::<Vec<_>>();

                if windows.len() > 0 {
                    Some(Ok((t, windows)))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a, W, F> Pipeline<'a> for Filter<W, F>
where
    W: Pipeline<'a>,
    F: Fn(W::Asset, W::Window) -> bool,
{
    type Asset = W::Asset;
    type Key = W::Key;
    type Window = W::Window;
    type Error = W::Error;

    fn iter(
        &self,
    ) -> impl Iterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_ {
        self.wrapped
            .iter()
            .filter_map(|item| Self::filter_item(&self.filter, item))
    }
}

impl<'a, W, F> ParallelPipeline<'a> for Filter<W, F>
where
    W: ParallelPipeline<'a> + Sync,
    W::Asset: Sync + Send,
    W::Window: Send,
    W::Error: Send,
    F: Fn(W::Asset, W::Window) -> bool + Sync,
{
    fn par_iter(
        &self,
    ) -> impl ParallelIterator<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + '_
    {
        self.wrapped
            .par_iter()
            .filter_map(|item| Self::filter_item(&self.filter, item))
    }
}

impl<'a, W, F> StreamPipeline<'a> for Filter<W, F>
where
    W: StreamPipeline<'a> + Sync,
    W::Asset: Sync + Send,
    W::Window: Send,
    W::Error: Send,
    F: Fn(W::Asset, W::Window) -> bool + Sync,
{
    async fn stream<'sc>(
        &'sc self,
        scope: &RelayScope<'sc>,
    ) -> impl Stream<Item = PipelineItem<Self::Asset, Self::Window, Self::Error>> + 'sc
    where
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
        visibility::{ElevationDetectFn},
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
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let gs = GroundStation::new("cebreros", gs_loc);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let step = TimeDelta::from_seconds(60);

        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let result = AssetPairAnalysis::new(&ground_assets, &space_assets, interval)
            .refine(|(gs, sc), interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
                let elev = ElevationDetectFn {
                    gs: gs,
                    sc: sc_traj,
                };
                elev.into_intervals(UniformSampler::new(step), interval)
            })
            .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
            .collect()
            .unwrap();

        let result_par = AssetPairAnalysis::new(&ground_assets, &space_assets, interval)
            .refine(|(gs, sc), interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                    "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
        	);
                let elev = ElevationDetectFn {
                    gs: gs,
                    sc: sc_traj,
                };

		elev.into_intervals(UniformSampler::new(step), interval)
            })
            .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
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
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let gs = GroundStation::new("cebreros", gs_loc);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let step = TimeDelta::from_seconds(60);

        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let result = AssetPairAnalysis::new(&ground_assets, &space_assets, interval)
            .refine(|(gs, sc), interval| {
                let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
                let elev = ElevationDetectFn {
                    gs: gs,
                    sc: sc_traj,
                };
                elev.into_intervals(UniformSampler::new(step), interval)
            })
            .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
            .collect()
            .unwrap();

        let result_stream = {
            let pool = ThreadPool::new().unwrap();

            let pipeline = AssetPairAnalysis::new(&ground_assets, &space_assets, interval)
                .refine(|(gs, sc), interval| {
                    let sc_traj = ensemble.get(sc.id()).expect(
                        "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
                    );
                    let elev = ElevationDetectFn {
                        gs: gs,
                        sc: sc_traj,
                    };

                    elev.into_intervals(UniformSampler::new(step), interval)

                })
                .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000));

            block_on(pipeline.stream_collect(&pool)).unwrap()
        };

        assert_eq!(result, result_stream);
        // dbg!(result);
    }
}
