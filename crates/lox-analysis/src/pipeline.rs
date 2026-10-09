use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;

use futures::stream::{FuturesOrdered, FuturesUnordered};
use futures::task::Spawn;
use futures::{StreamExt, TryStreamExt};
use futures_scopes::relay::new_relay_scope;
use futures_scopes::{ScopedSpawnExt, SpawnScope};
use itertools::Either;
use lox_time::intervals::TimeInterval;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::assets::{AssetId, GroundStation, Spacecraft};

type AnalysisMap<K, V> = HashMap<K, Vec<V>>;

pub trait AsKey {
    type Key: Eq + Hash + core::fmt::Debug;

    fn as_key(&self) -> Self::Key;
}

impl AsKey for Spacecraft {
    type Key = AssetId;

    fn as_key(&self) -> AssetId {
        self.id().clone()
    }
}

impl AsKey for GroundStation {
    type Key = AssetId;

    fn as_key(&self) -> AssetId {
        self.id().clone()
    }
}

impl<'a, T1, T2> AsKey for (&'a T1, &'a T2)
where
    T1: AsKey<Key = AssetId>,
    T2: AsKey<Key = AssetId>,
{
    type Key = (AssetId, AssetId);

    fn as_key(&self) -> (AssetId, AssetId) {
        let (t1, t2) = self;
        (t1.as_key(), t2.as_key())
    }
}

pub trait ConvertResult<IN, OUT> {
    fn convert<T>(input: Result<T, IN>) -> Result<T, OUT>;
}

pub enum FallibleResult {}

impl<IN, OUT> ConvertResult<IN, OUT> for FallibleResult
where
    OUT: From<IN>,
{
    fn convert<T>(input: Result<T, IN>) -> Result<T, OUT> {
        Ok(input?)
    }
}

pub enum InfallibleResult {}

impl<IN, OUT> ConvertResult<IN, OUT> for InfallibleResult
where
    IN: core::fmt::Debug,
{
    fn convert<T>(input: Result<T, IN>) -> Result<T, OUT> {
        Ok(input.unwrap())
    }
}

pub trait Pipeline<'a>: Sized {
    type Asset: 'a;
    type Window: Clone;
    type Error: core::fmt::Debug;
    type ConvertResult;

    fn map_asset<F, O, I>(self, f: F) -> MapAsset<Self, F>
    where
        F: Fn(Self::Asset) -> I,
        I: Iterator<Item = O>,
    {
        MapAsset { wrapped: self, f }
    }

    fn refine<R, I>(self, refinement: R) -> Refine<Self, R>
    where
        R: RefineFn<Self::Asset, Self::Window>
            + Fn(&Self::Asset, Self::Window) -> Result<I, R::Error>,
    {
        Refine {
            wrapped: self,
            refinement,
        }
    }

    fn filter<F>(self, filter: F) -> Filter<Self, F>
    where
        F: Fn(&Self::Asset, &Self::Window) -> bool,
    {
        Filter {
            wrapped: self,
            filter,
        }
    }

    fn filter_asset<F>(self, f: F) -> Filter<Self, impl Fn(&Self::Asset, &Self::Window) -> bool>
    where
        F: Fn(&Self::Asset) -> bool + Sync + 'a,
    {
        self.filter(move |t, _| f(t))
    }

    fn filter_window<F>(self, f: F) -> Filter<Self, impl Fn(&Self::Asset, &Self::Window) -> bool>
    where
        F: Fn(&Self::Window) -> bool + Sync + 'a,
    {
        self.filter(move |_, window| f(window))
    }

    fn batches(&self) -> usize;

    fn iter_batch(
        &self,
        batch: usize,
    ) -> impl Iterator<Item = Result<(Self::Asset, Vec<Self::Window>), Self::Error>>;

    fn compute_batch(
        &self,
        batch: usize,
    ) -> Result<AnalysisMap<<Self::Asset as AsKey>::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey,
    {
        self.iter_batch(batch)
            .filter_map(|item| match item {
                Ok((asset, windows)) => {
                    if !windows.is_empty() {
                        Some(Ok((asset.as_key(), windows)))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err)),
            })
            .collect()
    }

    fn compute(self) -> Result<AnalysisMap<<Self::Asset as AsKey>::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey,
    {
        (0..self.batches())
            .map(|batch| self.compute_batch(batch))
            .fold(Ok(HashMap::new()), |map, batch_map| {
                let mut map = map?;
                let batch_map = batch_map?;

                map.extend(batch_map);

                Ok(map)
            })
    }
}

pub trait ParallelPipeline<'a>: Pipeline<'a> {
    fn par_collect(
        self,
    ) -> Result<AnalysisMap<<Self::Asset as AsKey>::Key, Self::Window>, Self::Error>
    where
        Self: Sync,
        Self::Asset: AsKey + Sync + Send,
        Self::Window: Send,
        Self::Error: Send,
        <Self::Asset as AsKey>::Key: Send,
    {
        (0..self.batches())
            .into_par_iter()
            .map(|batch| self.compute_batch(batch))
            .reduce(
                || Ok(HashMap::new()),
                |map, batch_map| {
                    let mut map = map?;
                    let batch_map = batch_map?;

                    map.extend(batch_map);

                    Ok(map)
                },
            )
    }
}

impl<'a, T> ParallelPipeline<'a> for T where T: Pipeline<'a> {}

pub trait StreamPipeline<'a>: Pipeline<'a> {
    async fn stream_collect<S>(
        self,
        spawner: &'a S,
    ) -> Result<AnalysisMap<<Self::Asset as AsKey>::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey + Sync + Send,
        S: Spawn + Clone + Send,
        Self: Send + Sync + 'a,
        <Self::Asset as AsKey>::Key: Send + 'static,
        Self::Window: Send + 'static,
        Self::Error: Send + 'static,
    {
        let this = &self;
        let scope = new_relay_scope!(spawner);

        let handles = (0..self.batches())
            .map(|batch| {
                scope
                    .spawner()
                    .spawn_scoped_with_handle(async move { this.compute_batch(batch) })
                    .unwrap()
            })
            .collect::<FuturesUnordered<_>>();

        handles
            .fold(Ok(HashMap::new()), async |map, batch_map| {
                let mut map = map?;
                let batch_map = batch_map?;

                map.extend(batch_map);

                Ok(map)
            })
            .await
    }
}

impl<'a, T> StreamPipeline<'a> for T where T: Pipeline<'a> {}

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
    T1: AsKey,
    T2: AsKey,
    (&'a T1, &'a T2): AsKey,
{
    type Asset = (&'a T1, &'a T2);
    type Window = TimeInterval;
    type Error = Infallible;
    type ConvertResult = InfallibleResult;

    fn batches(&self) -> usize {
        self.t1.len() * self.t2.len()
    }

    fn iter_batch(
        &self,
        batch: usize,
    ) -> impl Iterator<Item = Result<(Self::Asset, Vec<Self::Window>), Self::Error>> {
        let i = batch % self.t1.len();
        let j = batch / self.t1.len();
        let asset = (&self.t1[i], &self.t2[j]);

        std::iter::once(Ok((asset, vec![self.interval])))
    }
}

pub trait MapFn<'a, IN> {
    type Out: 'a;
    type Iter: Iterator<Item = Self::Out>;

    fn map(&self, input: IN) -> Self::Iter;
}

impl<'a, IN, OUT, F, I> MapFn<'a, IN> for F
where
    OUT: 'a,
    F: Fn(IN) -> I,
    I: Iterator<Item = OUT>,
{
    type Out = OUT;
    type Iter = I;

    fn map(&self, input: IN) -> I {
        self(input)
    }
}

pub struct MapAsset<W, F> {
    wrapped: W,
    f: F,
}

impl<'a, W, F> Pipeline<'a> for MapAsset<W, F>
where
    W: Pipeline<'a>,
    F: MapFn<'a, W::Asset>,
{
    type Asset = F::Out;
    type Window = W::Window;
    type Error = W::Error;
    type ConvertResult = W::ConvertResult;

    fn batches(&self) -> usize {
        self.wrapped.batches()
    }

    fn iter_batch(
        &self,
        batch: usize,
    ) -> impl Iterator<Item = Result<(Self::Asset, Vec<Self::Window>), Self::Error>> {
        self.wrapped.iter_batch(batch).flat_map(|item| match item {
            Ok((asset, windows)) => Either::Left(
                self.f
                    .map(asset)
                    .map(move |asset| Ok((asset, windows.clone()))),
            ),
            Err(err) => Either::Right(std::iter::once(Err(err))),
        })
    }
}

pub trait RefineFn<A, W> {
    type Error;
    type OutWindow: Clone;
    type Iter: Iterator<Item = Result<Self::OutWindow, Self::Error>>;

    fn refine(&self, asset: &A, window: W) -> Result<Self::Iter, Self::Error>;
}

impl<T, A, WI, WO, E, I> RefineFn<A, WI> for T
where
    T: Fn(&A, WI) -> Result<I, E>,
    I: Iterator<Item = Result<WO, E>>,
    WO: Clone,
{
    type Error = E;
    type OutWindow = WO;
    type Iter = I;

    fn refine(&self, asset: &A, window: WI) -> Result<Self::Iter, Self::Error> {
        self(asset, window)
    }
}

pub struct Refine<W, R> {
    wrapped: W,
    refinement: R,
}

impl<'a, W, R> Pipeline<'a> for Refine<W, R>
where
    W: Pipeline<'a>,
    R: RefineFn<W::Asset, W::Window>,
    R::Error: core::fmt::Debug,
    W::ConvertResult: ConvertResult<W::Error, R::Error>,
{
    type Asset = W::Asset;
    type Window = R::OutWindow;
    type Error = R::Error;
    type ConvertResult = FallibleResult;

    fn batches(&self) -> usize {
        self.wrapped.batches()
    }

    fn iter_batch(
        &self,
        batch: usize,
    ) -> impl Iterator<Item = Result<(Self::Asset, Vec<Self::Window>), Self::Error>> {
        self.wrapped.iter_batch(batch).map(move |item| {
            let (asset, windows) = W::ConvertResult::convert(item)?;

            let mut sub_windows = Vec::new();

            for window in windows {
                for sub_window in self.refinement.refine(&asset, window)? {
                    sub_windows.push(sub_window?);
                }
            }

            Ok((asset, sub_windows))
        })
    }
}

pub struct Filter<W, F> {
    wrapped: W,
    filter: F,
}

impl<'a, W, F> Pipeline<'a> for Filter<W, F>
where
    W: Pipeline<'a>,
    F: Fn(&W::Asset, &W::Window) -> bool,
{
    type Asset = W::Asset;
    type Window = W::Window;
    type Error = W::Error;
    type ConvertResult = W::ConvertResult;

    fn batches(&self) -> usize {
        self.wrapped.batches()
    }

    fn iter_batch(
        &self,
        batch: usize,
    ) -> impl Iterator<Item = Result<(Self::Asset, Vec<Self::Window>), Self::Error>> {
        self.wrapped.iter_batch(batch).map(move |item| {
            let (asset, windows) = item?;

            let windows = windows
                .into_iter()
                .filter(|window| (self.filter)(&asset, window))
                .collect();

            Ok((asset, windows))
        })
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

    use crate::{events::UniformSampler, visibility::ElevationDetectFn};

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

                Ok(elev.into_intervals(UniformSampler::new(step), interval))
            })
            .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
            .compute()
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

		Ok(elev.into_intervals(UniformSampler::new(step), interval))
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
                Ok(elev.into_intervals(UniformSampler::new(step), interval))
            })
            .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
            .compute()
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

                    Ok(elev.into_intervals(UniformSampler::new(step), interval))
                })
                .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000));

            block_on(pipeline.stream_collect(&pool)).unwrap()
        };

        assert_eq!(result, result_stream);
    }
}
