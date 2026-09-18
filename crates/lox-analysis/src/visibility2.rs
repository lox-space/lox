use lox_bodies::{CoordinateOrigin, Origin, TrySpheroid};
use lox_core::units::{AngularRate, Distance};
use lox_ephem::Ephemeris;
use lox_frames::{
    Frame, ReferenceFrame, providers::DefaultRotationProvider, rotations::TryRotation,
};
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::{deltas::TimeDelta, intervals::TimeInterval, time_scales::TimeScale};

use crate::events::{AdaptiveSampler, IntervalIterExt, Intervals, Sampler};
use crate::visibility::{InterSatLosOccluderDetectFn, LineOfSightDetectFn};
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
    fn refine_trajectory<O, R, F, I, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        f: F,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        F: Fn(&'a GroundStation, (&'a Spacecraft, &'a Trajectory<O, R>), TimeInterval) -> I
            + Sync
            + 'a,
        I: Iterator<Item = Result<TimeInterval, E>> + Send + Sync + 'a,
        E: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error> + Sync + Send,
    {
        self.refine(move |gs, sc, interval| {
            let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );

            f(gs, (sc, sc_traj), interval)
        })
    }

    fn refine_visibility<O, R, F, S>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        sampler: F,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        S: Sampler<ElevationDetectFn<'a, O, R>> + Sync + Send + 'a,
        F: Fn(TimeInterval) -> S + Sync + 'a,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        DetectError: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error>,
    {
        self.refine_trajectory(ensemble, move |gs, (_, sc_traj), interval| {
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

    fn refine_line_of_sight<O, R, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        occulting_bodies: &'a [Origin],
        ephemeris: &'a E,
        step: TimeDelta,
    ) -> impl Pipeline<'a, GroundStation, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        E: Ephemeris + Sync,
        E::Error: 'static,
        DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
        <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
            std::error::Error + Send + Sync + 'static,
        DetectError: From<<Self as Pipeline<'a, GroundStation, Spacecraft>>::Error>,
    {
        self.refine_trajectory(ensemble, move |gs, (_, sc_traj), interval| {
            let make_los = |body: Origin| {
                LineOfSightDetectFn {
                    gs: gs.location(),
                    sc: sc_traj,
                    body,
                    ephemeris,
                }
                .into_intervals(UniformSampler::new(step), interval)
            };
            let mut los: Box<
                dyn Iterator<Item = Result<TimeInterval, DetectError>> + Sync + Send + '_,
            > = Box::new(make_los(occulting_bodies[0]));
            for &body in &occulting_bodies[1..] {
                los = Box::new(los.intersect(make_los(body)));
            }
            los
        })
    }
}

impl<'a, T> GroundSpacePipeline<'a> for T where T: Pipeline<'a, GroundStation, Spacecraft> + Sync {}

enum EitherIterator<I1, I2> {
    Iter1(I1),
    Iter2(I2),
}

impl<I1, I2, Item> Iterator for EitherIterator<I1, I2>
where
    I1: Iterator<Item = Item>,
    I2: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIterator::Iter1(iter) => iter.next(),
            EitherIterator::Iter2(iter) => iter.next(),
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
                EitherIterator::Iter1(
                    InterSatelliteSlewRateDetectFn {
                        sc1: traj1,
                        sc2: traj2,
                        threshold,
                    }
                    .into_intervals(UniformSampler::new(step), interval),
                )
            } else {
                EitherIterator::Iter2([Ok(interval)].into_iter())
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

    fn refine_line_of_sight<O, R, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        occulting_bodies: &'a [Origin],
        ephemeris: &'a E,
        step: TimeDelta,
    ) -> impl Pipeline<'a, Spacecraft, Spacecraft>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        E: Ephemeris + Sync,
        E::Error: 'static,
        DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
        <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
            std::error::Error + Send + Sync + 'static,
        DetectError: From<<Self as Pipeline<'a, Spacecraft, Spacecraft>>::Error>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1), (_, traj2), interval| {
            let make_los = |body: Origin| {
                InterSatLosOccluderDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    body,
                    ephemeris,
                }
                .into_intervals(UniformSampler::new(step), interval)
            };
            let mut los: Box<
                dyn Iterator<Item = Result<TimeInterval, DetectError>> + Sync + Send + '_,
            > = Box::new(make_los(occulting_bodies[0]));
            for &body in &occulting_bodies[1..] {
                los = Box::new(los.intersect(make_los(body)));
            }
            los
        })
    }
}

impl<'a, T> InterSatelitePipeline<'a> for T where T: Pipeline<'a, Spacecraft, Spacecraft> + Sync {}
