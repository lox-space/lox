use lox_bodies::{CoordinateOrigin, Origin, TrySpheroid};
use lox_core::units::{AngularRate, Distance};
use lox_frames::{
    Frame, ReferenceFrame, providers::DefaultRotationProvider, rotations::TryRotation,
};
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::{deltas::TimeDelta, intervals::TimeInterval, time_scales::TimeScale};

use crate::events::{AdaptiveSampler, IntervalIterExt, Intervals, Sampler};
use crate::{
    assets::{AssetId, GroundStation, Spacecraft},
    events::{DetectError, DetectFn, DetectFnExt, UniformSampler},
    pipeline::{AssetIdExt, Pipeline},
    power::SpacecraftFilter,
    visibility::{
        ElevationDetectFn, InterSatLosCentralBodyDetectFn, InterSatelliteRangeDetectFn,
        InterSatelliteSlewRateDetectFn, RangeDirection,
    },
};

pub trait GroundSpacePipeline<'a>: Pipeline<'a, GroundStation, Spacecraft>
where
    Self: Sync,
{
    fn refine_visibility<O, R, F, S>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        sampler: F,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        S: Sampler<ElevationDetectFn<'a, O, R>> + Sync + 'a,
        F: Fn(TimeInterval) -> S + Sync,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        DetectError: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error>,
    {
        self.refine(move |gs, sc, interval| {
            let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let elev = ElevationDetectFn {
                gs: gs.location(),
                mask: gs.mask(),
                sc: sc_traj,
            };
            elev.into_intervals(sampler(interval), interval)
        })
    }

    fn refine_visibility_uniform<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        DetectError: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error>,
    {
        self.refine_visibility(ensemble, move |_| UniformSampler::new(step))
    }

    fn refine_visibility_adaptive<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        DetectError: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error>,
    {
        self.refine_visibility(ensemble, move |interval| {
            AdaptiveSampler::new(step, interval.duration().max(step))
        })
    }
}

impl<'a, T> GroundSpacePipeline<'a> for T where T: Pipeline<'a, GroundStation, Spacecraft> + Sync {}

struct OptionalIterator<I>(Option<I>);

impl<I, E> Iterator for OptionalIterator<I>
where
    I: Iterator<Item = Result<TimeInterval, E>>,
{
    type Item = Result<TimeInterval, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Some(iter) => iter.next(),
            None => None,
        }
    }
}

pub trait InterSatelitePipeline<'a>: Pipeline<'a, Spacecraft, Spacecraft>
where
    Self: Sync,
{
    fn refine_trajectories<O, R, F, I, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        f: F,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        F: Fn(
                (&'a Spacecraft, &'a Trajectory<O, R>),
                (&'a Spacecraft, &'a Trajectory<O, R>),
                TimeInterval,
            ) -> I
            + Sync,
        I: Iterator<Item = Result<TimeInterval, E>> + Send + Sync + 'a,
        E: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error> + Send + Sync,
    {
        self.refine(move |sc1, sc2, interval| {
            let traj1 = ensemble
                .get(sc1.id())
                .expect("trajectory not found in ensemble");
            let traj2 = ensemble
                .get(sc2.id())
                .expect("trajectory not found in ensemble");

            f((sc1, traj1), (sc2, traj2), interval)
        })
    }

    fn refine_visibility_intersect<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        min: Distance,
        max: Distance,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DetectError: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1), (_, traj2), interval| {
            let min = InterSatelliteRangeDetectFn {
                sc1: traj1,
                sc2: traj2,
                threshold: min,
                direction: RangeDirection::Min,
            }
            .into_intervals(UniformSampler::new(step), interval);
            let max = InterSatelliteRangeDetectFn {
                sc1: traj1,
                sc2: traj2,
                threshold: max,
                direction: RangeDirection::Max,
            }
            .into_intervals(UniformSampler::new(step), interval);

            max.intersect(min)
        })
    }

    fn refine_visibility<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        threshold: Distance,
        direction: RangeDirection,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DetectError: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1), (_, traj2), interval| {
            InterSatelliteRangeDetectFn {
                sc1: traj1,
                sc2: traj2,
                threshold,
                direction,
            }
            .into_intervals(UniformSampler::new(step), interval)
        })
    }

    fn refine_slew<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DetectError: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error>,
    {
        self.refine_trajectories(ensemble, move |(sc1, traj1), (sc2, traj2), interval| {
            let effective_slew_rate = match (sc1.max_slew_rate(), sc2.max_slew_rate()) {
                (Some(a), Some(b)) => {
                    Some(if a.to_radians_per_second() < b.to_radians_per_second() {
                        a
                    } else {
                        b
                    })
                }
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };

            if let Some(threshold) = effective_slew_rate {
                OptionalIterator(Some(
                    InterSatelliteSlewRateDetectFn {
                        sc1: traj1,
                        sc2: traj2,
                        threshold,
                    }
                    .into_intervals(UniformSampler::new(step), interval),
                ))
            } else {
                OptionalIterator(None)
            }
        })
    }

    fn refine_los_central_body<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        central_body: Origin,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DetectError: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1), (_, traj2), interval| {
            InterSatLosCentralBodyDetectFn {
                sc1: traj1,
                sc2: traj2,
                body: central_body,
            }
            .into_intervals(UniformSampler::new(step), interval)
        })
    }
}

impl<'a, T> InterSatelitePipeline<'a> for T where T: Pipeline<'a, Spacecraft, Spacecraft> + Sync {}
