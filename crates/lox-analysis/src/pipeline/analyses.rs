// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Pipeline-backed replacements for the eager `*Analysis` types.
//!
//! These carry the names the old implementations hand over at the hard cut (plan
//! step 7). Until then they coexist under this module path, so
//! `pipeline::analyses::VisibilityAnalysis` and the old
//! `visibility::VisibilityAnalysis` are both reachable and the old public API is
//! untouched (migration §2). At the cut these move up and the old ones go.
//!
//! **Why these types exist at all**, given that fan-out is supposed to be the
//! caller's business (design §7): Python binds *Rust*, so something on the Rust
//! side has to own the fan-out for it. Rust callers may use them, but a Rust
//! caller who wants anything else — a different pool, a filtered target list,
//! early exit — is better served by [`single`](VisibilityAnalysis::single) and
//! their own iterator. That is why the pair-selection filters of the old API are
//! *gone* rather than reimplemented (design §8): filtering a target list is one
//! line at the call site, and threading predicates through a batch runner to
//! save it was never a good trade.
//!
//! Each `run` returns **per-target** `Result`s, so one unresolvable ephemeris
//! cannot sink a 500-pair batch.

use std::sync::Arc;

use itertools::Itertools as _;
use lox_bodies::{CoordinateOrigin, Origin, TryMeanRadius, TrySpheroid};
use lox_core::units::Distance;
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::series::TimeSeries;
use lox_time::time_scales::TimeScale;

use crate::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use crate::pipeline::sources::{
    Eclipse, EclipseSource, GroundSpaceConfig, InterSatelliteConfig, MaterialisePass, StationView,
    Window, effective_slew_rate, ground_space_stack, inter_satellite_stack, lift,
    sample_sun_channels,
};
use crate::pipeline::{AnalysisError, HasInterval, PipelineExt as _, Source as _};
use crate::visibility::Pass;

#[cfg(feature = "parallel")]
use crate::pipeline::Parallelism;

#[cfg(feature = "async")]
use crate::pipeline::sources::ItemStream;
#[cfg(feature = "async")]
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// One `run` outcome per target, keyed rather than positional.
///
/// The `Result` is **per target**: one unresolvable ephemeris fails its own pair
/// and nothing else. The key is carried rather than implied by position because
/// rayon and the streaming path both complete out of order.
pub type Keyed<K, T> = Vec<(K, Result<T, AnalysisError>)>;

/// A ground-station-to-spacecraft or spacecraft-to-spacecraft pair id.
pub type PairId = (AssetId, AssetId);

/// The coarsest scan step that cannot miss a pass of at least `min_duration`.
///
/// Half the minimum duration is the bound: a pass shorter than one step can fall
/// between samples entirely. This is the *only* thing `min_pass_duration` does
/// to the scan — the actual discarding is a [`filter_ok`](itertools::Itertools)
/// stage on materialised items (design §8), and only a caller holding both knobs
/// can make this trade, which is why it lives here rather than in the source.
fn scan_step(step: TimeDelta, min_duration: Option<TimeDelta>) -> TimeDelta {
    match min_duration {
        Some(d) if 0.5 * d > step => 0.5 * d,
        _ => step,
    }
}

/// Keeps only items lasting at least `min_duration`.
fn long_enough<T: HasInterval>(item: &T, min_duration: Option<TimeDelta>) -> bool {
    min_duration.is_none_or(|min| item.interval().duration() >= min)
}

/// Maps `f` over `targets` under the requested parallelism.
///
/// `Rayon(Some(n))` builds a **local** pool: the global pool cannot be resized
/// per call, and a server sizing one request must not disturb every other one.
/// A pool that cannot be built falls back to the global pool rather than failing
/// the run — the alternative would make every `run` return a `Result` for a
/// condition that only arises under thread exhaustion.
#[cfg(feature = "parallel")]
fn map_targets<T, U>(
    targets: Vec<T>,
    parallelism: Parallelism,
    f: impl Fn(T) -> U + Send + Sync,
) -> Vec<U>
where
    T: Send,
    U: Send,
{
    use rayon::prelude::*;

    match parallelism {
        Parallelism::Sequential => targets.into_iter().map(f).collect(),
        Parallelism::Rayon(None) => targets.into_par_iter().map(f).collect(),
        Parallelism::Rayon(Some(n)) => match rayon::ThreadPoolBuilder::new().num_threads(n).build()
        {
            Ok(pool) => pool.install(|| targets.into_par_iter().map(f).collect()),
            Err(_) => targets.into_par_iter().map(f).collect(),
        },
    }
}

// ---------------------------------------------------------------------------
// VisibilityAnalysis — ground-space, yields `Pass`
// ---------------------------------------------------------------------------

/// Ground-station-to-spacecraft visibility, yielding [`Pass`] items with
/// azimuth/elevation/range observables.
pub struct VisibilityAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: &'a E,
    occulting_bodies: Vec<Origin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
    adaptive: bool,
}

impl<'a, O, R, E> VisibilityAnalysis<'a, O, R, E>
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
    /// Creates a visibility analysis over a propagated scenario.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, O, R>,
        ephemeris: &'a E,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_pass_duration: None,
            min_range: None,
            max_range: None,
            adaptive: false,
        }
    }

    /// Adds occulting bodies, checked by an ephemeris-backed line-of-sight
    /// detector inside the elevation windows.
    pub fn with_occulting_bodies(mut self, bodies: Vec<Origin>) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Sets the sampling step for event detection and observables.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Discards passes shorter than `min_pass_duration`, and coarsens the scan
    /// as far as that allows.
    pub fn with_min_pass_duration(mut self, min_pass_duration: TimeDelta) -> Self {
        self.min_pass_duration = Some(min_pass_duration);
        self
    }

    /// Restricts passes by slant range.
    pub fn with_range_limits(
        mut self,
        min_range: Option<Distance>,
        max_range: Option<Distance>,
    ) -> Self {
        self.min_range = min_range;
        self.max_range = max_range;
        self
    }

    /// Drives the elevation scan by its own rate bound rather than a fixed step.
    pub fn with_adaptive_detection(mut self) -> Self {
        self.adaptive = true;
        self
    }

    fn config(&self) -> GroundSpaceConfig {
        GroundSpaceConfig {
            step: scan_step(self.step, self.min_pass_duration),
            min_range: self.min_range,
            max_range: self.max_range,
            adaptive: self.adaptive,
            cancel: None,
        }
    }

    /// Computes the passes for one (station, spacecraft) pair.
    ///
    /// Returns an error if the spacecraft has no trajectory in the ensemble —
    /// unlike the eager path, which panicked.
    pub fn single(
        &self,
        station: &GroundStation,
        spacecraft: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<Vec<Pass>, AnalysisError> {
        let trajectory = self.trajectory(spacecraft.id())?;
        let windows = ground_space_stack(
            StationView::of(station),
            trajectory,
            self.ephemeris,
            self.occulting_bodies.clone(),
            self.config(),
            interval,
        );
        let stage = MaterialisePass {
            station: StationView::of(station),
            trajectory: Arc::new(trajectory.clone().into_dynamic()),
            resolution: self.step,
        };
        let min_duration = self.min_pass_duration;
        lift(windows)
            .then(stage)
            .filter_ok(|pass| long_enough(pass, min_duration))
            .try_collect()
    }

    fn trajectory(&self, id: &AssetId) -> Result<&'a Trajectory<O, R>, AnalysisError> {
        self.ensemble.get(id).ok_or_else(|| missing_trajectory(id))
    }

    /// Every (station, spacecraft) pair in the scenario.
    ///
    /// Exposed so a caller can filter, reorder, or iterate the targets itself —
    /// which is the recommended path, and the only one in a build without the
    /// `parallel` feature.
    pub fn pairs(&self) -> Vec<(&'a GroundStation, &'a Spacecraft)> {
        let spacecraft = self.scenario.spacecraft();
        self.scenario
            .ground_stations()
            .iter()
            .flat_map(|gs| spacecraft.iter().map(move |sc| (gs, sc)))
            .collect()
    }

    /// Computes passes for every pair from [`pairs`](Self::pairs).
    ///
    /// Results arrive keyed, never positionally: under `Rayon` they complete out
    /// of order.
    #[cfg(feature = "parallel")]
    pub fn run(
        &self,
        interval: TimeInterval,
        parallelism: Parallelism,
    ) -> Keyed<PairId, Vec<Pass>> {
        map_targets(self.pairs(), parallelism, |(gs, sc)| {
            (
                (gs.id().clone(), sc.id().clone()),
                self.single(gs, sc, interval),
            )
        })
    }
}

// ---------------------------------------------------------------------------
// InterSatelliteAnalysis — yields `Window`
// ---------------------------------------------------------------------------

/// Spacecraft-to-spacecraft contacts, yielding [`Window`] items.
///
/// A separate type rather than a boolean on [`VisibilityAnalysis`] because the
/// item type genuinely differs (design §8): sat-to-sat contacts have no ground
/// observables, so they cannot be [`Pass`]es.
pub struct InterSatelliteAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: &'a E,
    occulting_bodies: Vec<Origin>,
    step: TimeDelta,
    min_duration: Option<TimeDelta>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

impl<'a, O, R, E> InterSatelliteAnalysis<'a, O, R, E>
where
    O: CoordinateOrigin + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates an inter-satellite analysis over a propagated scenario.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, O, R>,
        ephemeris: &'a E,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_duration: None,
            min_range: None,
            max_range: None,
        }
    }

    /// Adds occulting bodies beyond the scenario's central body, which is always
    /// checked.
    pub fn with_occulting_bodies(mut self, bodies: Vec<Origin>) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Sets the sampling step for event detection.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Discards windows shorter than `min_duration`.
    pub fn with_min_duration(mut self, min_duration: TimeDelta) -> Self {
        self.min_duration = Some(min_duration);
        self
    }

    /// Restricts contacts by range.
    pub fn with_range_limits(
        mut self,
        min_range: Option<Distance>,
        max_range: Option<Distance>,
    ) -> Self {
        self.min_range = min_range;
        self.max_range = max_range;
        self
    }

    /// Computes the contact windows for one spacecraft pair.
    pub fn single(
        &self,
        a: &Spacecraft,
        b: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<Vec<Window>, AnalysisError> {
        let traj_a = self
            .ensemble
            .get(a.id())
            .ok_or_else(|| missing_trajectory(a.id()))?;
        let traj_b = self
            .ensemble
            .get(b.id())
            .ok_or_else(|| missing_trajectory(b.id()))?;

        let mut config = InterSatelliteConfig::new(
            scan_step(self.step, self.min_duration),
            self.scenario.origin().into(),
        );
        config.slew_rate = effective_slew_rate(a, b);
        config.min_range = self.min_range;
        config.max_range = self.max_range;

        let min_duration = self.min_duration;
        inter_satellite_stack(
            traj_a,
            traj_b,
            self.ephemeris,
            self.occulting_bodies.clone(),
            config,
            interval,
        )
        .map(|r| r.map(Window).map_err(AnalysisError::from))
        .filter_ok(|window| long_enough(window, min_duration))
        .try_collect()
    }

    /// Every unordered spacecraft pair in the scenario.
    ///
    /// See [`VisibilityAnalysis::pairs`] for why this is public.
    pub fn pairs(&self) -> Vec<(&'a Spacecraft, &'a Spacecraft)> {
        let spacecraft = self.scenario.spacecraft();
        (0..spacecraft.len())
            .flat_map(|i| ((i + 1)..spacecraft.len()).map(move |j| (i, j)))
            .map(|(i, j)| (&spacecraft[i], &spacecraft[j]))
            .collect()
    }

    /// Computes contact windows for every pair from [`pairs`](Self::pairs).
    #[cfg(feature = "parallel")]
    pub fn run(
        &self,
        interval: TimeInterval,
        parallelism: Parallelism,
    ) -> Keyed<PairId, Vec<Window>> {
        map_targets(self.pairs(), parallelism, |(a, b)| {
            (
                (a.id().clone(), b.id().clone()),
                self.single(a, b, interval),
            )
        })
    }
}

// ---------------------------------------------------------------------------
// PowerBudgetAnalysis — three heterogeneous channels
// ---------------------------------------------------------------------------

/// One spacecraft's power-budget outputs.
///
/// Not an aggregate results object: it is the *per-target* value, so a failing
/// spacecraft yields `Err` for itself alone.
pub struct SpacecraftPower {
    /// Intervals in the body's umbra (no penumbra model).
    pub eclipses: Vec<Eclipse>,
    /// Sun beta angle over the arc, in radians.
    pub beta: TimeSeries,
    /// Solar flux over the arc, in W/m².
    pub flux: TimeSeries,
}

/// Eclipse intervals plus the continuous beta-angle and solar-flux channels.
///
/// The pipeline covers only the eclipses (migration §4.1): beta and flux are
/// continuous functions with a value at every instant, not sparse item streams,
/// so they are sampled directly. Forcing them through `Source` would either
/// shred a `TimeSeries` into thousands of anonymous items or wrap the whole
/// series in a single item that gains nothing from `then` or laziness.
pub struct PowerBudgetAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: &'a E,
    step: TimeDelta,
}

impl<'a, O, R, E> PowerBudgetAnalysis<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates a power-budget analysis over a propagated scenario.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, O, R>,
        ephemeris: &'a E,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris,
            step: TimeDelta::from_seconds(60),
        }
    }

    /// Sets the sampling step for detection and for the continuous channels.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Computes one spacecraft's eclipse intervals.
    pub fn eclipses(
        &self,
        spacecraft: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<Vec<Eclipse>, AnalysisError> {
        let trajectory = self
            .ensemble
            .get(spacecraft.id())
            .ok_or_else(|| missing_trajectory(spacecraft.id()))?;
        EclipseSource::new(trajectory, self.ephemeris, self.step)
            .detect(interval)
            .try_collect()
    }

    /// Samples one spacecraft's beta angle.
    pub fn beta_angle(
        &self,
        spacecraft: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<TimeSeries, AnalysisError> {
        self.channels(spacecraft, interval).map(|(beta, _)| beta)
    }

    /// Samples one spacecraft's solar flux.
    pub fn solar_flux(
        &self,
        spacecraft: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<TimeSeries, AnalysisError> {
        self.channels(spacecraft, interval).map(|(_, flux)| flux)
    }

    fn channels(
        &self,
        spacecraft: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<(TimeSeries, TimeSeries), AnalysisError> {
        let trajectory = self
            .ensemble
            .get(spacecraft.id())
            .ok_or_else(|| missing_trajectory(spacecraft.id()))?;
        sample_sun_channels(trajectory, self.ephemeris, interval, self.step)
    }

    /// Every spacecraft in the scenario.
    ///
    /// See [`VisibilityAnalysis::pairs`] for why this is public.
    pub fn spacecraft(&self) -> Vec<&'a Spacecraft> {
        self.scenario.spacecraft().iter().collect()
    }

    /// Computes all three channels for every spacecraft from
    /// [`spacecraft`](Self::spacecraft).
    #[cfg(feature = "parallel")]
    pub fn run(
        &self,
        interval: TimeInterval,
        parallelism: Parallelism,
    ) -> Keyed<AssetId, SpacecraftPower> {
        map_targets(self.spacecraft(), parallelism, |sc| {
            let power = self.eclipses(sc, interval).and_then(|eclipses| {
                // One sampling pass for both channels: they share the
                // per-sample Sun lookup, which is the expensive part.
                let (beta, flux) = self.channels(sc, interval)?;
                Ok(SpacecraftPower {
                    eclipses,
                    beta,
                    flux,
                })
            });
            (sc.id().clone(), power)
        })
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
#[error("no trajectory for {0} in the ensemble; was the scenario propagated?")]
struct MissingTrajectory(AssetId);

fn missing_trajectory(id: &AssetId) -> AnalysisError {
    AnalysisError::Stage(lox_core::error::LoxError::new(MissingTrajectory(
        id.clone(),
    )))
}

// ---------------------------------------------------------------------------
// Streaming forks — owned inputs, visibility only in v1
// ---------------------------------------------------------------------------

/// The per-spacecraft trajectory handles a streaming analysis shares with its
/// workers.
///
/// `Arc` per *spacecraft*, not per pair: a station-by-spacecraft fan-out reuses
/// the same trajectory across every station, so cloning per pair would multiply
/// the largest input in the scenario by the number of stations.
#[cfg(feature = "async")]
type SharedTrajectories = HashMap<AssetId, Arc<Trajectory>>;

#[cfg(feature = "async")]
fn share_trajectories(ensemble: &Ensemble<AssetId, Origin, Frame>) -> SharedTrajectories {
    ensemble
        .iter()
        .map(|(id, traj)| (id.clone(), Arc::new(traj.clone())))
        .collect()
}

/// Streams ground-space passes per (station, spacecraft) pair.
///
/// A distinct owned type rather than a method on [`VisibilityAnalysis`]: the
/// returned stream outlives the call, so every input a worker touches has to be
/// owned or `Arc`-shared (design §7.1). Consumed by value for the same reason.
///
/// Restricted to runtime-typed (`Origin`/`Frame`) trajectories, which is what
/// the server-side and Python callers this exists for already have; the typed
/// forms stay on the borrowing path.
#[cfg(feature = "async")]
pub struct VisibilityStreamAnalysis<E> {
    pairs: Vec<((AssetId, AssetId), StationView, Arc<Trajectory>)>,
    ephemeris: Arc<E>,
    occulting_bodies: Vec<Origin>,
    config: GroundSpaceConfig,
    resolution: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
}

#[cfg(feature = "async")]
impl<E> VisibilityStreamAnalysis<E>
where
    E: Ephemeris + Send + Sync + 'static,
    E::Error: 'static,
{
    /// Snapshots a scenario into owned, shareable inputs.
    pub fn new(
        scenario: &Scenario<Origin, Frame>,
        ensemble: &Ensemble<AssetId, Origin, Frame>,
        ephemeris: Arc<E>,
    ) -> Self {
        let shared = share_trajectories(ensemble);
        let pairs = scenario
            .ground_stations()
            .iter()
            .flat_map(|gs| {
                let shared = &shared;
                scenario.spacecraft().iter().filter_map(move |sc| {
                    let traj = shared.get(sc.id())?;
                    Some((
                        (gs.id().clone(), sc.id().clone()),
                        StationView::of(gs),
                        Arc::clone(traj),
                    ))
                })
            })
            .collect();
        Self {
            pairs,
            ephemeris,
            occulting_bodies: Vec::new(),
            config: GroundSpaceConfig::new(TimeDelta::from_seconds(60)),
            resolution: TimeDelta::from_seconds(60),
            min_pass_duration: None,
        }
    }

    /// Adds occulting bodies.
    pub fn with_occulting_bodies(mut self, bodies: Vec<Origin>) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Sets the sampling step for detection and observables.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.resolution = step;
        self.config.step = step;
        self
    }

    /// Discards passes shorter than `min_pass_duration`.
    pub fn with_min_pass_duration(mut self, min_pass_duration: TimeDelta) -> Self {
        self.min_pass_duration = Some(min_pass_duration);
        self.config.step = scan_step(self.resolution, Some(min_pass_duration));
        self
    }

    /// Streams every pair's passes, interleaved in completion order.
    ///
    /// `cancel` is additive with the stream's own drop guard, and is threaded
    /// into each scan, so shutdown is bounded by one detector evaluation rather
    /// than one pass.
    pub fn stream(
        self,
        interval: TimeInterval,
        cancel: Option<lox_core::sync::CancellationToken>,
    ) -> crate::stream::AnalysisStream<PairId, Pass> {
        let Self {
            pairs,
            ephemeris,
            occulting_bodies,
            config,
            resolution,
            min_pass_duration,
        } = self;

        let inputs = pairs
            .into_iter()
            .map(|(key, station, trajectory)| (key, (station, trajectory)));

        crate::stream::stream(
            inputs,
            move |(station, trajectory), cancel| {
                let mut config = config.clone();
                config.cancel = Some(cancel);
                let windows = ground_space_stack(
                    station.clone(),
                    Arc::clone(&trajectory),
                    Arc::clone(&ephemeris),
                    occulting_bodies.clone(),
                    config,
                    interval,
                );
                let stage = MaterialisePass {
                    station,
                    trajectory,
                    resolution,
                };
                ItemStream::new(
                    lift(windows)
                        .then(stage)
                        .filter_ok(move |pass| long_enough(pass, min_pass_duration)),
                )
            },
            cancel,
        )
    }
}

/// One streaming inter-satellite target. The config travels with the pair
/// because the slew-rate limit is resolved per pair.
#[cfg(feature = "async")]
type InterSatelliteTarget = (
    PairId,
    Arc<Trajectory>,
    Arc<Trajectory>,
    InterSatelliteConfig,
);

/// Streams inter-satellite contact windows per spacecraft pair.
///
/// The [`VisibilityStreamAnalysis`] analogue; see there for why it owns its
/// inputs.
#[cfg(feature = "async")]
pub struct InterSatelliteStreamAnalysis<E> {
    pairs: Vec<InterSatelliteTarget>,
    ephemeris: Arc<E>,
    occulting_bodies: Vec<Origin>,
    min_duration: Option<TimeDelta>,
}

#[cfg(feature = "async")]
impl<E> InterSatelliteStreamAnalysis<E>
where
    E: Ephemeris + Send + Sync + 'static,
    E::Error: 'static,
{
    /// Snapshots a scenario into owned, shareable inputs.
    pub fn new(
        scenario: &Scenario<Origin, Frame>,
        ensemble: &Ensemble<AssetId, Origin, Frame>,
        ephemeris: Arc<E>,
        step: TimeDelta,
    ) -> Self {
        let shared = share_trajectories(ensemble);
        let spacecraft = scenario.spacecraft();
        let central_body = scenario.origin();

        let mut pairs = Vec::new();
        for i in 0..spacecraft.len() {
            for j in (i + 1)..spacecraft.len() {
                let (a, b) = (&spacecraft[i], &spacecraft[j]);
                let (Some(ta), Some(tb)) = (shared.get(a.id()), shared.get(b.id())) else {
                    continue;
                };
                // Resolved per pair, because the limit is the tighter of the two.
                let mut config = InterSatelliteConfig::new(step, central_body);
                config.slew_rate = effective_slew_rate(a, b);
                pairs.push((
                    (a.id().clone(), b.id().clone()),
                    Arc::clone(ta),
                    Arc::clone(tb),
                    config,
                ));
            }
        }

        Self {
            pairs,
            ephemeris,
            occulting_bodies: Vec::new(),
            min_duration: None,
        }
    }

    /// Adds occulting bodies beyond the central body.
    pub fn with_occulting_bodies(mut self, bodies: Vec<Origin>) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    /// Restricts contacts by range.
    pub fn with_range_limits(
        mut self,
        min_range: Option<Distance>,
        max_range: Option<Distance>,
    ) -> Self {
        for (_, _, _, config) in &mut self.pairs {
            config.min_range = min_range;
            config.max_range = max_range;
        }
        self
    }

    /// Streams every pair's contact windows, interleaved in completion order.
    pub fn stream(
        self,
        interval: TimeInterval,
        cancel: Option<lox_core::sync::CancellationToken>,
    ) -> crate::stream::AnalysisStream<PairId, Window> {
        let Self {
            pairs,
            ephemeris,
            occulting_bodies,
            min_duration,
        } = self;

        let inputs = pairs
            .into_iter()
            .map(|(key, ta, tb, config)| (key, (ta, tb, config)));

        crate::stream::stream(
            inputs,
            move |(ta, tb, mut config), cancel| {
                config.cancel = Some(cancel);
                ItemStream::new(
                    inter_satellite_stack(
                        ta,
                        tb,
                        Arc::clone(&ephemeris),
                        occulting_bodies.clone(),
                        config,
                        interval,
                    )
                    .map(|r| r.map(Window).map_err(AnalysisError::from))
                    .filter_ok(move |window| long_enough(window, min_duration)),
                )
            },
            cancel,
        )
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Origin;
    use lox_ephem::spk::parser::Spk;
    use lox_frames::Frame;
    use lox_orbits::propagators::OrbitSource;
    use lox_test_utils::data_file;
    use std::sync::OnceLock;

    #[cfg(feature = "parallel")]
    use crate::pipeline::sources::tests::iss_trajectory;
    use crate::pipeline::sources::tests::{cebreros, lunar_trajectory, scenario_and_ensemble};
    use crate::visibility::VisibilityAnalysis as EagerVisibilityAnalysis;
    use lox_time::deltas::ToDelta as _;

    use super::*;

    fn ephemeris() -> &'static Spk {
        shared_ephemeris()
    }

    /// The streaming fork needs an owned handle, so the fixture is `Arc`-shared
    /// and the borrowing paths hand out a reference into it.
    fn shared_ephemeris() -> &'static Arc<Spk> {
        static SPK: OnceLock<Arc<Spk>> = OnceLock::new();
        SPK.get_or_init(|| Arc::new(Spk::from_file(data_file("spice/de440s.bsp")).unwrap()))
    }

    type Fixture = (
        Scenario<Origin, Frame>,
        Ensemble<AssetId, Origin, Frame>,
        GroundStation,
        Vec<Spacecraft>,
        TimeInterval,
    );

    /// One station and two spacecraft, so a run has more than one target and
    /// mis-keying is observable.
    fn fixture() -> Fixture {
        let traj = lunar_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let gs = cebreros();
        let spacecraft = vec![
            Spacecraft::new("lunar-a", OrbitSource::Trajectory(traj.clone())),
            Spacecraft::new("lunar-b", OrbitSource::Trajectory(traj)),
        ];
        let stations = [gs.clone()];
        let (scenario, ensemble) = scenario_and_ensemble(&stations, &spacecraft, interval);
        (scenario, ensemble, gs, spacecraft, interval)
    }

    fn boundaries(passes: &[Pass]) -> Vec<(f64, f64)> {
        passes
            .iter()
            .map(|p| {
                let iv = p.interval();
                (
                    iv.start().to_delta().to_seconds().to_f64(),
                    iv.end().to_delta().to_seconds().to_f64(),
                )
            })
            .collect()
    }

    #[test]
    fn single_matches_the_eager_path() {
        // The Step-5 parity suite covers the sources; this covers the wiring —
        // that the analysis reaches them with the knobs it was given.
        let (scenario, ensemble, gs, spacecraft, interval) = fixture();
        let step = TimeDelta::from_seconds(60);

        let eager = EagerVisibilityAnalysis::new(&scenario, &ensemble).with_step(step);
        let eager_results = eager.compute().expect("eager visibility failed");
        let eager_passes = eager.to_passes(&eager_results);
        let eager_passes = eager_passes
            .get(&(gs.id().clone(), spacecraft[0].id().clone()))
            .expect("pair missing");

        let new = VisibilityAnalysis::new(&scenario, &ensemble, ephemeris()).with_step(step);
        let new_passes = new
            .single(&gs, &spacecraft[0], interval)
            .expect("pipeline visibility failed");

        assert!(!eager_passes.is_empty(), "fixture produced no passes");
        assert_eq!(boundaries(eager_passes), boundaries(&new_passes));
    }

    #[test]
    fn min_pass_duration_actually_discards_short_passes() {
        // The eager path only ever coarsened the scan on this knob's account; it
        // never filtered. The pipeline does both, so this is a deliberate
        // behaviour change rather than a parity failure.
        let (scenario, ensemble, gs, spacecraft, interval) = fixture();
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, ephemeris())
            .with_step(TimeDelta::from_seconds(60));

        let unfiltered = analysis
            .single(&gs, &spacecraft[0], interval)
            .expect("unfiltered failed");
        let shortest = unfiltered
            .iter()
            .map(|p| p.interval().duration().to_seconds().to_f64())
            .fold(f64::INFINITY, f64::min);

        // A threshold just above the shortest pass must drop at least that one.
        let threshold = TimeDelta::from_seconds(shortest as i64 + 1);
        let filtered = VisibilityAnalysis::new(&scenario, &ensemble, ephemeris())
            .with_step(TimeDelta::from_seconds(60))
            .with_min_pass_duration(threshold)
            .single(&gs, &spacecraft[0], interval)
            .expect("filtered failed");

        assert!(
            filtered.len() < unfiltered.len(),
            "min_pass_duration dropped nothing: {} of {} survived a {shortest:.0} s threshold",
            filtered.len(),
            unfiltered.len()
        );
        for pass in &filtered {
            assert!(pass.interval().duration() >= threshold);
        }
    }

    #[test]
    fn a_missing_trajectory_is_an_error_not_a_panic() {
        // The eager path used `.expect()` here, which took down the whole batch —
        // and under rayon, from an unhelpful thread.
        let (scenario, ensemble, gs, _, interval) = fixture();
        let orphan = Spacecraft::new(
            "never-propagated",
            OrbitSource::Trajectory(lunar_trajectory()),
        );
        let result = VisibilityAnalysis::new(&scenario, &ensemble, ephemeris())
            .single(&gs, &orphan, interval);
        let err = result.expect_err("expected a missing-trajectory error");
        assert!(err.to_string().contains("never-propagated"), "{err}");
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn sequential_and_rayon_runs_agree_and_key_correctly() {
        let (scenario, ensemble, _, spacecraft, interval) = fixture();
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, ephemeris())
            .with_step(TimeDelta::from_seconds(60));

        let collect = |parallelism| {
            let mut keyed: Vec<_> = analysis
                .run(interval, parallelism)
                .into_iter()
                .map(|(key, passes)| (key, boundaries(&passes.expect("target failed"))))
                .collect();
            // Rayon completes out of order, so compare as sets.
            keyed.sort_by(|a, b| {
                (a.0.0.as_str(), a.0.1.as_str()).cmp(&(b.0.0.as_str(), b.0.1.as_str()))
            });
            keyed
        };

        let sequential = collect(Parallelism::Sequential);
        assert_eq!(sequential.len(), spacecraft.len());
        assert!(
            sequential.iter().all(|(_, b)| !b.is_empty()),
            "fixture produced no passes"
        );
        assert_eq!(sequential, collect(Parallelism::Rayon(None)));
        assert_eq!(sequential, collect(Parallelism::Rayon(Some(3))));

        // The two spacecraft share a trajectory, so every target's passes are
        // identical and only the *keys* distinguish them. That is exactly the
        // case where keying by completion position would go unnoticed, so assert
        // the keys are the ones we asked for.
        let keys: Vec<_> = sequential
            .iter()
            .map(|(k, _)| k.1.as_str().to_string())
            .collect();
        assert_eq!(keys, vec!["lunar-a", "lunar-b"]);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn inter_satellite_run_keys_every_unordered_pair_once() {
        let traj = lunar_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let spacecraft: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|id| Spacecraft::new(*id, OrbitSource::Trajectory(traj.clone())))
            .collect();
        let (scenario, ensemble) = scenario_and_ensemble(&[], &spacecraft, interval);

        let run = InterSatelliteAnalysis::new(&scenario, &ensemble, ephemeris())
            .run(interval, Parallelism::Rayon(None));

        let mut keys: Vec<_> = run
            .iter()
            .map(|((a, b), _)| (a.as_str().to_string(), b.as_str().to_string()))
            .collect();
        keys.sort();
        assert_eq!(
            keys,
            vec![
                ("a".into(), "b".into()),
                ("a".into(), "c".into()),
                ("b".into(), "c".into())
            ]
        );
        // Colocated trajectories are always mutually visible.
        for (_, windows) in run {
            assert_eq!(windows.expect("pair failed").len(), 1);
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn power_run_reports_all_three_channels_per_spacecraft() {
        // ISS, so the eclipse channel is non-empty and the assertion has teeth.
        let traj = iss_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let spacecraft = vec![Spacecraft::new("iss", OrbitSource::Trajectory(traj))];
        let (scenario, ensemble) = scenario_and_ensemble(&[], &spacecraft, interval);

        let run = PowerBudgetAnalysis::new(&scenario, &ensemble, ephemeris())
            .with_step(TimeDelta::from_seconds(60))
            .run(interval, Parallelism::Sequential);

        assert_eq!(run.len(), spacecraft.len());
        for (id, power) in run {
            let power = power.unwrap_or_else(|e| panic!("{id} failed: {e}"));
            assert!(
                power.eclipses.len() >= 10,
                "{id}: expected >=10 eclipses over 24 h, got {}",
                power.eclipses.len()
            );
            assert_eq!(power.beta.values().len(), power.flux.values().len());
        }
    }

    #[cfg(feature = "async")]
    mod streaming {
        use futures_util::StreamExt;
        use lox_core::sync::CancellationToken;

        use crate::stream::StreamEvent;

        use super::*;

        /// Drains a stream into per-key passes plus a `Completed` tally.
        async fn drain(
            mut events: crate::stream::AnalysisStream<(AssetId, AssetId), Pass>,
        ) -> (
            std::collections::HashMap<(AssetId, AssetId), Vec<Pass>>,
            std::collections::HashMap<(AssetId, AssetId), usize>,
        ) {
            let mut passes: std::collections::HashMap<_, Vec<Pass>> = Default::default();
            let mut completed: std::collections::HashMap<_, usize> = Default::default();
            while let Some((key, event)) = events.next().await {
                match event {
                    StreamEvent::Item(item) => passes
                        .entry(key)
                        .or_default()
                        .push(item.expect("stream item failed")),
                    StreamEvent::Completed => *completed.entry(key).or_default() += 1,
                }
            }
            (passes, completed)
        }

        #[tokio::test]
        async fn streaming_matches_run_modulo_order() {
            let (scenario, ensemble, _, spacecraft, interval) = fixture();
            let step = TimeDelta::from_seconds(60);

            let expected: std::collections::HashMap<_, _> =
                VisibilityAnalysis::new(&scenario, &ensemble, ephemeris())
                    .with_step(step)
                    .run(interval, Parallelism::Sequential)
                    .into_iter()
                    .map(|(key, passes)| (key, boundaries(&passes.expect("target failed"))))
                    .collect();

            let streamed =
                VisibilityStreamAnalysis::new(&scenario, &ensemble, Arc::clone(shared_ephemeris()))
                    .with_step(step);
            let (passes, completed) = drain(streamed.stream(interval, None)).await;

            let actual: std::collections::HashMap<_, _> = passes
                .iter()
                .map(|(key, passes)| (key.clone(), boundaries(passes)))
                .collect();
            assert!(!expected.is_empty(), "fixture produced no targets");
            assert_eq!(actual, expected);

            // Exactly one `Completed` per target, for every target.
            assert_eq!(completed.len(), spacecraft.len());
            assert!(completed.values().all(|&c| c == 1), "{completed:?}");
        }

        #[tokio::test]
        async fn a_pre_cancelled_token_yields_nothing_at_all() {
            // Not even `Completed`: a target cancelled before its first item is
            // never observed, which is what lets the engine skip `Started`
            // events entirely.
            let (scenario, ensemble, _, _, interval) = fixture();
            let cancel = CancellationToken::new();
            cancel.cancel();

            let streamed =
                VisibilityStreamAnalysis::new(&scenario, &ensemble, Arc::clone(shared_ephemeris()));
            let (passes, completed) = drain(streamed.stream(interval, Some(cancel))).await;
            assert!(passes.is_empty(), "cancelled run produced items");
            assert!(completed.is_empty(), "cancelled run reported Completed");
        }

        #[tokio::test]
        async fn inter_satellite_streaming_matches_its_run() {
            let traj = lunar_trajectory();
            let interval = TimeInterval::new(traj.start_time(), traj.end_time());
            let spacecraft: Vec<_> = ["a", "b"]
                .iter()
                .map(|id| Spacecraft::new(*id, OrbitSource::Trajectory(traj.clone())))
                .collect();
            let (scenario, ensemble) = scenario_and_ensemble(&[], &spacecraft, interval);
            let step = TimeDelta::from_seconds(60);

            let expected: Vec<_> = InterSatelliteAnalysis::new(&scenario, &ensemble, ephemeris())
                .with_step(step)
                .run(interval, Parallelism::Sequential)
                .into_iter()
                .map(|(key, windows)| (key, windows.expect("target failed").len()))
                .collect();

            let mut events = InterSatelliteStreamAnalysis::new(
                &scenario,
                &ensemble,
                Arc::clone(shared_ephemeris()),
                step,
            )
            .stream(interval, None);

            let mut windows: std::collections::HashMap<_, usize> = Default::default();
            let mut completed = 0;
            while let Some((key, event)) = events.next().await {
                match event {
                    StreamEvent::Item(item) => {
                        item.expect("stream item failed");
                        *windows.entry(key).or_default() += 1;
                    }
                    StreamEvent::Completed => completed += 1,
                }
            }

            assert_eq!(expected.len(), 1);
            assert_eq!(completed, 1);
            for (key, count) in expected {
                assert_eq!(windows.get(&key), Some(&count));
            }
        }
    }
}
