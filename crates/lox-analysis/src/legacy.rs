// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! The eager `*Analysis` implementations, staged for deletion.
//!
//! These are the pre-pipeline `compute()`-and-aggregate-`*Results` types, moved
//! here verbatim so that every commit in the hard cut builds: the canonical
//! paths already point at the pipeline-backed replacements, and the Python
//! bindings import from here until they are ported.
//!
//! **Nothing new should be added to this module, and nothing should start
//! depending on it.** It is deleted in the commit that finishes the Python
//! surface (plan step 8).

#![allow(missing_docs)]

/// The eager access analysis.
#[cfg(feature = "imaging")]
pub mod imaging;

use std::collections::HashMap;

use lox_bodies::{CoordinateOrigin, Origin, Sun, TryMeanRadius, TrySpheroid};
use lox_core::math::series::{InterpolationType, SeriesError};
use lox_core::units::Distance;
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::ground::GroundLocation;
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::series::TimeSeries;
use lox_time::time_scales::{Tdb, TimeScale};
use thiserror::Error;

use crate::assets::{AssetId, ConstellationId, GroundStation, Scenario, Spacecraft};
use crate::events::{
    AdaptiveSampler, DetectError, DetectFnExt as _, IntervalIterExt as _, UniformSampler,
};
use crate::par::try_map;
use crate::power::{EclipseDetectFn, beta_angle, solar_flux};
use crate::visibility::{
    ElevationDetectFn, ElevationMask, EvalError, InterSatLosCentralBodyDetectFn,
    InterSatLosOccluderDetectFn, InterSatelliteRangeDetectFn, InterSatelliteSlewRateDetectFn,
    LineOfSightDetectFn, Pass, RangeDirection,
};

// ===========================================================================
// Visibility
// ===========================================================================

/// Errors from visibility interval computation.
#[derive(Debug, Error)]
pub enum VisibilityError {
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] DetectError),
    /// Series interpolation failed.
    #[error(transparent)]
    Series(#[from] SeriesError),
}

/// Error returned when computing passes for an invalid pair type.
#[derive(Debug, Error)]
pub enum PassError {
    #[error(
        "passes are not supported for inter-satellite pair ({0}, {1}): use intervals() instead"
    )]
    /// Passes are not supported for inter-satellite pairs; use intervals instead.
    InterSatellitePair(String, String),
}

/// Distinguishes ground-to-space from inter-satellite visibility pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairType {
    /// Ground station to spacecraft pair.
    GroundSpace,
    /// Spacecraft to spacecraft pair.
    InterSatellite,
}

type IntervalMap = HashMap<(AssetId, AssetId), Vec<TimeInterval>>;
type PairTypeMap = HashMap<(AssetId, AssetId), PairType>;
type GroundSpaceFilter<'a> = Box<dyn Fn(&GroundStation, &Spacecraft) -> bool + 'a>;
type InterSatelliteFilter<'a> = Box<dyn Fn(&Spacecraft, &Spacecraft) -> bool + 'a>;

/// Parameters shared by the per-pair compute functions, extracted from
/// `VisibilityAnalysis` so that they can be passed into the parallel section
/// without borrowing the non-`Send` filter closures.
struct ComputeParams<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: &'a E,
    occulting_bodies: &'a [Origin],
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
    adaptive: bool,
}

impl<O, R, E> ComputeParams<'_, O, R, E>
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
    /// The scan step, widened when `min_pass_duration` allows a coarser sweep.
    ///
    /// A pass shorter than the step can be missed, so half the minimum pass
    /// duration is the coarsest safe stride.
    fn scan_step(&self) -> TimeDelta {
        match self.min_pass_duration {
            Some(d) if 0.5 * d > self.step => 0.5 * d,
            _ => self.step,
        }
    }

    /// Compute visibility intervals for a single (ground, space) pair.
    fn compute_ground_space_pair(
        &self,
        gs: &GroundStation,
        sc_traj: &Trajectory<O, R>,
        interval: TimeInterval,
    ) -> Result<Vec<TimeInterval>, VisibilityError> {
        let body_fixed_frame = gs.body_fixed_frame();
        let step = self.scan_step();

        let elev = ElevationDetectFn {
            gs: gs.location().clone(),
            mask: gs.mask().clone(),
            sc: sc_traj,
            body_fixed_frame,
        };

        // Elevation is the cheap constraint and always runs over the whole
        // window; `adaptive` lets it stride by its own rate bound.
        let elev_windows: Box<dyn Iterator<Item = Result<TimeInterval, DetectError>> + '_> =
            if self.adaptive {
                Box::new(elev.into_intervals(
                    AdaptiveSampler::new(step, interval.duration().max(step)),
                    interval,
                ))
            } else {
                Box::new(elev.into_intervals(UniformSampler::new(step), interval))
            };

        if self.occulting_bodies.is_empty() {
            return Ok(elev_windows.collect::<Result<_, _>>()?);
        }

        // Line of sight needs an ephemeris lookup per sample, so it runs only
        // inside the windows elevation already admitted.
        let occulters = self.occulting_bodies;
        let ephemeris = self.ephemeris;
        let location = gs.location();
        let windows = elev_windows.then_within(move |window| {
            let make_los = |body: Origin| {
                LineOfSightDetectFn {
                    gs: location.clone(),
                    sc: sc_traj,
                    body,
                    ephemeris,
                    body_fixed_frame,
                }
                .into_intervals(UniformSampler::new(step), window)
            };
            let mut los: Box<dyn Iterator<Item = Result<TimeInterval, DetectError>> + '_> =
                Box::new(make_los(occulters[0]));
            for &body in &occulters[1..] {
                los = Box::new(los.intersect(make_los(body)));
            }
            los
        });

        Ok(windows.collect::<Result<_, _>>()?)
    }

    /// Compute LOS intervals for a single inter-satellite pair,
    /// optionally filtered by min/max range constraints.
    ///
    /// The scenario's central body is always checked for occultation.
    /// Any additional occulting bodies are checked as well.
    fn compute_inter_satellite_pair(
        &self,
        sc1: &Spacecraft,
        sc2: &Spacecraft,
        traj1: &Trajectory<O, R>,
        traj2: &Trajectory<O, R>,
        interval: TimeInterval,
    ) -> Result<Vec<TimeInterval>, VisibilityError> {
        // Resolve per-pair slew rate limit: min of both assets' limits.
        let effective_slew_rate = match (sc1.max_slew_rate(), sc2.max_slew_rate()) {
            (Some(a), Some(b)) => Some(if a.to_radians_per_second() < b.to_radians_per_second() {
                a
            } else {
                b
            }),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let step = self.scan_step();
        let ephemeris = self.ephemeris;
        let central_body: Origin = self.scenario.origin().into();

        let make_range =
            move |threshold: Distance, direction: RangeDirection, window: TimeInterval| {
                InterSatelliteRangeDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    threshold,
                    direction,
                }
                .into_intervals(UniformSampler::new(step), window)
            };

        // The stages run cheapest-first, each scanning only inside the windows
        // the previous one admitted. Seeding with the whole interval lets every
        // stage be applied uniformly, whether or not range limits are set.
        let mut windows: Box<dyn Iterator<Item = Result<TimeInterval, DetectError>> + '_> =
            match (self.max_range, self.min_range) {
                (Some(max), Some(min)) => Box::new(
                    make_range(max, RangeDirection::Max, interval).intersect(make_range(
                        min,
                        RangeDirection::Min,
                        interval,
                    )),
                ),
                (Some(max), None) => Box::new(make_range(max, RangeDirection::Max, interval)),
                (None, Some(min)) => Box::new(make_range(min, RangeDirection::Min, interval)),
                (None, None) => Box::new(std::iter::once(Ok(interval))),
            };

        // Slew rate: position and velocity, no ephemeris.
        if let Some(threshold) = effective_slew_rate {
            windows = Box::new(windows.then_within(move |window| {
                InterSatelliteSlewRateDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    threshold,
                }
                .into_intervals(UniformSampler::new(step), window)
            }));
        }

        // Central-body occultation always applies, and needs no ephemeris.
        windows = Box::new(windows.then_within(move |window| {
            InterSatLosCentralBodyDetectFn {
                sc1: traj1,
                sc2: traj2,
                body: central_body,
            }
            .into_intervals(UniformSampler::new(step), window)
        }));

        // Additional occulters are the most expensive: ephemeris per sample.
        for &body in self.occulting_bodies {
            windows = Box::new(windows.then_within(move |window| {
                InterSatLosOccluderDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    body,
                    ephemeris,
                }
                .into_intervals(UniformSampler::new(step), window)
            }));
        }

        Ok(windows.collect::<Result<_, _>>()?)
    }
}

/// Stores raw visibility intervals per asset pair.
///
/// This is the primary result type for visibility analysis. Intervals are
/// cheap to compute; conversion to [`Pass`] (with observables) happens
/// separately and on demand.
pub struct VisibilityResults {
    intervals: IntervalMap,
    pair_types: PairTypeMap,
}

impl VisibilityResults {
    /// Return all intervals for a specific pair.
    pub fn intervals_for(&self, id1: &AssetId, id2: &AssetId) -> Option<&[TimeInterval]> {
        let key = (id1.clone(), id2.clone());
        self.intervals.get(&key).map(|v| v.as_slice())
    }

    /// Return all intervals keyed by pair ids.
    pub fn all_intervals(&self) -> &IntervalMap {
        &self.intervals
    }

    /// Iterate over all pair keys.
    pub fn pair_ids(&self) -> impl Iterator<Item = &(AssetId, AssetId)> {
        self.intervals.keys()
    }

    /// Return the [`PairType`] for a given pair, if present.
    pub fn pair_type(&self, id1: &AssetId, id2: &AssetId) -> Option<PairType> {
        self.pair_types.get(&(id1.clone(), id2.clone())).copied()
    }

    /// Return pair ids for ground-to-space pairs only.
    pub fn ground_space_pair_ids(&self) -> Vec<&(AssetId, AssetId)> {
        self.pair_types
            .iter()
            .filter(|&(_, &pt)| pt == PairType::GroundSpace)
            .map(|(k, _)| k)
            .collect()
    }

    /// Return pair ids for inter-satellite pairs only.
    pub fn inter_satellite_pair_ids(&self) -> Vec<&(AssetId, AssetId)> {
        self.pair_types
            .iter()
            .filter(|&(_, &pt)| pt == PairType::InterSatellite)
            .map(|(k, _)| k)
            .collect()
    }

    /// Returns `true` if no visibility intervals were found.
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Returns the number of asset pairs with visibility data.
    pub fn num_pairs(&self) -> usize {
        self.intervals.len()
    }

    /// Total number of visibility intervals across all pairs.
    pub fn total_intervals(&self) -> usize {
        self.intervals.values().map(|v| v.len()).sum()
    }

    /// Consume self and return the inner intervals and pair types maps.
    pub fn into_parts(self) -> (IntervalMap, PairTypeMap) {
        (self.intervals, self.pair_types)
    }

    /// Convert intervals for a specific ground-space pair to visibility passes.
    ///
    /// Returns an error if the pair is an inter-satellite pair, since passes
    /// with ground-station observables are not meaningful for such pairs.
    /// Returns an empty vec if the pair is not found.
    #[allow(clippy::too_many_arguments)]
    pub fn to_passes(
        &self,
        ground_id: &AssetId,
        space_id: &AssetId,
        gs: &GroundLocation,
        mask: &ElevationMask,
        sc: &lox_orbits::orbits::Trajectory,
        time_resolution: TimeDelta,
        body_fixed_frame: Frame,
    ) -> Result<Vec<Pass>, PassError> {
        let key = (ground_id.clone(), space_id.clone());
        if self.pair_types.get(&key) == Some(&PairType::InterSatellite) {
            return Err(PassError::InterSatellitePair(
                ground_id.as_str().to_string(),
                space_id.as_str().to_string(),
            ));
        }
        Ok(self
            .intervals
            .get(&key)
            .map(|intervals| {
                intervals
                    .iter()
                    .filter_map(|interval| {
                        let dynamic_interval = TimeInterval::new(
                            interval.start().into_dynamic(),
                            interval.end().into_dynamic(),
                        );
                        Pass::from_interval(
                            dynamic_interval,
                            time_resolution,
                            gs,
                            mask,
                            sc,
                            body_fixed_frame,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// VisibilityAnalysis
// ---------------------------------------------------------------------------

/// Marker type for a [`VisibilityAnalysis`] that has not been bound to an
/// [`Ephemeris`]. The default for the `E` type parameter.
///
/// Building an analysis without an ephemeris is the right choice when no
/// extra occulting bodies are configured; in that case the ephemeris would
/// never be consulted. Use [`VisibilityAnalysis::with_occulting_bodies`]
/// to supply an ephemeris when needed.
#[derive(Default, Clone, Copy, Debug)]
pub struct NoEphemeris;

/// Computes ground-station-to-spacecraft and inter-satellite visibility.
///
/// Generic over origin `O`, reference frame `R`, and ephemeris `E`.
/// Ground-to-space pairs are always computed when ground assets are present.
/// Inter-satellite pairs are additionally computed when enabled via
/// [`with_inter_satellite`](Self::with_inter_satellite).
///
/// Trajectories are looked up from a pre-computed [`Ensemble`] by asset id.
pub struct VisibilityAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame, E = NoEphemeris> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: E,
    occulting_bodies: Vec<Origin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    inter_satellite: bool,
    ground_space_filter: Option<GroundSpaceFilter<'a>>,
    inter_satellite_filter: Option<InterSatelliteFilter<'a>>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
    adaptive: bool,
}

// ---------------------------------------------------------------------------
// Block A — generic builder methods (no ephemeris bound, shared across both
// variants). Also includes `to_passes` since it never consults the ephemeris.
// ---------------------------------------------------------------------------

impl<'a, O, R, E> VisibilityAnalysis<'a, O, R, E>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Enables inter-satellite visibility computation.
    pub fn with_inter_satellite(mut self) -> Self {
        self.inter_satellite = true;
        self
    }

    /// Sets a pre-filter for ground-to-space pairs.
    ///
    /// The filter is called once per candidate pair during pair enumeration,
    /// before the parallel computation phase. Only pairs for which the filter
    /// returns `true` are evaluated.
    pub fn with_ground_space_filter(
        mut self,
        filter: impl Fn(&GroundStation, &Spacecraft) -> bool + 'a,
    ) -> Self {
        self.ground_space_filter = Some(Box::new(filter));
        self
    }

    /// Enables inter-satellite visibility with a pre-filter.
    ///
    /// The filter is called once per candidate pair during pair enumeration,
    /// before the parallel computation phase. Only pairs for which the filter
    /// returns `true` are evaluated.
    pub fn with_inter_satellite_filter(
        mut self,
        filter: impl Fn(&Spacecraft, &Spacecraft) -> bool + 'a,
    ) -> Self {
        self.inter_satellite = true;
        self.inter_satellite_filter = Some(Box::new(filter));
        self
    }

    /// Sets the time step for event detection sampling.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Sets the minimum pass duration; shorter passes will be discarded.
    pub fn with_min_pass_duration(mut self, min_pass_duration: TimeDelta) -> Self {
        self.min_pass_duration = Some(min_pass_duration);
        self
    }

    /// Sets the minimum range filter for inter-satellite links.
    pub fn with_min_range(mut self, min_range: Distance) -> Self {
        self.min_range = Some(min_range);
        self
    }

    /// Sets the maximum range filter for inter-satellite links.
    pub fn with_max_range(mut self, max_range: Distance) -> Self {
        self.max_range = Some(max_range);
        self
    }

    /// Drives the scan with the detect function's rate bound instead of a
    /// fixed step, taking large strides far from a crossing. The configured
    /// step acts as the minimum stride.
    pub fn with_adaptive_detection(mut self) -> Self {
        self.adaptive = true;
        self
    }

    /// Returns the current time step.
    pub fn step(&self) -> TimeDelta {
        self.step
    }
}

impl<'a, O, R, E> VisibilityAnalysis<'a, O, R, E>
where
    O: CoordinateOrigin + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<Frame>,
{
    /// Convert all ground-space intervals in a [`VisibilityResults`] to passes.
    ///
    /// Inter-satellite pairs are skipped since passes with ground-station
    /// observables are not meaningful for them.
    pub fn to_passes(&self, results: &VisibilityResults) -> HashMap<(AssetId, AssetId), Vec<Pass>> {
        let gs_map: HashMap<&AssetId, &GroundStation> = self
            .scenario
            .ground_stations()
            .iter()
            .map(|g| (g.id(), g))
            .collect();

        results
            .ground_space_pair_ids()
            .into_iter()
            .filter_map(|(gs_id, sc_id)| {
                let gs = gs_map.get(gs_id)?;
                let sc_traj = self.ensemble.get(sc_id)?;
                let intervals = results.intervals_for(gs_id, sc_id)?;
                let passes: Vec<Pass> = intervals
                    .iter()
                    .filter_map(|interval| {
                        // The trajectory may carry typed origin/frame; erase them
                        // for pass computation.
                        let dynamic_traj = sc_traj.clone().into_dynamic();
                        Pass::from_interval(
                            *interval,
                            self.step,
                            gs.location(),
                            gs.mask(),
                            &dynamic_traj,
                            gs.body_fixed_frame(),
                        )
                    })
                    .collect();
                Some(((gs_id.clone(), sc_id.clone()), passes))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Block B — `NoEphemeris` constructor and transition to the ephemeris variant.
// ---------------------------------------------------------------------------

impl<'a, O, R> VisibilityAnalysis<'a, O, R, NoEphemeris>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Creates a new visibility analysis without an ephemeris.
    ///
    /// Use [`with_occulting_bodies`](Self::with_occulting_bodies) to bind
    /// an ephemeris when occulting-body checks are required.
    pub fn new(scenario: &'a Scenario<O, R>, ensemble: &'a Ensemble<AssetId, O, R>) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris: NoEphemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_pass_duration: None,
            inter_satellite: false,
            ground_space_filter: None,
            inter_satellite_filter: None,
            min_range: None,
            max_range: None,
            adaptive: false,
        }
    }

    /// Binds an ephemeris and configures additional occulting bodies.
    ///
    /// For inter-satellite visibility, the scenario's central body is
    /// always checked for occultation automatically (using an
    /// ephemeris-free path). Use this method to add extra occulting
    /// bodies (e.g. the Moon for an Earth-centred scenario).
    pub fn with_occulting_bodies<E>(
        self,
        ephemeris: &'a E,
        bodies: Vec<Origin>,
    ) -> VisibilityAnalysis<'a, O, R, &'a E>
    where
        E: Ephemeris,
    {
        VisibilityAnalysis {
            scenario: self.scenario,
            ensemble: self.ensemble,
            ephemeris,
            occulting_bodies: bodies,
            step: self.step,
            min_pass_duration: self.min_pass_duration,
            inter_satellite: self.inter_satellite,
            ground_space_filter: self.ground_space_filter,
            inter_satellite_filter: self.inter_satellite_filter,
            min_range: self.min_range,
            max_range: self.max_range,
            adaptive: self.adaptive,
        }
    }
}

// ---------------------------------------------------------------------------
// Block C — `compute()` on the `NoEphemeris` variant.
// No ephemeris is consulted. The central body is checked via the
// ephemeris-free `InterSatLosCentralBodyDetectFn`.
// ---------------------------------------------------------------------------

impl<'a, O, R> VisibilityAnalysis<'a, O, R, NoEphemeris>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<Frame>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale> + TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Compute visibility intervals for all pairs without an ephemeris.
    pub fn compute(&self) -> Result<VisibilityResults, VisibilityError> {
        debug_assert!(self.occulting_bodies.is_empty());
        let interval = *self.scenario.interval();

        let mut intervals = HashMap::new();
        let mut pair_types = HashMap::new();

        if !self.scenario.ground_stations().is_empty() {
            let gs_results = self.compute_ground_space_no_eph(interval.into_dynamic())?;
            let (gs_intervals, gs_pair_types) = gs_results.into_parts();
            intervals.extend(gs_intervals);
            pair_types.extend(gs_pair_types);
        }
        if self.inter_satellite {
            let is_results = self.compute_inter_satellite_no_eph(interval.into_dynamic())?;
            let (is_intervals, is_pair_types) = is_results.into_parts();
            intervals.extend(is_intervals);
            pair_types.extend(is_pair_types);
        }
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute ground-to-space visibility without occulting-body checks.
    fn compute_ground_space_no_eph(
        &self,
        interval: TimeInterval,
    ) -> Result<VisibilityResults, VisibilityError> {
        let ground_stations = self.scenario.ground_stations();
        let spacecraft = self.scenario.spacecraft();
        let step = self.step;
        let min_pass_duration = self.min_pass_duration;

        let pairs: Vec<_> = ground_stations
            .iter()
            .flat_map(|gs| spacecraft.iter().map(move |sc| (gs, sc)))
            .filter(|(gs, sc)| self.ground_space_filter.as_ref().is_none_or(|f| f(gs, sc)))
            .collect();

        // Extract references needed in the parallel closure without borrowing self.
        let ensemble = self.ensemble;
        let adaptive = self.adaptive;

        let compute_one = |(gs, sc): &(&GroundStation, &Spacecraft)| {
            let key = (gs.id().clone(), sc.id().clone());
            let sc_traj = ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let body_fixed_frame = gs.body_fixed_frame();
            let step = match min_pass_duration {
                Some(d) if 0.5 * d > step => 0.5 * d,
                _ => step,
            };
            let elev = ElevationDetectFn {
                gs: gs.location().clone(),
                mask: gs.mask().clone(),
                sc: sc_traj,
                body_fixed_frame,
            };
            let windows = if adaptive {
                elev.intervals(
                    AdaptiveSampler::new(step, interval.duration().max(step)),
                    interval,
                )?
            } else {
                elev.intervals(UniformSampler::new(step), interval)?
            };
            Ok((key, windows))
        };

        const PARALLEL_THRESHOLD: usize = 100;

        let results: Result<Vec<_>, VisibilityError> =
            try_map(&pairs, pairs.len() > PARALLEL_THRESHOLD, compute_one);

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::GroundSpace))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute inter-satellite visibility against the central body only.
    fn compute_inter_satellite_no_eph(
        &self,
        interval: TimeInterval,
    ) -> Result<VisibilityResults, VisibilityError> {
        let spacecraft = self.scenario.spacecraft();
        let n = spacecraft.len();

        let mut pairs: Vec<(usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                let accepted = self
                    .inter_satellite_filter
                    .as_ref()
                    .is_none_or(|f| f(&spacecraft[i], &spacecraft[j]));
                if accepted {
                    pairs.push((i, j));
                }
            }
        }

        let step = self.step;
        let min_pass_duration = self.min_pass_duration;
        let central_body: Origin = self.scenario.origin().into();
        let min_range = self.min_range;
        let max_range = self.max_range;
        let ensemble = self.ensemble;

        let step = match min_pass_duration {
            Some(d) if 0.5 * d > step => 0.5 * d,
            _ => step,
        };

        let results: Result<Vec<_>, VisibilityError> = try_map(&pairs, true, |&(i, j)| {
            let sc1 = &spacecraft[i];
            let sc2 = &spacecraft[j];
            let key = (sc1.id().clone(), sc2.id().clone());
            let traj1 = ensemble
                .get(sc1.id())
                .expect("trajectory not found in ensemble");
            let traj2 = ensemble
                .get(sc2.id())
                .expect("trajectory not found in ensemble");

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

            let make_range =
                move |threshold: Distance, direction: RangeDirection, window: TimeInterval| {
                    InterSatelliteRangeDetectFn {
                        sc1: traj1,
                        sc2: traj2,
                        threshold,
                        direction,
                    }
                    .into_intervals(UniformSampler::new(step), window)
                };

            // Cheapest-first staging, seeded with the whole interval so
            // every later stage applies uniformly.
            let mut windows: Box<dyn Iterator<Item = Result<TimeInterval, DetectError>> + '_> =
                match (max_range, min_range) {
                    (Some(max), Some(min)) => Box::new(
                        make_range(max, RangeDirection::Max, interval).intersect(make_range(
                            min,
                            RangeDirection::Min,
                            interval,
                        )),
                    ),
                    (Some(max), None) => Box::new(make_range(max, RangeDirection::Max, interval)),
                    (None, Some(min)) => Box::new(make_range(min, RangeDirection::Min, interval)),
                    (None, None) => Box::new(std::iter::once(Ok(interval))),
                };

            if let Some(threshold) = effective_slew_rate {
                windows = Box::new(windows.then_within(move |window| {
                    InterSatelliteSlewRateDetectFn {
                        sc1: traj1,
                        sc2: traj2,
                        threshold,
                    }
                    .into_intervals(UniformSampler::new(step), window)
                }));
            }

            windows = Box::new(windows.then_within(move |window| {
                InterSatLosCentralBodyDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    body: central_body,
                }
                .into_intervals(UniformSampler::new(step), window)
            }));

            let windows = windows.collect::<Result<_, _>>()?;
            Ok((key, windows))
        });

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::InterSatellite))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }
}

// ---------------------------------------------------------------------------
// Block D — `compute()` for the with-ephemeris variant.
// Methods moved from the old big impl block unchanged.
// ---------------------------------------------------------------------------

impl<'a, O, R, E> VisibilityAnalysis<'a, O, R, &'a E>
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
    /// Compute visibility intervals for all pairs.
    pub fn compute(&self) -> Result<VisibilityResults, VisibilityError> {
        let interval = *self.scenario.interval();

        let mut intervals = HashMap::new();
        let mut pair_types = HashMap::new();

        if !self.scenario.ground_stations().is_empty() {
            let gs_results = self.compute_ground_space(interval.into_dynamic())?;
            let (gs_intervals, gs_pair_types) = gs_results.into_parts();
            intervals.extend(gs_intervals);
            pair_types.extend(gs_pair_types);
        }

        if self.inter_satellite {
            let is_results = self.compute_inter_satellite(interval.into_dynamic())?;
            let (is_intervals, is_pair_types) = is_results.into_parts();
            intervals.extend(is_intervals);
            pair_types.extend(is_pair_types);
        }

        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute ground-to-space visibility for all (ground, space) pairs.
    fn compute_ground_space(
        &self,
        interval: TimeInterval,
    ) -> Result<VisibilityResults, VisibilityError> {
        let ground_stations = self.scenario.ground_stations();
        let spacecraft = self.scenario.spacecraft();

        // Pre-filter while we still have access to `self` (and the filter).
        let pairs: Vec<_> = ground_stations
            .iter()
            .flat_map(|gs| spacecraft.iter().map(move |sc| (gs, sc)))
            .filter(|(gs, sc)| self.ground_space_filter.as_ref().is_none_or(|f| f(gs, sc)))
            .collect();

        // Extract Send+Sync fields into a params struct, avoiding a shared
        // borrow of `self` (which contains the non-Send filter closures).
        let params = ComputeParams {
            scenario: self.scenario,
            ensemble: self.ensemble,
            ephemeris: self.ephemeris,
            occulting_bodies: &self.occulting_bodies,
            step: self.step,
            min_pass_duration: self.min_pass_duration,
            min_range: self.min_range,
            max_range: self.max_range,
            adaptive: self.adaptive,
        };

        const PARALLEL_THRESHOLD: usize = 100;
        let use_parallel = pairs.len() > PARALLEL_THRESHOLD;

        let compute_one = |(gs, sc): &(&GroundStation, &Spacecraft)| {
            let key = (gs.id().clone(), sc.id().clone());
            let sc_traj = params.ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let windows = params.compute_ground_space_pair(gs, sc_traj, interval)?;
            Ok((key, windows))
        };

        let results: Result<Vec<_>, VisibilityError> = try_map(&pairs, use_parallel, compute_one);

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::GroundSpace))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute LOS visibility for all unique spacecraft pairs (i, j) where i < j.
    fn compute_inter_satellite(
        &self,
        interval: TimeInterval,
    ) -> Result<VisibilityResults, VisibilityError> {
        let spacecraft = self.scenario.spacecraft();
        let n = spacecraft.len();

        // Pre-filter while we still have access to `self` (and the filter).
        let mut pairs: Vec<(usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                let accepted = self
                    .inter_satellite_filter
                    .as_ref()
                    .is_none_or(|f| f(&spacecraft[i], &spacecraft[j]));
                if accepted {
                    pairs.push((i, j));
                }
            }
        }

        // Extract Send+Sync fields into a params struct for the parallel section.
        let params = ComputeParams {
            scenario: self.scenario,
            ensemble: self.ensemble,
            ephemeris: self.ephemeris,
            occulting_bodies: &self.occulting_bodies,
            step: self.step,
            min_pass_duration: self.min_pass_duration,
            min_range: self.min_range,
            max_range: self.max_range,
            adaptive: self.adaptive,
        };

        let results: Result<Vec<_>, VisibilityError> = try_map(&pairs, true, |&(i, j)| {
            let sc1 = &spacecraft[i];
            let sc2 = &spacecraft[j];
            let key = (sc1.id().clone(), sc2.id().clone());
            let traj1 = params
                .ensemble
                .get(sc1.id())
                .expect("trajectory not found in ensemble");
            let traj2 = params
                .ensemble
                .get(sc2.id())
                .expect("trajectory not found in ensemble");
            let windows = params.compute_inter_satellite_pair(sc1, sc2, traj1, traj2, interval)?;
            Ok((key, windows))
        });

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::InterSatellite))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// ===========================================================================
// Power
// ===========================================================================

/// Per-spacecraft power-budget output tuple.
type SpacecraftPowerData = (AssetId, Vec<TimeInterval>, TimeSeries, TimeSeries);

/// Errors from power-budget analysis.
#[derive(Debug, Error)]
pub enum PowerError {
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] crate::events::DetectError),
    /// Evaluation error (frame rotation, ephemeris, …).
    #[error(transparent)]
    Eval(#[from] EvalError),
}

// ---------------------------------------------------------------------------
// PowerBudgetResults
// ---------------------------------------------------------------------------

/// Results of a power-budget analysis.
///
/// Contains eclipse intervals, beta-angle time series, and solar-flux time
/// series for each spacecraft.
pub struct PowerBudgetResults {
    eclipse_intervals: HashMap<AssetId, Vec<TimeInterval>>,
    beta_angles: HashMap<AssetId, TimeSeries>,
    solar_fluxes: HashMap<AssetId, TimeSeries>,
    scenario_duration: f64,
}

impl PowerBudgetResults {
    /// Eclipse intervals for a given spacecraft.
    pub fn eclipse_intervals_for(&self, id: &AssetId) -> Option<&[TimeInterval]> {
        self.eclipse_intervals.get(id).map(|v| v.as_slice())
    }

    /// All eclipse intervals keyed by spacecraft id.
    pub fn all_eclipse_intervals(&self) -> &HashMap<AssetId, Vec<TimeInterval>> {
        &self.eclipse_intervals
    }

    /// Eclipse fraction for a given spacecraft (ratio of total eclipse time to
    /// scenario duration, in \[0, 1\]).
    pub fn eclipse_fraction(&self, id: &AssetId) -> Option<f64> {
        let intervals = self.eclipse_intervals.get(id)?;
        let total_eclipse: f64 = intervals
            .iter()
            .map(|i| (i.end() - i.start()).to_seconds().to_f64())
            .sum();
        Some(total_eclipse / self.scenario_duration)
    }

    /// Sunlit fraction for a given spacecraft (`1 − eclipse_fraction`).
    pub fn sunlit_fraction(&self, id: &AssetId) -> Option<f64> {
        self.eclipse_fraction(id).map(|f| 1.0 - f)
    }

    /// Beta-angle time series for a given spacecraft (radians).
    pub fn beta_angles_for(&self, id: &AssetId) -> Option<&TimeSeries> {
        self.beta_angles.get(id)
    }

    /// Solar-flux time series for a given spacecraft (W/m²).
    pub fn solar_flux_for(&self, id: &AssetId) -> Option<&TimeSeries> {
        self.solar_fluxes.get(id)
    }
}

// ---------------------------------------------------------------------------
// PowerBudgetAnalysis
// ---------------------------------------------------------------------------

/// Filter for restricting which spacecraft are analysed.
#[derive(Clone)]
pub enum SpacecraftFilter {
    /// Analyse only spacecraft whose id is in the given list.
    Ids(Vec<AssetId>),
    /// Analyse only spacecraft belonging to the given constellation.
    Constellation(ConstellationId),
}

/// Computes eclipse intervals, beta angles, and solar flux for spacecraft
/// in a scenario.
///
/// Generic over origin `O`, reference frame `R`, and ephemeris `E`.
/// The shadow model is cylindrical (umbra only) — penumbra is not modelled.
pub struct PowerBudgetAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    ephemeris: &'a E,
    step: TimeDelta,
    filter: Option<SpacecraftFilter>,
}

impl<'a, O, R, E> PowerBudgetAnalysis<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates a new power-budget analysis.
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
            filter: None,
        }
    }

    /// Sets the time step for sampling and event detection.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Restricts the analysis to a subset of spacecraft.
    ///
    /// See [`SpacecraftFilter`] for the available filter modes.
    pub fn with_filter(mut self, filter: SpacecraftFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Compute the power-budget analysis for all (or filtered) spacecraft in
    /// the scenario.
    pub fn compute(&self) -> Result<PowerBudgetResults, PowerError> {
        let interval = *self.scenario.interval();
        let all_spacecraft = self.scenario.spacecraft();
        let spacecraft: Vec<&Spacecraft> = match &self.filter {
            Some(SpacecraftFilter::Ids(ids)) => all_spacecraft
                .iter()
                .filter(|sc| ids.contains(sc.id()))
                .collect(),
            Some(SpacecraftFilter::Constellation(cid)) => all_spacecraft
                .iter()
                .filter(|sc| sc.constellation_id() == Some(cid))
                .collect(),
            None => all_spacecraft.iter().collect(),
        };
        let duration_s = (interval.end() - interval.start()).to_seconds().to_f64();

        let results: Result<Vec<_>, PowerError> = try_map(&spacecraft, true, |sc| {
            self.compute_spacecraft(sc, interval.into_dynamic())
        });

        let mut eclipse_intervals = HashMap::new();
        let mut beta_angles = HashMap::new();
        let mut solar_fluxes = HashMap::new();

        for (id, eclipses, betas, fluxes) in results? {
            eclipse_intervals.insert(id.clone(), eclipses);
            beta_angles.insert(id.clone(), betas);
            solar_fluxes.insert(id, fluxes);
        }

        Ok(PowerBudgetResults {
            eclipse_intervals,
            beta_angles,
            solar_fluxes,
            scenario_duration: duration_s,
        })
    }

    /// Compute all quantities for a single spacecraft.
    fn compute_spacecraft(
        &self,
        sc: &Spacecraft,
        interval: TimeInterval,
    ) -> Result<SpacecraftPowerData, PowerError> {
        let sc_traj = self.ensemble.get(sc.id()).expect(
            "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
        );

        // 1. Eclipse intervals via root-finding
        let eclipse_fn = EclipseDetectFn {
            sc: sc_traj,
            ephemeris: self.ephemeris,
        };
        // The scan yields intervals where the function is positive (sunlit);
        // complementing them within the scan window gives the eclipses.
        let eclipse_intervals: Vec<TimeInterval> = eclipse_fn
            .iter_intervals(UniformSampler::new(self.step), interval)
            .complement(interval)
            .collect::<Result<_, _>>()?;

        // 2. Beta angle + solar flux sampled at `step`
        let epoch = interval.start();
        let mut offsets = Vec::new();
        let mut beta_values = Vec::new();
        let mut flux_values = Vec::new();

        for time in interval.step_by(self.step) {
            let tdb = time.to_scale(Tdb);
            let state = sc_traj.at(time.into_dynamic());
            let r = state.position();
            let v = state.velocity();
            let h = r.cross(v);
            let h_hat = h.normalize();

            let r_sun = self
                .ephemeris
                .position(tdb, sc_traj.origin(), Sun)
                .map_err(|e| PowerError::Eval(EvalError::Ephemeris(Box::new(e))))?;
            let sun_hat = r_sun.normalize();

            offsets.push((time - epoch).to_seconds().to_f64());
            beta_values.push(beta_angle(h_hat, sun_hat));
            flux_values.push(solar_flux(r_sun.length()));
        }

        let beta_series = TimeSeries::try_new(
            epoch,
            offsets.clone(),
            beta_values,
            InterpolationType::Linear,
        )
        .expect("sampled series should have valid dimensions");
        let flux_series =
            TimeSeries::try_new(epoch, offsets, flux_values, InterpolationType::Linear)
                .expect("sampled series should have valid dimensions");

        Ok((sc.id().clone(), eclipse_intervals, beta_series, flux_series))
    }
}
