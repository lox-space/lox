// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Real [`Source`] implementations for the four analyses.
//!
//! These coexist with the eager `*Analysis::compute()` path (migration §2) and
//! are exercised only by the differential parity tests below until the hard cut
//! promotes them. Nothing here is public.
//!
//! Every source in this module **borrows** its scenario inputs, so its
//! [`Source::Stream`] carries the source's own `'a` — permitted precisely
//! because `Source::Stream` has no lifetime tied to the `&self` of
//! [`Source::detect`] (design §4.1). The owned `'static` fork the streaming
//! engine needs is a separate concern and does not exist yet.
//!
//! **Cost ordering is the organising principle.** Each source stacks its
//! detectors cheapest-first with [`IntervalIterExt::then_within`], so an
//! expensive detector is only ever sampled inside the windows a cheaper one
//! already admitted. That is why range is a gate *inside* the source rather
//! than a [`Stage`] after materialisation (design §6.1): as a stage it would
//! run after the observables it is meant to avoid computing.

// Every item here is reachable only from the parity tests until the hard cut
// (plan step 7) deletes the eager `compute()` path and promotes these to the
// public names. The blanket allow is scoped to this transitional module and
// goes away with it — do not copy it elsewhere.
#![allow(dead_code)]

use std::convert::Infallible;
use std::ops::Deref;
use std::sync::Arc;

use lox_bodies::{CoordinateOrigin, Origin, Sun, TryMeanRadius, TrySpheroid};
use lox_core::math::series::InterpolationType;
use lox_core::sync::CancellationToken;
use lox_core::units::{AngularRate, Distance};
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::ground::GroundLocation;
use lox_orbits::orbits::Trajectory;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::series::TimeSeries;
use lox_time::time_scales::{Tdb, TimeScale};

use crate::assets::{GroundStation, Spacecraft};
use crate::events::{
    AdaptiveSampler, DetectError, DetectFnExt as _, IntervalIterExt as _, Intervals, UniformSampler,
};
use crate::pipeline::{AnalysisError, HasInterval, PipelineExt as _, Source, Stage};
use crate::power::{beta_angle, solar_flux};
use crate::visibility::{
    ElevationDetectFn, ElevationMask, EvalError, GroundSpaceRangeDetectFn,
    InterSatLosCentralBodyDetectFn, InterSatLosOccluderDetectFn, InterSatelliteRangeDetectFn,
    InterSatelliteSlewRateDetectFn, LineOfSightDetectFn, Pass, RangeDirection,
};

#[cfg(feature = "imaging")]
use crate::imaging::{
    analysis::{AccessDetectFn, AccessPayload, pass_direction_of, sub_sat_sample},
    aoi::Aoi,
    results::AccessWindow,
};

// ---------------------------------------------------------------------------
// Stream plumbing
// ---------------------------------------------------------------------------

/// A lazy window stream in the [`events`](crate::events) error domain, boxed so
/// that a staged stack of detectors has one nameable type.
///
/// Erasure is unavoidable here: `then_within` and `intersect` each wrap their
/// operand in a new type, so a stack whose shape depends on runtime
/// configuration (how many occulting bodies, which range limits are set) has no
/// single static type. The eager path boxes at exactly the same points.
pub(crate) type BoxedWindows<'a> =
    Box<dyn Iterator<Item = Result<TimeInterval, DetectError>> + Send + 'a>;

/// A lazy stream of analysis items.
///
/// The `'a` is the lifetime of the *inputs* the stream reads — a scenario and its
/// trajectories — not of the analysis handle that produced it, so the stream
/// outlives the call that built it and can be stored, filtered, or partly
/// consumed.
pub struct ItemStream<'a, T>(Box<dyn Iterator<Item = Result<T, AnalysisError>> + Send + 'a>);

impl<'a, T> ItemStream<'a, T> {
    /// Boxes any compatible iterator as an item stream.
    pub fn new(items: impl Iterator<Item = Result<T, AnalysisError>> + Send + 'a) -> Self {
        Self(Box::new(items))
    }
}

impl<T> Iterator for ItemStream<'_, T> {
    type Item = Result<T, AnalysisError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Lifts a window stream out of the detect-error domain into the pipeline's.
pub(crate) fn lift<'a>(windows: BoxedWindows<'a>) -> ItemStream<'a, TimeInterval> {
    ItemStream(Box::new(windows.map(|r| r.map_err(AnalysisError::from))))
}

// ---------------------------------------------------------------------------
// Item types
// ---------------------------------------------------------------------------

/// An inter-satellite contact window.
///
/// [`HasInterval`]-only: sat-to-sat contacts have no ground observables, so
/// they cannot be [`Pass`]es. Relative geometry becomes a capability if a
/// second consumer appears (design §6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window(
    /// The contact interval.
    pub TimeInterval,
);

/// A single eclipse interval (cylindrical umbra, no penumbra).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eclipse(
    /// The eclipse interval.
    pub TimeInterval,
);

impl HasInterval for Pass {
    fn interval(&self) -> TimeInterval {
        *Pass::interval(self)
    }
}

impl HasInterval for Window {
    fn interval(&self) -> TimeInterval {
        self.0
    }
}

impl HasInterval for Eclipse {
    fn interval(&self) -> TimeInterval {
        self.0
    }
}

#[cfg(feature = "imaging")]
impl HasInterval for AccessWindow {
    fn interval(&self) -> TimeInterval {
        self.interval
    }
}

// ---------------------------------------------------------------------------
// Ground-space windows
// ---------------------------------------------------------------------------

/// The station data a detector needs, lifted out of the [`GroundStation`] once
/// so a stack never retains the station itself. Cheap to clone — the only
/// non-trivial field is a `Variable` mask's series, cloned once per scan rather
/// than once per sample.
#[derive(Clone)]
pub(crate) struct StationView {
    location: GroundLocation,
    mask: ElevationMask,
    body_fixed_frame: Frame,
}

impl StationView {
    pub(crate) fn of(station: &GroundStation) -> Self {
        Self {
            location: station.location().clone(),
            mask: station.mask().clone(),
            body_fixed_frame: station.body_fixed_frame(),
        }
    }
}

/// The knobs a ground-space stack needs, independent of how its inputs are held.
///
/// `step` is the already-resolved scan step: `min_pass_duration` is **not** here,
/// because it is a `filter_ok` stage on materialised items (design §8). Only a
/// caller that sees both knobs can decide to coarsen the scan on its account.
#[derive(Clone, Default)]
pub(crate) struct GroundSpaceConfig {
    pub(crate) step: TimeDelta,
    pub(crate) min_range: Option<Distance>,
    pub(crate) max_range: Option<Distance>,
    pub(crate) adaptive: bool,
    /// Checked inside every scan, so a cancelled run stops within one detector
    /// evaluation rather than waiting for the next item (design §7.1).
    pub(crate) cancel: Option<CancellationToken>,
}

impl GroundSpaceConfig {
    pub(crate) fn new(step: TimeDelta) -> Self {
        Self {
            step,
            ..Default::default()
        }
    }
}

/// Attaches `cancel` to a scan when one is set.
///
/// A free function rather than a method chain because
/// [`Intervals::with_cancellation`] consumes and returns `Self`, so there is no
/// way to apply it conditionally in place.
fn cancellable<F, S, R>(
    intervals: Intervals<F, S, R>,
    cancel: &Option<CancellationToken>,
) -> Intervals<F, S, R> {
    match cancel {
        Some(token) => intervals.with_cancellation(token.clone()),
        None => intervals,
    }
}

/// Builds the staged ground-space window stack: elevation, then line-of-sight
/// per occulting body, then the range gate.
///
/// `'x` is the lifetime the returned stream is valid for, and it comes from the
/// *handles* rather than from a `&self` — which is exactly what lets one copy of
/// this staging logic serve both the borrowing sources (`'x = 'a`) and the
/// streaming fork (`'x = 'static`, handles being `Arc`s). Writing it as a method
/// would tie the stream to the borrow of the source and force the two paths to
/// duplicate it.
pub(crate) fn ground_space_stack<'x, O, R, T, E>(
    station: StationView,
    trajectory: T,
    ephemeris: E,
    occulters: Vec<Origin>,
    config: GroundSpaceConfig,
    interval: TimeInterval,
) -> BoxedWindows<'x>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + 'x,
    R: ReferenceFrame + Copy + Send + Sync + 'x,
    T: Deref<Target = Trajectory<O, R>> + Clone + Send + Sync + 'x,
    E: Deref + Clone + Send + Sync + 'x,
    E::Target: Ephemeris,
    <E::Target as Ephemeris>::Error: 'static,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    let step = config.step;
    let cancel = config.cancel.clone();

    // Elevation is the cheap constraint and always runs over the whole
    // interval; `adaptive` lets it stride by its own rate bound.
    let elev = ElevationDetectFn {
        gs: station.location.clone(),
        mask: station.mask.clone(),
        sc: trajectory.clone(),
        body_fixed_frame: station.body_fixed_frame,
    };
    let mut windows: BoxedWindows<'x> = if config.adaptive {
        Box::new(cancellable(
            elev.into_intervals(
                AdaptiveSampler::new(step, interval.duration().max(step)),
                interval,
            ),
            &cancel,
        ))
    } else {
        Box::new(cancellable(
            elev.into_intervals(UniformSampler::new(step), interval),
            &cancel,
        ))
    };

    // Line of sight needs an ephemeris lookup per sample, so it runs only
    // inside the windows elevation already admitted.
    if !occulters.is_empty() {
        let view = station.clone();
        let traj = trajectory.clone();
        let cancel = cancel.clone();
        windows = Box::new(windows.then_within(move |window| {
            let make_los = |body: Origin| {
                cancellable(
                    LineOfSightDetectFn {
                        gs: view.location.clone(),
                        sc: traj.clone(),
                        body,
                        ephemeris: ephemeris.clone(),
                        body_fixed_frame: view.body_fixed_frame,
                    }
                    .into_intervals(UniformSampler::new(step), window),
                    &cancel,
                )
            };
            let mut los: BoxedWindows<'x> = Box::new(make_los(occulters[0]));
            for &body in &occulters[1..] {
                los = Box::new(los.intersect(make_los(body)));
            }
            los
        }));
    }

    // The range gate (design §6.1). Nesting max inside min is equivalent to
    // intersecting them and strictly cheaper, since the second detector never
    // sees the stretches the first ruled out.
    let gate = |windows: BoxedWindows<'x>, threshold: Distance, direction: RangeDirection| {
        let location = station.location.clone();
        let body_fixed_frame = station.body_fixed_frame;
        let traj = trajectory.clone();
        let cancel = cancel.clone();
        Box::new(windows.then_within(move |window| {
            cancellable(
                GroundSpaceRangeDetectFn {
                    gs: location.clone(),
                    sc: traj.clone(),
                    body_fixed_frame,
                    threshold,
                    direction,
                }
                .into_intervals(UniformSampler::new(step), window),
                &cancel,
            )
        })) as BoxedWindows<'x>
    };
    if let Some(max) = config.max_range {
        windows = gate(windows, max, RangeDirection::Max);
    }
    if let Some(min) = config.min_range {
        windows = gate(windows, min, RangeDirection::Min);
    }
    windows
}

/// The bare-window half of the ground-space source, over borrowed inputs.
pub(crate) struct GroundSpaceWindows<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    station: &'a GroundStation,
    trajectory: &'a Trajectory<O, R>,
    ephemeris: &'a E,
    occulting_bodies: &'a [Origin],
    config: GroundSpaceConfig,
}

impl<'a, O, R, E> GroundSpaceWindows<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Creates a ground-space window source with no occulters and no range gate.
    pub(crate) fn new(
        station: &'a GroundStation,
        trajectory: &'a Trajectory<O, R>,
        ephemeris: &'a E,
        step: TimeDelta,
    ) -> Self {
        Self {
            station,
            trajectory,
            ephemeris,
            occulting_bodies: &[],
            config: GroundSpaceConfig::new(step),
        }
    }

    /// Adds occulting bodies, each checked by an ephemeris-backed
    /// line-of-sight detector inside the elevation windows.
    pub(crate) fn with_occulting_bodies(mut self, bodies: &'a [Origin]) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Sets the slant-range gate (design §6.1).
    pub(crate) fn with_range_limits(
        mut self,
        min_range: Option<Distance>,
        max_range: Option<Distance>,
    ) -> Self {
        self.config.min_range = min_range;
        self.config.max_range = max_range;
        self
    }

    /// Drives the elevation scan by its own rate bound rather than a fixed step.
    pub(crate) fn adaptive(mut self) -> Self {
        self.config.adaptive = true;
        self
    }

    /// Makes every scan in the stack cancellable.
    pub(crate) fn with_cancellation(mut self, cancel: Option<CancellationToken>) -> Self {
        self.config.cancel = cancel;
        self
    }

    fn stack(&self, interval: TimeInterval) -> BoxedWindows<'a> {
        ground_space_stack(
            StationView::of(self.station),
            self.trajectory,
            self.ephemeris,
            self.occulting_bodies.to_vec(),
            self.config.clone(),
            interval,
        )
    }
}

impl<'a, O, R, E> Source for GroundSpaceWindows<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Out = TimeInterval;
    type Stream = ItemStream<'a, TimeInterval>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        lift(self.stack(interval))
    }
}

// ---------------------------------------------------------------------------
// Pass materialisation
// ---------------------------------------------------------------------------

/// Turns a visibility window into a [`Pass`] by sampling observables across it.
///
/// A window whose interior never clears the mask yields no `Pass` at all —
/// `Stage` flat-maps to `0..n`, so dropping is free and needs no error.
pub(crate) struct MaterialisePass {
    pub(crate) station: StationView,
    /// Shared because the stage outlives the `&self` of `detect`, and `Pass`
    /// needs the origin/frame-erased trajectory that `Pass::from_interval`
    /// takes. Cloned once per source rather than once per window.
    pub(crate) trajectory: Arc<Trajectory>,
    pub(crate) resolution: TimeDelta,
}

impl Stage<TimeInterval> for MaterialisePass {
    type Out = Pass;
    type Error = Infallible;

    fn apply(&self, window: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
        Ok(Pass::from_interval(
            window,
            self.resolution,
            &self.station.location,
            &self.station.mask,
            &self.trajectory,
            self.station.body_fixed_frame,
        )
        .into_iter()
        .collect())
    }
}

/// The ground-space source proper: staged windows, then observables.
pub(crate) struct PassSource<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    windows: GroundSpaceWindows<'a, O, R, E>,
    trajectory: Arc<Trajectory>,
    resolution: TimeDelta,
}

impl<'a, O, R, E> PassSource<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<Frame>,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Wraps a window source, sampling observables at `resolution`.
    pub(crate) fn new(windows: GroundSpaceWindows<'a, O, R, E>, resolution: TimeDelta) -> Self {
        let trajectory = Arc::new(windows.trajectory.clone().into_dynamic());
        Self {
            windows,
            trajectory,
            resolution,
        }
    }
}

impl<'a, O, R, E> Source for PassSource<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<Frame>,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Out = Pass;
    type Stream = ItemStream<'a, Pass>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        let stage = MaterialisePass {
            station: StationView::of(self.windows.station),
            trajectory: Arc::clone(&self.trajectory),
            resolution: self.resolution,
        };
        ItemStream(Box::new(lift(self.windows.stack(interval)).then(stage)))
    }
}

// ---------------------------------------------------------------------------
// Inter-satellite windows
// ---------------------------------------------------------------------------

/// The knobs an inter-satellite stack needs, independent of how its inputs are
/// held. `slew_rate` is already resolved to the tighter of the pair's limits.
#[derive(Clone)]
pub(crate) struct InterSatelliteConfig {
    pub(crate) step: TimeDelta,
    pub(crate) central_body: Origin,
    pub(crate) slew_rate: Option<AngularRate>,
    pub(crate) min_range: Option<Distance>,
    pub(crate) max_range: Option<Distance>,
    /// See [`GroundSpaceConfig::cancel`].
    pub(crate) cancel: Option<CancellationToken>,
}

impl InterSatelliteConfig {
    pub(crate) fn new(step: TimeDelta, central_body: Origin) -> Self {
        Self {
            step,
            central_body,
            slew_rate: None,
            min_range: None,
            max_range: None,
            cancel: None,
        }
    }
}

/// The per-pair slew-rate limit: the tighter of the two assets' limits, or
/// whichever one is set.
pub(crate) fn effective_slew_rate(a: &Spacecraft, b: &Spacecraft) -> Option<AngularRate> {
    match (a.max_slew_rate(), b.max_slew_rate()) {
        (Some(a), Some(b)) => Some(if a.to_radians_per_second() < b.to_radians_per_second() {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Builds the staged inter-satellite window stack: range, then slew rate, then
/// central-body occultation, then any extra occulters.
///
/// The scenario's central body is always checked and needs no ephemeris. See
/// [`ground_space_stack`] for why `'x` comes from the handles.
pub(crate) fn inter_satellite_stack<'x, O, R, T, E>(
    traj1: T,
    traj2: T,
    ephemeris: E,
    occulters: Vec<Origin>,
    config: InterSatelliteConfig,
    interval: TimeInterval,
) -> BoxedWindows<'x>
where
    O: CoordinateOrigin + TryMeanRadius + Copy + Send + Sync + 'x,
    R: ReferenceFrame + Copy + Send + Sync + 'x,
    T: Deref<Target = Trajectory<O, R>> + Clone + Send + Sync + 'x,
    E: Deref + Clone + Send + Sync + 'x,
    E::Target: Ephemeris,
    <E::Target as Ephemeris>::Error: 'static,
{
    let step = config.step;
    let cancel = config.cancel.clone();

    let make_range = {
        let (t1, t2) = (traj1.clone(), traj2.clone());
        let cancel = cancel.clone();
        move |threshold: Distance, direction: RangeDirection, window: TimeInterval| {
            cancellable(
                InterSatelliteRangeDetectFn {
                    sc1: t1.clone(),
                    sc2: t2.clone(),
                    threshold,
                    direction,
                }
                .into_intervals(UniformSampler::new(step), window),
                &cancel,
            )
        }
    };

    // Range is position-only — the cheapest detector here, so it seeds the
    // stack. With both limits set the two scans are `intersect`ed over the
    // whole interval; the streaming merge that would let them nest instead is
    // design §10's deferred item.
    let mut windows: BoxedWindows<'x> = match (config.max_range, config.min_range) {
        (Some(max), Some(min)) => Box::new(
            make_range(max, RangeDirection::Max, interval).intersect(make_range(
                min,
                RangeDirection::Min,
                interval,
            )),
        ),
        (Some(max), None) => Box::new(make_range(max, RangeDirection::Max, interval)),
        (None, Some(min)) => Box::new(make_range(min, RangeDirection::Min, interval)),
        // Seeding with the whole interval lets every later stage apply
        // uniformly, whether or not range limits are set.
        (None, None) => Box::new(std::iter::once(Ok(interval))),
    };

    // Slew rate: position and velocity, no ephemeris.
    if let Some(threshold) = config.slew_rate {
        let (t1, t2) = (traj1.clone(), traj2.clone());
        let cancel = cancel.clone();
        windows = Box::new(windows.then_within(move |window| {
            cancellable(
                InterSatelliteSlewRateDetectFn {
                    sc1: t1.clone(),
                    sc2: t2.clone(),
                    threshold,
                }
                .into_intervals(UniformSampler::new(step), window),
                &cancel,
            )
        }));
    }

    // Central-body occultation always applies, and needs no ephemeris.
    {
        let (t1, t2) = (traj1.clone(), traj2.clone());
        let body = config.central_body;
        let cancel = cancel.clone();
        windows = Box::new(windows.then_within(move |window| {
            cancellable(
                InterSatLosCentralBodyDetectFn {
                    sc1: t1.clone(),
                    sc2: t2.clone(),
                    body,
                }
                .into_intervals(UniformSampler::new(step), window),
                &cancel,
            )
        }));
    }

    // Additional occulters are the most expensive: ephemeris per sample.
    for body in occulters {
        let (t1, t2) = (traj1.clone(), traj2.clone());
        let eph = ephemeris.clone();
        let cancel = cancel.clone();
        windows = Box::new(windows.then_within(move |window| {
            cancellable(
                InterSatLosOccluderDetectFn {
                    sc1: t1.clone(),
                    sc2: t2.clone(),
                    body,
                    ephemeris: eph.clone(),
                }
                .into_intervals(UniformSampler::new(step), window),
                &cancel,
            )
        }));
    }

    windows
}

/// The inter-satellite source over borrowed inputs.
pub(crate) struct InterSatelliteSource<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    traj1: &'a Trajectory<O, R>,
    traj2: &'a Trajectory<O, R>,
    ephemeris: &'a E,
    occulting_bodies: &'a [Origin],
    config: InterSatelliteConfig,
}

impl<'a, O, R, E> InterSatelliteSource<'a, O, R, E>
where
    O: CoordinateOrigin + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates an inter-satellite source checking only `central_body`.
    pub(crate) fn new(
        sc1: &Spacecraft,
        sc2: &Spacecraft,
        traj1: &'a Trajectory<O, R>,
        traj2: &'a Trajectory<O, R>,
        ephemeris: &'a E,
        central_body: Origin,
        step: TimeDelta,
    ) -> Self {
        let mut config = InterSatelliteConfig::new(step, central_body);
        config.slew_rate = effective_slew_rate(sc1, sc2);
        Self {
            traj1,
            traj2,
            ephemeris,
            occulting_bodies: &[],
            config,
        }
    }

    /// Adds occulting bodies beyond the central body.
    pub(crate) fn with_occulting_bodies(mut self, bodies: &'a [Origin]) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Sets the range gate (design §6.1).
    pub(crate) fn with_range_limits(
        mut self,
        min_range: Option<Distance>,
        max_range: Option<Distance>,
    ) -> Self {
        self.config.min_range = min_range;
        self.config.max_range = max_range;
        self
    }

    /// Makes every scan in the stack cancellable.
    pub(crate) fn with_cancellation(mut self, cancel: Option<CancellationToken>) -> Self {
        self.config.cancel = cancel;
        self
    }

    fn stack(&self, interval: TimeInterval) -> BoxedWindows<'a> {
        inter_satellite_stack(
            self.traj1,
            self.traj2,
            self.ephemeris,
            self.occulting_bodies.to_vec(),
            self.config.clone(),
            interval,
        )
    }
}

impl<'a, O, R, E> Source for InterSatelliteSource<'a, O, R, E>
where
    O: CoordinateOrigin + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    type Out = Window;
    type Stream = ItemStream<'a, Window>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        ItemStream(Box::new(
            self.stack(interval)
                .map(|r| r.map(Window).map_err(AnalysisError::from)),
        ))
    }
}

// ---------------------------------------------------------------------------
// Eclipses
// ---------------------------------------------------------------------------

/// The eclipse source: the complement, within the scan interval, of the
/// windows where the spacecraft has line of sight to the Sun.
pub(crate) struct EclipseSource<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    trajectory: &'a Trajectory<O, R>,
    ephemeris: &'a E,
    step: TimeDelta,
}

impl<'a, O, R, E> EclipseSource<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates an eclipse source sampling at `step`.
    pub(crate) fn new(trajectory: &'a Trajectory<O, R>, ephemeris: &'a E, step: TimeDelta) -> Self {
        Self {
            trajectory,
            ephemeris,
            step,
        }
    }
}

impl<'a, O, R, E> Source for EclipseSource<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    type Out = Eclipse;
    type Stream = ItemStream<'a, Eclipse>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        let sunlit = crate::power::EclipseDetectFn {
            sc: self.trajectory,
            ephemeris: self.ephemeris,
        }
        .into_intervals(UniformSampler::new(self.step), interval);
        ItemStream(Box::new(
            sunlit
                .complement(interval)
                .map(|r| r.map(Eclipse).map_err(AnalysisError::from)),
        ))
    }
}

// ---------------------------------------------------------------------------
// Power's continuous channels — sampling functions, not sources
// ---------------------------------------------------------------------------

/// Samples the beta angle and solar flux across `interval` at `step`.
///
/// Neither channel is a [`Source`]: they are continuous functions of time with
/// a value at *every* instant, not a sparse stream of items. Forcing them into
/// `Source` would mean either one item per sample — a `TimeSeries` shredded
/// into thousands of anonymous points that the consumer has to reassemble — or
/// a single item holding the whole series, which is a `Source` in name only and
/// gains nothing from `then`, `filter_ok`, or laziness. So they stay plain
/// functions and only the fan-out layer generalises over them (migration §4.1).
///
/// Both are returned together because they share the per-sample Sun-position
/// lookup, which is the expensive part.
pub(crate) fn sample_sun_channels<O, R, E>(
    trajectory: &Trajectory<O, R>,
    ephemeris: &E,
    interval: TimeInterval,
    step: TimeDelta,
) -> Result<(TimeSeries, TimeSeries), AnalysisError>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    E: Ephemeris,
    E::Error: 'static,
{
    let epoch = interval.start();
    let mut offsets = Vec::new();
    let mut beta_values = Vec::new();
    let mut flux_values = Vec::new();

    for time in interval.step_by(step) {
        let tdb = time.to_scale(Tdb);
        let state = trajectory.at(time.into_dynamic());
        let h_hat = state.position().cross(state.velocity()).normalize();

        let r_sun = ephemeris
            .position(tdb, trajectory.origin(), Sun)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;

        offsets.push((time - epoch).to_seconds().to_f64());
        beta_values.push(beta_angle(h_hat, r_sun.normalize()));
        flux_values.push(solar_flux(r_sun.length()));
    }

    let beta = TimeSeries::try_new(
        epoch,
        offsets.clone(),
        beta_values,
        InterpolationType::Linear,
    )
    .expect("sampled series should have valid dimensions");
    let flux = TimeSeries::try_new(epoch, offsets, flux_values, InterpolationType::Linear)
        .expect("sampled series should have valid dimensions");
    Ok((beta, flux))
}

// ---------------------------------------------------------------------------
// Access windows
// ---------------------------------------------------------------------------

/// Annotates an access window with the pass direction at its midpoint.
#[cfg(feature = "imaging")]
pub(crate) struct AnnotateDirection<'a, O: CoordinateOrigin, R: ReferenceFrame> {
    trajectory: &'a Trajectory<O, R>,
    origin: O,
    body_fixed_frame: Frame,
}

#[cfg(feature = "imaging")]
impl<O, R> Stage<TimeInterval> for AnnotateDirection<'_, O, R>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Out = AccessWindow;
    type Error = EvalError;

    fn apply(&self, interval: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
        let midpoint = interval.start() + 0.5 * interval.duration();
        let sample = sub_sat_sample(
            self.trajectory,
            midpoint,
            self.origin,
            self.body_fixed_frame,
        )?;
        Ok(vec![AccessWindow {
            interval,
            direction: pass_direction_of(&sample),
        }])
    }
}

/// The access source: sub-satellite-geometry windows, annotated with pass
/// direction.
#[cfg(feature = "imaging")]
pub(crate) struct AccessSource<'a, P, O: CoordinateOrigin, R: ReferenceFrame> {
    payload: P,
    aoi: &'a Aoi,
    trajectory: &'a Trajectory<O, R>,
    origin: O,
    body_fixed_frame: Frame,
    step: TimeDelta,
}

#[cfg(feature = "imaging")]
impl<'a, P, O, R> AccessSource<'a, P, O, R>
where
    P: AccessPayload + Copy + Send + Sync,
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
{
    /// Creates an access source for one (payload, AOI) pair.
    pub(crate) fn new(
        payload: P,
        aoi: &'a Aoi,
        trajectory: &'a Trajectory<O, R>,
        origin: O,
        body_fixed_frame: Frame,
        step: TimeDelta,
    ) -> Self {
        Self {
            payload,
            aoi,
            trajectory,
            origin,
            body_fixed_frame,
            step,
        }
    }
}

#[cfg(feature = "imaging")]
impl<'a, P, O, R> Source for AccessSource<'a, P, O, R>
where
    P: AccessPayload + Copy + Send + Sync + 'a,
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Out = AccessWindow;
    type Stream = ItemStream<'a, AccessWindow>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        let windows = AccessDetectFn {
            payload: self.payload,
            aoi: self.aoi,
            trajectory: self.trajectory,
            origin: self.origin,
            body_fixed_frame: self.body_fixed_frame,
        }
        .into_intervals(UniformSampler::new(self.step), interval);
        let stage = AnnotateDirection {
            trajectory: self.trajectory,
            origin: self.origin,
            body_fixed_frame: self.body_fixed_frame,
        };
        ItemStream(Box::new(lift(Box::new(windows)).then(stage)))
    }
}

// ---------------------------------------------------------------------------
// Differential parity harness
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use itertools::Itertools as _;
    use lox_bodies::Origin;
    use lox_core::coords::Cartesian;
    use lox_core::units::AngularRate;
    use lox_ephem::spk::parser::Spk;
    use lox_frames::Frame;
    use lox_orbits::ground::GroundLocation;
    use lox_orbits::orbits::Ensemble;
    use lox_orbits::propagators::OrbitSource;
    use lox_test_utils::{data_file, read_data_file};
    use lox_time::Time;
    use lox_time::time_scales::Tdb;

    use crate::assets::{AssetId, Scenario};
    use crate::visibility::ElevationMask;

    use super::*;

    // -- Fixtures ------------------------------------------------------------

    pub(crate) fn ephemeris() -> &'static Spk {
        static SPK: OnceLock<Spk> = OnceLock::new();
        SPK.get_or_init(|| Spk::from_file(data_file("spice/de440s.bsp")).unwrap())
    }

    pub(crate) fn lunar_trajectory() -> Trajectory {
        Trajectory::from_csv_dynamic(
            &read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap()
    }

    pub(crate) fn cebreros() -> GroundStation {
        let coords = lox_core::coords::LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::try_new(coords, Origin::Earth).unwrap();
        GroundStation::new(
            "cebreros",
            location,
            ElevationMask::with_fixed_elevation(0.0),
        )
    }

    /// The ISS over a day: ~15.5 orbits, so eclipses are guaranteed. A lunar arc
    /// has none at all, and even a polar LEO can spend a short window entirely in
    /// sunlight, so anything eclipse-related needs a fixture like this or its
    /// assertions hold vacuously.
    pub(crate) fn iss_trajectory() -> Trajectory {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            b"1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996",
            b"2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731",
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        sgp4.with_step(TimeDelta::from_seconds(30))
            .propagate(Interval::new(t0, t0 + TimeDelta::from_hours(24)).into_dynamic())
            .unwrap()
            .into_dynamic()
    }

    /// Two OneWeb satellites in near-opposite planes: their crossing orbits
    /// give a genuinely non-trivial inter-satellite window structure, unlike
    /// the colocated fixture used elsewhere.
    fn oneweb_pair() -> (Trajectory, Trajectory) {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        let tle1 = Elements::from_tle(
            Some("ONEWEB-0012".to_string()),
            b"1 44057U 19010A   24322.58825131  .00000088  00000+0  19693-3 0  9993",
            b"2 44057  87.9092 343.6767 0002420  76.7970 283.3431 13.16592150275693",
        )
        .unwrap();
        let tle2 = Elements::from_tle(
            Some("ONEWEB-0017".to_string()),
            b"1 45132U 20008B   24322.88240834 -.00000016  00000+0 -81930-4 0  9998",
            b"2 45132  87.8896 151.0343 0001369  78.1189 282.0092 13.10376984232476",
        )
        .unwrap();

        let sgp4_1 = Sgp4::new(tle1).unwrap();
        let sgp4_2 = Sgp4::new(tle2).unwrap();
        let t0 = sgp4_1.time().max(sgp4_2.time());
        let interval = Interval::new(t0, t0 + TimeDelta::from_hours(2));

        let propagate = |sgp4: Sgp4| {
            sgp4.with_step(TimeDelta::from_seconds(10))
                .propagate(interval.into_dynamic())
                .unwrap()
                .into_dynamic()
        };
        (propagate(sgp4_1), propagate(sgp4_2))
    }

    /// Builds the `Scenario` + `Ensemble` the eager path needs, re-tagging each
    /// trajectory's epoch to TAI exactly as the existing tests do.
    pub(crate) fn scenario_and_ensemble(
        stations: &[GroundStation],
        spacecraft: &[Spacecraft],
        interval: TimeInterval,
    ) -> (Scenario<Origin, Frame>, Ensemble<AssetId, Origin, Frame>) {
        let scenario = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(stations)
            .with_spacecraft(spacecraft);
        let mut map = HashMap::new();
        for sc in spacecraft {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                map.insert(
                    sc.id().clone(),
                    Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data),
                );
            }
        }
        (scenario, Ensemble::new(map))
    }

    /// Wraps an ephemeris to count lookups, so a test can assert *where* the
    /// expensive work happens rather than only what it returns.
    struct CountingEphemeris<'a, E> {
        inner: &'a E,
        calls: AtomicUsize,
    }

    impl<'a, E> CountingEphemeris<'a, E> {
        fn new(inner: &'a E) -> Self {
            Self {
                inner,
                calls: AtomicUsize::new(0),
            }
        }

        fn count(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }

    impl<E: Ephemeris> Ephemeris for CountingEphemeris<'_, E> {
        type Error = E::Error;

        fn state<O1: CoordinateOrigin, O2: CoordinateOrigin>(
            &self,
            time: Time<Tdb>,
            origin: O1,
            target: O2,
        ) -> Result<Cartesian, Self::Error> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.inner.state(time, origin, target)
        }
    }

    // -- Comparison helpers --------------------------------------------------

    /// Absolute tolerance, in seconds, on a window boundary.
    ///
    /// Deliberately absolute rather than relative: J2000-relative epochs run to
    /// ~7.6e8 s, so even `rtol <= 1e-4` would admit a ±21-hour discrepancy.
    const BOUNDARY_TOL_S: f64 = 1e-6;

    /// Asserts the invariants `Intervals` guarantees by construction and every
    /// combinator must preserve: non-empty windows, sorted by start, pairwise
    /// disjoint, and contained in the scan interval.
    ///
    /// These were differential assertions against the eager implementation until
    /// it was deleted. What is left is weaker but still real — a mis-staged
    /// detector or a bad merge in `intersect`/`then_within` breaks one of them.
    fn assert_window_invariants(label: &str, windows: &[TimeInterval], scan: TimeInterval) {
        assert!(!windows.is_empty(), "{label}: no windows");
        for (i, w) in windows.iter().enumerate() {
            let duration = w.duration().to_seconds().to_f64();
            assert!(duration > 0.0, "{label}: window {i} is empty");
            assert!(
                w.start() >= scan.start() && w.end() <= scan.end(),
                "{label}: window {i} escapes the scan interval"
            );
        }
        for (i, pair) in windows.windows(2).enumerate() {
            let gap = (pair[1].start() - pair[0].end()).to_seconds().to_f64();
            assert!(
                gap >= -BOUNDARY_TOL_S,
                "{label}: windows {i} and {} overlap or are out of order",
                i + 1
            );
        }
    }

    fn assert_series_match(label: &str, old: &TimeSeries, new: &TimeSeries, tol: f64) {
        assert_eq!(
            old.values().len(),
            new.values().len(),
            "{label}: sample count differs"
        );
        let d_epoch = (old.epoch() - new.epoch()).to_seconds().to_f64().abs();
        assert!(d_epoch <= BOUNDARY_TOL_S, "{label}: epoch differs");
        for (i, (a, b)) in old.values().iter().zip(new.values()).enumerate() {
            assert!(
                (a - b).abs() <= tol,
                "{label}: sample {i} differs: {a} vs {b}"
            );
        }
    }

    /// Runs a source to completion, panicking on the first error.
    pub(crate) fn collect<S: Source>(source: &S, interval: TimeInterval) -> Vec<S::Out> {
        source
            .detect(interval)
            .try_collect()
            .expect("pipeline source failed")
    }

    // -- Ground-space parity -------------------------------------------------

    /// Runs the eager and pipeline ground-space paths over the lunar fixture
    /// and returns both window lists. `occulters` empty exercises the
    /// `NoEphemeris` variant of `compute()`.
    fn ground_space_windows(
        occulters: &[Origin],
        adaptive: bool,
    ) -> (Vec<TimeInterval>, TimeInterval) {
        let traj = lunar_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let gs = cebreros();
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(traj));
        let spacecraft = [sc.clone()];
        let (_scenario, ensemble) =
            scenario_and_ensemble(std::slice::from_ref(&gs), &spacecraft, interval);
        let traj = ensemble.get(sc.id()).expect("trajectory missing");
        let step = TimeDelta::from_seconds(60);

        let source =
            GroundSpaceWindows::new(&gs, traj, ephemeris(), step).with_occulting_bodies(occulters);
        let source = if adaptive { source.adaptive() } else { source };
        (collect(&source, interval), interval)
    }

    #[test]
    fn parity_ground_space_elevation_only() {
        let (windows, interval) = ground_space_windows(&[], false);
        assert_window_invariants("ground-space elevation only", &windows, interval);
    }

    #[test]
    fn parity_ground_space_adaptive() {
        let (windows, interval) = ground_space_windows(&[], true);
        assert_window_invariants("ground-space adaptive", &windows, interval);
    }

    #[test]
    fn parity_ground_space_single_occulter() {
        let (windows, interval) = ground_space_windows(&[Origin::Moon], false);
        assert_window_invariants("ground-space one occulter", &windows, interval);
    }

    #[test]
    fn parity_ground_space_multi_occulter() {
        // Two occulters exercise the `intersect` fallback inside the staged LOS scan.
        let (windows, interval) = ground_space_windows(&[Origin::Moon, Origin::Venus], false);
        assert_window_invariants("ground-space two occulters", &windows, interval);
    }

    // -- Inter-satellite parity ----------------------------------------------

    fn inter_satellite_windows(
        min_range: Option<Distance>,
        max_range: Option<Distance>,
        slew_rate: Option<AngularRate>,
        occulters: &[Origin],
    ) -> (Vec<TimeInterval>, TimeInterval) {
        let (traj1, traj2) = oneweb_pair();
        let interval = TimeInterval::new(traj1.start_time(), traj1.end_time());

        let mut sc1 = Spacecraft::new("ow12", OrbitSource::Trajectory(traj1));
        let mut sc2 = Spacecraft::new("ow17", OrbitSource::Trajectory(traj2));
        if let Some(rate) = slew_rate {
            sc1 = sc1.with_max_slew_rate(rate);
            sc2 = sc2.with_max_slew_rate(rate);
        }
        let spacecraft = [sc1.clone(), sc2.clone()];
        let (_scenario, ensemble) = scenario_and_ensemble(&[], &spacecraft, interval);
        let t1 = ensemble.get(sc1.id()).expect("trajectory missing");
        let t2 = ensemble.get(sc2.id()).expect("trajectory missing");

        let source = InterSatelliteSource::new(
            &spacecraft[0],
            &spacecraft[1],
            t1,
            t2,
            ephemeris(),
            Origin::Earth,
            TimeDelta::from_seconds(60),
        )
        .with_range_limits(min_range, max_range)
        .with_occulting_bodies(occulters);
        let windows = collect(&source, interval)
            .into_iter()
            .map(|w| w.0)
            .collect();
        (windows, interval)
    }

    #[test]
    fn parity_inter_satellite_central_body_only() {
        let (windows, interval) = inter_satellite_windows(None, None, None, &[]);
        assert_window_invariants("inter-satellite central body", &windows, interval);
    }

    #[test]
    fn parity_inter_satellite_max_range() {
        let (windows, interval) =
            inter_satellite_windows(None, Some(Distance::kilometers(5000.0)), None, &[]);
        assert_window_invariants("inter-satellite max range", &windows, interval);
    }

    #[test]
    fn parity_inter_satellite_min_and_max_range() {
        // Both limits set takes the eager `intersect` path inside the source.
        let (windows, interval) = inter_satellite_windows(
            Some(Distance::kilometers(100.0)),
            Some(Distance::kilometers(5000.0)),
            None,
            &[],
        );
        assert_window_invariants("inter-satellite min+max range", &windows, interval);
    }

    #[test]
    fn parity_inter_satellite_slew_rate() {
        let (windows, interval) =
            inter_satellite_windows(None, None, Some(AngularRate::degrees_per_second(0.05)), &[]);
        assert_window_invariants("inter-satellite slew rate", &windows, interval);
    }

    #[test]
    fn parity_inter_satellite_extra_occulter() {
        let (windows, interval) = inter_satellite_windows(None, None, None, &[Origin::Moon]);
        assert_window_invariants("inter-satellite extra occulter", &windows, interval);
    }

    // -- Power parity --------------------------------------------------------

    // -- Access parity -------------------------------------------------------

    // -- Structural assertions the parity suite cannot make ------------------

    #[test]
    fn range_is_a_gate_inside_the_source_not_a_stage_after_it() {
        // The discriminator between the two placements (design §6.1): a `.then`
        // stage sees materialised passes and can only keep or drop each one, so
        // it can never turn one pass into two. A gate inside the source re-runs
        // root-finding within the window, so a `min_range` threshold that the
        // range dips below mid-pass splits that pass around its closest
        // approach. Counting the split is the assertion prose cannot make.
        //
        // A LEO fixture, not the lunar one: over a lunar visibility arc the
        // slant range is monotonic (the Moon's own recession dwarfs the
        // station's ~6400 km of topocentric parallax), so no threshold is
        // crossed twice and there is nothing to split.
        let (traj, _) = oneweb_pair();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let gs = cebreros();
        let sc = Spacecraft::new("ow12", OrbitSource::Trajectory(traj));
        let stations = [gs.clone()];
        let spacecraft = [sc.clone()];
        let (_scenario, ensemble) = scenario_and_ensemble(&stations, &spacecraft, interval);
        let traj = ensemble.get(sc.id()).expect("trajectory missing");
        let step = TimeDelta::from_seconds(10);

        let ungated = collect(
            &GroundSpaceWindows::new(&gs, traj, ephemeris(), step),
            interval,
        );
        assert!(!ungated.is_empty(), "fixture produced no windows");

        // The longest window gives the range the most room to vary within it.
        let target = *ungated
            .iter()
            .max_by(|a, b| {
                a.duration()
                    .to_seconds()
                    .to_f64()
                    .total_cmp(&b.duration().to_seconds().to_f64())
            })
            .unwrap();

        let range_at = |t| {
            let state = traj
                .at(t)
                .try_to_frame(gs.body_fixed_frame(), &DefaultRotationProvider)
                .unwrap();
            gs.location()
                .compute_observables(state.position(), state.velocity())
                .range()
        };
        // Find a threshold the range crosses *twice* inside the window: one
        // strictly between an interior extremum and both endpoints. Which
        // direction gates it depends on whether the extremum is a peak or a
        // trough, which is a property of the fixture, not of the design.
        const N: usize = 20;
        let samples: Vec<f64> = (0..=N)
            .map(|i| range_at(target.start() + (i as f64 / N as f64) * target.duration()))
            .collect();
        let edge_max = samples[0].max(samples[N]);
        let edge_min = samples[0].min(samples[N]);
        let interior_max = samples[1..N]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let interior_min = samples[1..N].iter().copied().fold(f64::INFINITY, f64::min);

        let source = GroundSpaceWindows::new(&gs, traj, ephemeris(), step);
        let gated = if interior_max > edge_max {
            // Range peaks mid-window: a max_range gate carves out the middle.
            let threshold = Distance::meters(0.5 * (edge_max + interior_max));
            collect(&source.with_range_limits(None, Some(threshold)), interval)
        } else if interior_min < edge_min {
            // Range troughs mid-window: a min_range gate carves out the middle.
            let threshold = Distance::meters(0.5 * (edge_min + interior_min));
            collect(&source.with_range_limits(Some(threshold), None), interval)
        } else {
            panic!(
                "fixture assumption broken: range is monotonic across the window, \
                 so no threshold is crossed twice"
            );
        };

        let inside = gated
            .iter()
            .filter(|w| w.start() >= target.start() && w.end() <= target.end())
            .count();
        assert_eq!(
            inside, 2,
            "the min_range gate should split one pass into two around its closest \
             approach; got {inside} window(s) inside it"
        );
    }

    #[test]
    fn staging_keeps_the_expensive_detector_out_of_ruled_out_stretches() {
        // The load-bearing claim behind `then_within` (design §4.5): the
        // ephemeris-backed LOS detector must only sample inside the windows
        // elevation already admitted. Counting lookups tests the *cost*
        // ordering, which no comparison of results can.
        let traj = lunar_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let gs = cebreros();
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(traj));
        let stations = [gs.clone()];
        let spacecraft = [sc.clone()];
        let (_scenario, ensemble) = scenario_and_ensemble(&stations, &spacecraft, interval);
        let traj = ensemble.get(sc.id()).expect("trajectory missing");
        let step = TimeDelta::from_seconds(60);
        let occulters = [Origin::Moon];

        let counting = CountingEphemeris::new(ephemeris());
        let staged =
            GroundSpaceWindows::new(&gs, traj, &counting, step).with_occulting_bodies(&occulters);
        let windows = collect(&staged, interval);
        let staged_calls = counting.count();

        // What an unstaged `intersect` would have cost: the LOS detector swept
        // over the whole interval at the same step.
        let counting = CountingEphemeris::new(ephemeris());
        let unstaged: Vec<TimeInterval> = LineOfSightDetectFn {
            gs: gs.location().clone(),
            sc: traj,
            body: Origin::Moon,
            ephemeris: &counting,
            body_fixed_frame: gs.body_fixed_frame(),
        }
        .into_intervals(UniformSampler::new(step), interval)
        .collect::<Result<_, _>>()
        .expect("los scan failed");
        let unstaged_calls = counting.count();

        assert!(!windows.is_empty(), "fixture produced no windows");
        assert!(!unstaged.is_empty(), "LOS scan produced no windows");
        assert!(
            staged_calls < unstaged_calls,
            "staging saved nothing: {staged_calls} lookups staged vs {unstaged_calls} unstaged"
        );
    }

    #[test]
    fn a_source_is_lazy_enough_to_stop_early() {
        // `take(1)` must not drive the scan to the end of the interval. The
        // ephemeris counter is the observable: a lazy source stops looking up
        // Moon positions once the first window is complete.
        let traj = lunar_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let gs = cebreros();
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(traj));
        let stations = [gs.clone()];
        let spacecraft = [sc.clone()];
        let (_scenario, ensemble) = scenario_and_ensemble(&stations, &spacecraft, interval);
        let traj = ensemble.get(sc.id()).expect("trajectory missing");
        let step = TimeDelta::from_seconds(60);
        let occulters = [Origin::Moon];

        let counting = CountingEphemeris::new(ephemeris());
        let source =
            GroundSpaceWindows::new(&gs, traj, &counting, step).with_occulting_bodies(&occulters);
        let first: Vec<_> = source
            .detect(interval)
            .take(1)
            .try_collect::<TimeInterval, Vec<_>, AnalysisError>()
            .expect("source failed");
        let early_calls = counting.count();

        let counting = CountingEphemeris::new(ephemeris());
        let source =
            GroundSpaceWindows::new(&gs, traj, &counting, step).with_occulting_bodies(&occulters);
        let all = collect(&source, interval);
        let full_calls = counting.count();

        assert_eq!(first.len(), 1);
        assert!(!all.is_empty());
        assert!(
            early_calls < full_calls,
            "take(1) cost as much as the full scan: {early_calls} vs {full_calls} lookups"
        );
    }
}

// ---------------------------------------------------------------------------
// Manual inspection
// ---------------------------------------------------------------------------
