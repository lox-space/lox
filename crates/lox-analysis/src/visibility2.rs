use itertools::Either;
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
    pipeline::{ConvertResult, Pipeline},
    power::SpacecraftFilter,
    visibility::{
        ElevationDetectFn, InterSatLosCentralBodyDetectFn, InterSatelliteRangeDetectFn,
        InterSatelliteSlewRateDetectFn, RangeDirection,
    },
};

pub trait GroundSpacePipeline<'a>:
    Pipeline<
        'a,
        Asset = (&'a GroundStation, &'a Spacecraft),
        Window = TimeInterval,
        Key = (AssetId, AssetId),
    >
where
    Self: Sync,
{
    fn refine_trajectory<O, R, F, I, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        f: F,
    ) -> impl GroundSpacePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        F: Fn(
                (&'a GroundStation, &'a Spacecraft, &'a Trajectory<O, R>),
                TimeInterval,
            ) -> Result<I, E>
            + Sync
            + 'a,
        I: Iterator<Item = Result<TimeInterval, E>> + Send + Sync + 'a,
        E: Sync + Send + core::fmt::Debug,
        <Self as Pipeline<'a>>::ConvertResult: ConvertResult<<Self as Pipeline<'a>>::Error, E>,
    {
        self.refine(move |(gs, sc), interval| {
            let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );

            f((gs, sc, sc_traj), interval)
        })
    }

    fn refine_visibility<O, R, F, S>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        sampler: F,
    ) -> impl GroundSpacePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        S: Sampler<ElevationDetectFn<'a, O, R>> + Sync + Send + 'a,
        F: Fn(TimeInterval) -> S + Sync + 'a,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectory(ensemble, move |(gs, _, sc_traj), interval| {
            let elev = ElevationDetectFn {
                gs: gs.location(),
                mask: gs.mask(),
                sc: sc_traj,
            };
            Ok(elev.into_intervals(sampler(interval), interval))
        })
    }

    fn refine_visibility_uniform<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl GroundSpacePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_visibility(ensemble, move |_| UniformSampler::new(step))
    }

    fn refine_visibility_adaptive<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl GroundSpacePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
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
    ) -> impl GroundSpacePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        E: Ephemeris + Sync,
        E::Error: 'static,
        DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
        <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
            std::error::Error + Send + Sync + 'static,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectory(ensemble, move |(gs, _, sc_traj), interval| {
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
            Ok(los)
        })
    }
}

impl<'a, T> GroundSpacePipeline<'a> for T where
    T: Pipeline<
            'a,
            Asset = (&'a GroundStation, &'a Spacecraft),
            Window = TimeInterval,
            Key = (AssetId, AssetId),
        > + Sync
{
}

pub trait InterSatelitePipeline<'a>:
    Pipeline<
        'a,
        Asset = (&'a Spacecraft, &'a Spacecraft),
        Window = TimeInterval,
        Key = (AssetId, AssetId),
    >
where
    Self: Sync,
{
    fn refine_trajectories<O, R, F, I, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        f: F,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        F: Fn(
                (
                    &'a Spacecraft,
                    &'a Trajectory<O, R>,
                    &'a Spacecraft,
                    &'a Trajectory<O, R>,
                ),
                TimeInterval,
            ) -> Result<I, E>
            + Sync,
        I: Iterator<Item = Result<TimeInterval, E>> + Send + Sync + 'a,
        E: Send + Sync + core::fmt::Debug,
        <Self as Pipeline<'a>>::ConvertResult: ConvertResult<<Self as Pipeline<'a>>::Error, E>,
    {
        self.refine(move |(sc1, sc2), interval| {
            let traj1 = ensemble
                .get(sc1.id())
                .expect("trajectory not found in ensemble");
            let traj2 = ensemble
                .get(sc2.id())
                .expect("trajectory not found in ensemble");

            f((sc1, traj1, sc2, traj2), interval)
        })
    }

    fn refine_visibility_intersect<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        min: Distance,
        max: Distance,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
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

            Ok(max.intersect(min))
        })
    }

    fn refine_visibility<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        threshold: Distance,
        direction: RangeDirection,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
            Ok(InterSatelliteRangeDetectFn {
                sc1: traj1,
                sc2: traj2,
                threshold,
                direction,
            }
            .into_intervals(UniformSampler::new(step), interval))
        })
    }

    fn refine_slew<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectories(ensemble, move |(sc1, traj1, sc2, traj2), interval| {
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

            Ok(if let Some(threshold) = effective_slew_rate {
                Either::Left(
                    InterSatelliteSlewRateDetectFn {
                        sc1: traj1,
                        sc2: traj2,
                        threshold,
                    }
                    .into_intervals(UniformSampler::new(step), interval),
                )
            } else {
                Either::Right([Ok(interval)].into_iter())
            })
        })
    }

    fn refine_los_central_body<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
        central_body: Origin,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
            Ok(InterSatLosCentralBodyDetectFn {
                sc1: traj1,
                sc2: traj2,
                body: central_body,
            }
            .into_intervals(UniformSampler::new(step), interval))
        })
    }

    fn refine_line_of_sight<O, R, E>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        occulting_bodies: &'a [Origin],
        ephemeris: &'a E,
        step: TimeDelta,
    ) -> impl InterSatelitePipeline<'a>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        E: Ephemeris + Sync,
        E::Error: 'static,
        DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
        <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
            std::error::Error + Send + Sync + 'static,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
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
            Ok(los)
        })
    }
}

impl<'a, T> InterSatelitePipeline<'a> for T where
    T: Pipeline<
            'a,
            Asset = (&'a Spacecraft, &'a Spacecraft),
            Window = TimeInterval,
            Key = (AssetId, AssetId),
        > + Sync
{
}
