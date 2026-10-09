#[cfg(feature = "comms")]
use std::collections::HashMap;

use itertools::Either;
use lox_bodies::{CoordinateOrigin, Origin, TrySpheroid};
#[cfg(feature = "comms")]
use lox_comms::LinkBudgetError;
#[cfg(feature = "comms")]
use lox_comms::channel::LinkDirection;
#[cfg(feature = "comms")]
use lox_comms::link_budget::{LinkBudget, LinkConditions};
#[cfg(feature = "comms")]
use lox_comms::terminal::{RxTerminal, TxTerminal};
use lox_core::units::{AngularRate, Distance};
use lox_ephem::Ephemeris;
use lox_frames::{
    Frame, ReferenceFrame, providers::DefaultRotationProvider, rotations::TryRotation,
};
use lox_orbits::orbits::{Ensemble, Trajectory};
#[cfg(feature = "comms")]
use lox_time::Time;
use lox_time::{deltas::TimeDelta, intervals::TimeInterval, time_scales::TimeScale};

#[cfg(feature = "comms")]
use crate::link_budget::LinkMarginDetectFn;

#[cfg(feature = "comms")]
use crate::assets::TerminalId;
use crate::events::{AdaptiveSampler, IntervalIterExt, Intervals, Sampler};

#[cfg(feature = "comms")]
use crate::link_budget::{
    CarrierPlan, Link, LinkAnalysisError, LinkEvalError, LinkGeometry, LinkWindow, summarize,
};

#[cfg(feature = "comms")]
use crate::pipeline::FallibleResult;
use crate::pipeline::{AsKey, Refine};
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

pub trait TimeIntervalPipeline<'a>: Pipeline<'a, Window = TimeInterval> {
    fn detect<F, D, S>(
        self,
        get_detector: F,
        sampler: S,
    ) -> Refine<Self, impl Fn(&Self::Asset, Self::Window) -> Result<Intervals<D, S>, DetectError>>
    where
        F: Fn(&Self::Asset) -> D + 'a,
        D: DetectFn,
        S: Sampler<D> + Clone + 'a,
    {
        self.refine(move |asset, window| {
            let detector = get_detector(asset);
            window.detect(detector, sampler.clone())
        })
    }
}

#[cfg(feature = "comms")]
impl<'a, O: CoordinateOrigin, R: ReferenceFrame> AsKey for Link<'a, O, R> {
    type Key = (TerminalId, TerminalId);

    fn as_key(&self) -> Self::Key {
        (self.tx_id.clone(), self.rx_id.clone())
    }
}

#[cfg(feature = "comms")]
pub struct TerminalPair<'a> {
    direction: LinkDirection,
    tx: (TerminalId, &'a TxTerminal),
    rx: (TerminalId, &'a RxTerminal),
}

pub trait MapAsset<'a>: Copy {
    #[cfg(feature = "comms")]
    fn iter_terminals(self) -> impl Iterator<Item = TerminalPair<'a>>;

    #[cfg(feature = "comms")]
    fn link_geometry<O: CoordinateOrigin, R: ReferenceFrame>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
    ) -> LinkGeometry<'a, O, R>;

    #[cfg(feature = "comms")]
    fn links<O: CoordinateOrigin, R: ReferenceFrame>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        plan: &'a CarrierPlan,
        overrides: &'a HashMap<(TerminalId, TerminalId), CarrierPlan>,
    ) -> impl Iterator<Item = Link<'a, O, R>> {
        self.iter_terminals().map(move |terminals| {
            let plan = overrides
                .get(&(terminals.tx.0.clone(), terminals.rx.0.clone()))
                .unwrap_or_else(|| plan);
            let geometry = self.link_geometry(ensemble);

            Link {
                tx_id: terminals.tx.0,
                rx_id: terminals.rx.0,
                direction: terminals.direction,
                tx: terminals.tx.1,
                rx: terminals.rx.1,
                plan,
                geometry,
                visibility: &[],
            }
        })
    }
}

impl<'a> MapAsset<'a> for (&'a GroundStation, &'a Spacecraft) {
    #[cfg(feature = "comms")]
    fn iter_terminals(self) -> impl Iterator<Item = TerminalPair<'a>> {
        let (gs, sc) = self;

        gs.tx_terminals()
            .into_iter()
            .flat_map(move |(tx_name, tx)| {
                sc.rx_terminals()
                    .into_iter()
                    .map(move |(rx_name, rx)| TerminalPair {
                        direction: LinkDirection::Uplink,
                        tx: (TerminalId::new(gs.id().clone(), tx_name), tx),
                        rx: (TerminalId::new(sc.id().clone(), rx_name), rx),
                    })
            })
            .chain(
                sc.tx_terminals()
                    .into_iter()
                    .flat_map(move |(tx_name, tx)| {
                        gs.rx_terminals()
                            .into_iter()
                            .map(move |(rx_name, rx)| TerminalPair {
                                direction: LinkDirection::Downlink,
                                tx: (TerminalId::new(sc.id().clone(), tx_name), tx),
                                rx: (TerminalId::new(gs.id().clone(), rx_name), rx),
                            })
                    }),
            )
    }

    #[cfg(feature = "comms")]
    fn link_geometry<O: CoordinateOrigin, R: ReferenceFrame>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
    ) -> LinkGeometry<'a, O, R> {
        let (gs, sc) = self;
        let sc_traj = ensemble.get(sc.id()).unwrap();

        LinkGeometry::GroundSpace {
            gs: gs.location(),
            sc: sc_traj,
        }
    }
}

impl<'a> MapAsset<'a> for (&'a Spacecraft, &'a Spacecraft) {
    #[cfg(feature = "comms")]
    fn iter_terminals(self) -> impl Iterator<Item = TerminalPair<'a>> {
        let (sc1, sc2) = self;

        sc1.tx_terminals()
            .into_iter()
            .flat_map(move |(tx_name, tx)| {
                sc2.rx_terminals()
                    .into_iter()
                    .map(move |(rx_name, rx)| TerminalPair {
                        direction: LinkDirection::Crosslink,
                        tx: (TerminalId::new(sc1.id().clone(), tx_name), tx),
                        rx: (TerminalId::new(sc2.id().clone(), rx_name), rx),
                    })
            })
    }

    #[cfg(feature = "comms")]
    fn link_geometry<O: CoordinateOrigin, R: ReferenceFrame>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
    ) -> LinkGeometry<'a, O, R> {
        let (sc1, sc2) = self;
        let traj1 = ensemble.get(sc1.id()).unwrap();
        let traj2 = ensemble.get(sc2.id()).unwrap();

        LinkGeometry::InterSatellite {
            sc1: traj1,
            sc2: traj2,
        }
    }
}

pub trait Window: Sized + Copy {
    fn detect<D, S>(self, detector: D, sampler: S) -> Result<Intervals<D, S>, DetectError>
    where
        D: DetectFn,
        S: Sampler<D> + Clone;

    fn elevation_detect<'a, S, O, R>(
        self,
        sampler: S,
        gs: &'a GroundStation,
        sc_traj: &'a Trajectory<O, R>,
    ) -> Result<Intervals<ElevationDetectFn<'a, O, R>, S>, DetectError>
    where
        O: CoordinateOrigin + TrySpheroid + Copy,
        R: ReferenceFrame + Copy,
        S: Sampler<ElevationDetectFn<'a, O, R>>,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    {
        let elev = ElevationDetectFn {
            gs: gs,
            sc: sc_traj,
        };

        self.detect(elev, sampler)
    }

    fn los_detect<'a, O, R, E>(
        self,
        gs: &'a GroundStation,
        sc_traj: &'a Trajectory<O, R>,
        occulting_bodies: &'a [Origin],
        ephemeris: &'a E,
        step: TimeDelta,
    ) -> Result<impl Iterator<Item = Result<TimeInterval, DetectError>> + 'a, DetectError>
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
        E: Ephemeris + Sync,
        E::Error: 'static,
        DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
        <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
            std::error::Error + Send + Sync + 'static,
    {
        let make_los = |body: Origin| {
            self.detect(
                LineOfSightDetectFn {
                    gs: gs.location(),
                    sc: sc_traj,
                    body,
                    ephemeris,
                },
                UniformSampler::new(step),
            )
        };
        let mut los: Box<
            dyn Iterator<Item = Result<TimeInterval, DetectError>> + Sync + Send + '_,
        > = Box::new(make_los(occulting_bodies[0])?);
        for &body in &occulting_bodies[1..] {
            los = Box::new(los.intersect(make_los(body)?));
        }
        Ok(los)
    }

    #[cfg(feature = "comms")]
    fn link_margin_detect<O, R>(
        self,
        step: TimeDelta,
        link: &Link<O, R>,
    ) -> Result<impl Iterator<Item = Result<LinkWindow, LinkAnalysisError>>, LinkAnalysisError>
    where
        O: CoordinateOrigin + TrySpheroid + Copy,
        R: ReferenceFrame + Copy,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    {
        let detector = LinkMarginDetectFn(link);

        let intervals = self.detect(detector, UniformSampler::new(step))?;

        Ok(intervals.map(move |interval| Ok(summarize(link, interval?, step)?)))
    }
}

impl Window for TimeInterval {
    fn detect<D, S>(self, detector: D, sampler: S) -> Result<Intervals<D, S>, DetectError>
    where
        D: DetectFn,
        S: Sampler<D>,
    {
        Ok(detector.into_intervals(sampler, self))
    }
}

#[cfg(feature = "comms")]
pub trait LinkPipeline<'a, T>: Pipeline<'a, Asset = T, Window = TimeInterval>
where
    T: MapAsset<'a>,
{
    fn map_to_links<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        plan: &'a CarrierPlan,
        overrides: &'a HashMap<(TerminalId, TerminalId), CarrierPlan>,
    ) -> impl Pipeline<
        'a,
        Asset = Link<'a, O, R>,
        Window = Self::Window,
        Error = Self::Error,
        ConvertResult = Self::ConvertResult,
    >
    where
        O: CoordinateOrigin + TrySpheroid + Copy + Sync,
        R: ReferenceFrame + Copy + Sync,
    {
        self.map_asset(move |asset| asset.links(ensemble, plan, overrides))
    }
}

#[cfg(feature = "comms")]
pub trait LinkBudgetPipeline<'a, O, R>:
    Pipeline<'a, Asset = Link<'a, O, R>, Window = TimeInterval>
where
    O: CoordinateOrigin + TrySpheroid + Copy + Sync + 'a,
    R: ReferenceFrame + Copy + Sync + 'a,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    Self::ConvertResult: ConvertResult<Self::Error, LinkAnalysisError>,
{
    fn link_budget(
        self,
        step: TimeDelta,
    ) -> impl Pipeline<
        'a,
        Asset = Self::Asset,
        Window = LinkWindow,
        Error = LinkAnalysisError,
        ConvertResult = FallibleResult,
    > {
        self.refine(move |link, interval| {
            let detector = LinkMarginDetectFn(link);
            let mut windows = Vec::new();

            for window in detector.iter_intervals(UniformSampler::new(step), interval) {
                windows.push(Ok(summarize(link, window?, step)?));
            }

            Ok(windows.into_iter())
        })
    }
}

pub trait GroundSpacePipeline<'a>:
    Pipeline<'a, Asset = (&'a GroundStation, &'a Spacecraft), Window = TimeInterval>
{
    fn refine_visibility<O, R, F, S>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        sampler: F,
    ) -> impl Pipeline<'a, Asset = Self::Asset, Window = Self::Window>
    where
        O: CoordinateOrigin + TrySpheroid + Copy,
        R: ReferenceFrame + Copy,
        S: Sampler<ElevationDetectFn<'a, O, R>> + 'a,
        F: Fn(TimeInterval) -> S + Sync + 'a,
        DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
        <Self as Pipeline<'a>>::ConvertResult:
            ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
    {
        self.refine(move |(gs, sc), interval| {
            let sc_traj = ensemble.get(sc.id()).unwrap();

            interval.elevation_detect(sampler(interval), gs, sc_traj)
        })
    }

    fn refine_visibility_uniform<O, R>(
        self,
        ensemble: &'a Ensemble<AssetId, O, R>,
        step: TimeDelta,
    ) -> impl Pipeline<'a, Asset = Self::Asset, Window = Self::Window>
    where
        O: CoordinateOrigin + TrySpheroid + Copy,
        R: ReferenceFrame + Copy,
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
    ) -> impl Pipeline<'a, Asset = Self::Asset, Window = Self::Window>
    where
        O: CoordinateOrigin + TrySpheroid + Copy,
        R: ReferenceFrame + Copy,
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
    ) -> impl Pipeline<'a, Asset = Self::Asset, Window = Self::Window>
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
        self.refine(move |(gs, sc), interval| {
            let sc_traj = ensemble.get(sc.id()).unwrap();
            interval.los_detect(gs, sc_traj, occulting_bodies, ephemeris, step)
        })
    }
}

// impl<'a, T> GroundSpacePipeline<'a> for T where
//     T: Pipeline<
//             'a,
//             Asset = (&'a GroundStation, &'a Spacecraft),
//             Window = TimeInterval,
//             Key = (AssetId, AssetId),
//         > + Sync
// {
// }

// pub trait InterSatelitePipeline<'a>:
//     Pipeline<
//     'a,
//     Asset = (&'a Spacecraft, &'a Spacecraft),
//     Window = TimeInterval,
//     Key = (AssetId, AssetId),
// >
// where
//     Self: Sync,
// {
//     fn refine_trajectories<O, R, F, I, E>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         f: F,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         F: Fn(
//                 (
//                     &'a Spacecraft,
//                     &'a Trajectory<O, R>,
//                     &'a Spacecraft,
//                     &'a Trajectory<O, R>,
//                 ),
//                 TimeInterval,
//             ) -> Result<I, E>
//             + Sync,
//         I: Iterator<Item = Result<TimeInterval, E>> + Send + Sync + 'a,
//         E: Send + Sync + core::fmt::Debug,
//         <Self as Pipeline<'a>>::ConvertResult: ConvertResult<<Self as Pipeline<'a>>::Error, E>,
//     {
//         self.refine(move |(sc1, sc2), interval| {
//             let traj1 = ensemble
//                 .get(sc1.id())
//                 .expect("trajectory not found in ensemble");
//             let traj2 = ensemble
//                 .get(sc2.id())
//                 .expect("trajectory not found in ensemble");

//             f((sc1, traj1, sc2, traj2), interval)
//         })
//     }

//     fn refine_visibility_intersect<O, R>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         step: TimeDelta,
//         min: Distance,
//         max: Distance,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         <Self as Pipeline<'a>>::ConvertResult:
//             ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
//     {
//         self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
//             let min = InterSatelliteRangeDetectFn {
//                 sc1: traj1,
//                 sc2: traj2,
//                 threshold: min,
//                 direction: RangeDirection::Min,
//             }
//             .into_intervals(UniformSampler::new(step), interval);
//             let max = InterSatelliteRangeDetectFn {
//                 sc1: traj1,
//                 sc2: traj2,
//                 threshold: max,
//                 direction: RangeDirection::Max,
//             }
//             .into_intervals(UniformSampler::new(step), interval);

//             Ok(max.intersect(min))
//         })
//     }

//     fn refine_visibility<O, R>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         step: TimeDelta,
//         threshold: Distance,
//         direction: RangeDirection,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         <Self as Pipeline<'a>>::ConvertResult:
//             ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
//     {
//         self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
//             Ok(InterSatelliteRangeDetectFn {
//                 sc1: traj1,
//                 sc2: traj2,
//                 threshold,
//                 direction,
//             }
//             .into_intervals(UniformSampler::new(step), interval))
//         })
//     }

//     fn refine_slew<O, R>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         step: TimeDelta,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         <Self as Pipeline<'a>>::ConvertResult:
//             ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
//     {
//         self.refine_trajectories(ensemble, move |(sc1, traj1, sc2, traj2), interval| {
//             let effective_slew_rate = match (sc1.max_slew_rate(), sc2.max_slew_rate()) {
//                 (Some(a), Some(b)) => {
//                     Some(if a.to_radians_per_second() < b.to_radians_per_second() {
//                         a
//                     } else {
//                         b
//                     })
//                 }
//                 (Some(a), None) => Some(a),
//                 (None, Some(b)) => Some(b),
//                 (None, None) => None,
//             };

//             Ok(if let Some(threshold) = effective_slew_rate {
//                 Either::Left(
//                     InterSatelliteSlewRateDetectFn {
//                         sc1: traj1,
//                         sc2: traj2,
//                         threshold,
//                     }
//                     .into_intervals(UniformSampler::new(step), interval),
//                 )
//             } else {
//                 Either::Right([Ok(interval)].into_iter())
//             })
//         })
//     }

//     fn refine_los_central_body<O, R>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         step: TimeDelta,
//         central_body: Origin,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         <Self as Pipeline<'a>>::ConvertResult:
//             ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
//     {
//         self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
//             Ok(InterSatLosCentralBodyDetectFn {
//                 sc1: traj1,
//                 sc2: traj2,
//                 body: central_body,
//             }
//             .into_intervals(UniformSampler::new(step), interval))
//         })
//     }

//     fn refine_line_of_sight<O, R, E>(
//         self,
//         ensemble: &'a Ensemble<AssetId, O, R>,
//         occulting_bodies: &'a [Origin],
//         ephemeris: &'a E,
//         step: TimeDelta,
//     ) -> impl InterSatelitePipeline<'a>
//     where
//         O: CoordinateOrigin + TrySpheroid + Copy + Sync,
//         R: ReferenceFrame + Copy + Sync,
//         E: Ephemeris + Sync,
//         E::Error: 'static,
//         DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
//         <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
//             std::error::Error + Send + Sync + 'static,
//         <Self as Pipeline<'a>>::ConvertResult:
//             ConvertResult<<Self as Pipeline<'a>>::Error, DetectError>,
//     {
//         self.refine_trajectories(ensemble, move |(_, traj1, _, traj2), interval| {
//             let make_los = |body: Origin| {
//                 InterSatLosOccluderDetectFn {
//                     sc1: traj1,
//                     sc2: traj2,
//                     body,
//                     ephemeris,
//                 }
//                 .into_intervals(UniformSampler::new(step), interval)
//             };
//             let mut los: Box<
//                 dyn Iterator<Item = Result<TimeInterval, DetectError>> + Sync + Send + '_,
//             > = Box::new(make_los(occulting_bodies[0]));
//             for &body in &occulting_bodies[1..] {
//                 los = Box::new(los.intersect(make_los(body)));
//             }
//             Ok(los)
//         })
//     }
// }

// impl<'a, T> InterSatelitePipeline<'a> for T where
//     T: Pipeline<
//             'a,
//             Asset = (&'a Spacecraft, &'a Spacecraft),
//             Window = TimeInterval,
//             Key = (AssetId, AssetId),
//         > + Sync
// {
// }
