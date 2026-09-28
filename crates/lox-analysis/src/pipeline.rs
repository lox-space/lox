use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;

use futures::stream::FuturesOrdered;
use futures::task::Spawn;
use futures::{StreamExt, TryStreamExt};
use futures_scopes::relay::new_relay_scope;
use futures_scopes::{ScopedSpawnExt, SpawnScope};
use lox_time::intervals::TimeInterval;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::assets::{AssetId, GroundStation, Spacecraft};

type AnalysisMap<K, V> = HashMap<K, Vec<V>>;

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

pub trait AsKey<K> {
    fn as_key(&self) -> K;
}

pub trait Pipeline<'a>: Sized {
    type Asset: Copy + 'a;
    type Key: Eq + Hash + core::fmt::Debug;
    type Window;
    type Error: core::fmt::Debug;
    type ConvertResult;

    fn refine<R, I>(self, refinement: R) -> Refine<Self, R>
    where
        R: RefineFn<Self::Asset, Self::Window>
            + Fn(Self::Asset, Self::Window) -> Result<I, R::Error>,
    {
        Refine {
            wrapped: self,
            refinement,
        }
    }

    fn filter<F>(self, filter: F) -> Filter<Self, F>
    where
        F: Fn(Self::Asset, &Self::Window) -> bool,
    {
        Filter {
            wrapped: self,
            filter,
        }
    }

    fn filter_asset<F>(self, f: F) -> Filter<Self, impl Fn(Self::Asset, &Self::Window) -> bool>
    where
        F: Fn(Self::Asset) -> bool + Sync + 'a,
    {
        self.filter(move |t, _| f(t))
    }

    fn filter_window<F>(self, f: F) -> Filter<Self, impl Fn(Self::Asset, &Self::Window) -> bool>
    where
        F: Fn(&Self::Window) -> bool + Sync + 'a,
    {
        self.filter(move |_, window| f(window))
    }

    fn compute_one(
        &self,
        asset: Self::Asset,
        window: Self::Window,
    ) -> Result<impl Iterator<Item = Result<Self::Window, Self::Error>>, Self::Error>;

    fn initial_window(&self) -> Self::Window;

    fn collect_one(
        &self,
        asset: Self::Asset,
        window: Self::Window,
    ) -> Result<(Self::Key, Vec<Self::Window>), Self::Error>
    where
        Self::Asset: AsKey<Self::Key>,
    {
        self.compute_one(asset, window)?
            .collect::<Result<Vec<_>, _>>()
            .map(|windows| (asset.as_key(), windows))
    }

    fn filter_emtpy(item: &Result<(Self::Key, Vec<Self::Window>), Self::Error>) -> bool {
        match item {
            Ok((_, windows)) => !windows.is_empty(),
            Err(_) => true,
        }
    }

    fn collect(self) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey<Self::Key>,
    {
        self.iter_assets()
            .map(|asset| self.collect_one(asset, self.initial_window()))
            .filter(Self::filter_emtpy)
            .collect()
    }

    fn iter_assets(&self) -> impl Iterator<Item = Self::Asset>;
}

pub trait ParallelPipeline<'a>: Pipeline<'a> {
    fn par_collect(self) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self: Sync,
        Self::Asset: AsKey<Self::Key> + Sync + Send,
        Self::Window: Send,
        Self::Error: Send,
        Self::Key: Send,
    {
        let assets = self.iter_assets().collect::<Vec<_>>();

        assets
            .into_par_iter()
            .map(|asset| self.collect_one(asset, self.initial_window()))
            .filter(Self::filter_emtpy)
            .collect()
    }
}

impl<'a, T> ParallelPipeline<'a> for T where T: Pipeline<'a> {}

pub trait StreamPipeline<'a>: Pipeline<'a> {
    async fn stream_collect<S>(
        self,
        spawner: &'a S,
    ) -> Result<AnalysisMap<Self::Key, Self::Window>, Self::Error>
    where
        Self::Asset: AsKey<Self::Key> + Sync + Send,
        S: Spawn + Clone + Send,
        Self: Send + Sync + 'a,
        Self::Key: Send + 'static,
        Self::Window: Send + 'static,
        Self::Error: Send + 'static,
    {
        let this = &self;
        let scope = new_relay_scope!(spawner);

        let handles = self
            .iter_assets()
            .map(|asset| {
                scope
                    .spawner()
                    .spawn_scoped_with_handle(async move {
                        let window = this.initial_window();
                        this.collect_one(asset, window)
                    })
                    .unwrap()
            })
            .collect::<FuturesOrdered<_>>();

        handles
            .filter(|item| futures::future::ready(Self::filter_emtpy(item)))
            .try_collect()
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
    T1: AsKey<AssetId>,
    T2: AsKey<AssetId>,
    (&'a T1, &'a T2): AsKey<(AssetId, AssetId)>,
{
    type Asset = (&'a T1, &'a T2);
    type Key = (AssetId, AssetId);
    type Window = TimeInterval;
    type Error = Infallible;
    type ConvertResult = InfallibleResult;

    fn compute_one(
        &self,
        _asset: Self::Asset,
        window: Self::Window,
    ) -> Result<impl Iterator<Item = Result<Self::Window, Self::Error>>, Self::Error> {
        Ok(std::iter::once(Ok(window)))
    }

    fn initial_window(&self) -> Self::Window {
        self.interval
    }

    fn iter_assets(&self) -> impl Iterator<Item = Self::Asset> {
        self.t1
            .iter()
            .flat_map(move |item1| self.t2.iter().map(move |item2| (item1, item2)))
    }
}

pub trait RefineFn<A, W> {
    type Error;
    type Iter: Iterator<Item = Result<W, Self::Error>>;

    fn refine(&self, asset: A, window: W) -> Result<Self::Iter, Self::Error>;
}

impl<T, A, W, E, I> RefineFn<A, W> for T
where
    T: Fn(A, W) -> Result<I, E>,
    I: Iterator<Item = Result<W, E>>,
{
    type Error = E;
    type Iter = I;

    fn refine(&self, asset: A, window: W) -> Result<Self::Iter, Self::Error> {
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
    type Key = W::Key;
    type Window = W::Window;
    type Error = R::Error;
    type ConvertResult = FallibleResult;

    fn compute_one(
        &self,
        asset: Self::Asset,
        window: Self::Window,
    ) -> Result<impl Iterator<Item = Result<Self::Window, Self::Error>>, Self::Error> {
        W::ConvertResult::convert(self.wrapped.compute_one(asset, window)).map(|windows| {
            windows
                .flat_map(move |window| {
                    let window = W::ConvertResult::convert(window);
                    window
                        .map(|window| self.refinement.refine(asset, window))
                        .flatten()
                })
                .flatten()
        })
    }

    fn initial_window(&self) -> Self::Window {
        self.wrapped.initial_window()
    }

    fn iter_assets(&self) -> impl Iterator<Item = Self::Asset> {
        self.wrapped.iter_assets()
    }
}

pub struct Filter<W, F> {
    wrapped: W,
    filter: F,
}

impl<'a, W, F> Pipeline<'a> for Filter<W, F>
where
    W: Pipeline<'a>,
    F: Fn(W::Asset, &W::Window) -> bool,
{
    type Asset = W::Asset;
    type Key = W::Key;
    type Window = W::Window;
    type Error = W::Error;
    type ConvertResult = W::ConvertResult;

    fn compute_one(
        &self,
        asset: Self::Asset,
        window: Self::Window,
    ) -> Result<impl Iterator<Item = Result<Self::Window, Self::Error>>, Self::Error> {
        self.wrapped.compute_one(asset, window).map(|windows| {
            windows.filter(move |window| match window {
                Ok(window) => (self.filter)(asset, window),
                Err(_) => true,
            })
        })
    }

    fn initial_window(&self) -> Self::Window {
        self.wrapped.initial_window()
    }

    fn iter_assets(&self) -> impl Iterator<Item = Self::Asset> {
        self.wrapped.iter_assets()
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

                Ok(elev.into_intervals(UniformSampler::new(step), interval))
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

                    Ok(elev.into_intervals(UniformSampler::new(step), interval))
                })
                .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000));

            block_on(pipeline.stream_collect(&pool)).unwrap()
        };

        assert_eq!(result, result_stream);
    }
}
