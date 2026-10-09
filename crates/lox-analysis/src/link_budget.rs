// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget analysis: the intervals in which a radio link actually closes.
//!
//! [`VisibilityAnalysis`](crate::visibility::VisibilityAnalysis) answers
//! whether two assets can see each other. [`LinkBudgetAnalysis`] refines those
//! windows down to the sub-intervals in which the link margin between a pair of
//! compatible terminals is non-negative, and annotates each surviving window
//! with the throughput it supports.
//!
//! Terminals pair up automatically: every transmit terminal is matched against
//! every receive terminal on the other asset whose band contains the carrier,
//! in both directions. The carrier, waveform, and coding a link is evaluated
//! against come from a [`CarrierPlan`] held by the analysis, not from the
//! terminals — terminals model hardware and know nothing about symbol rates.

use std::collections::{BTreeMap, HashMap};

use lox_bodies::{CoordinateOrigin, TrySpheroid};
use lox_comms::channel::{Channel, LinkDirection};
use lox_comms::link_budget::{Eirp, GOverT, LinkBudget, LinkConditions};
use lox_comms::modcod::ModCod;
use lox_comms::terminal::{RxTerminal, TxTerminal};
use lox_comms::{LinkBudgetError, PropagationLosses};
use lox_core::error::LoxError;
use lox_core::units::{Decibel, Distance, Frequency};
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::ground::EllipsoidLocation;
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;
use thiserror::Error;

use crate::assets::{AssetId, GroundStation, Scenario, Spacecraft, TerminalId};
use crate::events::{DetectError, DetectFn, DetectFnExt as _, UniformSampler};
use crate::parallel;
use crate::visibility::{EvalError, PairType, VisibilityResults};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from evaluating a link budget at a single time.
#[derive(Debug, Error)]
pub enum LinkEvalError {
    /// Resolving the range between the two ends failed.
    #[error(transparent)]
    Geometry(#[from] EvalError),
    /// The link budget itself could not be computed.
    #[error(transparent)]
    Budget(#[from] LinkBudgetError),
    /// The link margin evaluated to a non-finite value.
    #[error("non-finite link margin at {time}")]
    NonFiniteMargin {
        /// The time at which the margin was evaluated.
        time: Time,
    },
}

// `events::DetectFn` requires `Error: Into<LoxError>` so the lazy machinery can
// erase detector-specific errors.
impl From<LinkEvalError> for LoxError {
    fn from(e: LinkEvalError) -> Self {
        LoxError::new(e)
    }
}

/// Errors from a link budget analysis run.
#[derive(Debug, Error)]
pub enum LinkAnalysisError {
    /// The carrier plan was built without any MODCOD to evaluate against.
    #[error("a carrier plan requires at least one MODCOD")]
    EmptyModCodTable,
    /// The detection MODCOD is not one of the plan's candidate MODCODs.
    #[error("the detection MODCOD `{0}` is not in the carrier plan's MODCOD table")]
    DetectionModCodNotInTable(String),
    /// The sampling step is zero or negative.
    #[error("the sampling step must be positive, but was {0} s")]
    NonPositiveStep(f64),
    /// A spacecraft in the scenario has no trajectory in the ensemble.
    #[error("no trajectory for `{0}` in the ensemble; did you forget to propagate it?")]
    MissingTrajectory(AssetId),
    /// Two ground stations, or two spacecraft, share an id.
    #[error("duplicate asset id `{0}`; ids must be unique within a scenario")]
    DuplicateAssetId(AssetId),
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] DetectError),
    /// Evaluating the budget at a sample failed.
    #[error(transparent)]
    Eval(#[from] LinkEvalError),
}

// ---------------------------------------------------------------------------
// CarrierPlan
// ---------------------------------------------------------------------------

/// The carrier, waveform, and coding a link is evaluated against.
///
/// The MODCOD table drives adaptive coding and modulation: each closing window
/// is annotated with the highest-efficiency mode that closes at its peak. A
/// separate *detection* MODCOD — by default the most robust entry in the table
/// — defines the window boundaries themselves.
#[derive(Debug, Clone)]
pub struct CarrierPlan {
    pub carrier: Frequency,
    pub channel: Channel,
    pub modcods: Vec<ModCod>,
    pub design_margin: Decibel,
    pub losses: PropagationLosses,
    pub detection_modcod: ModCod,
}

impl CarrierPlan {
    /// Creates a carrier plan for the given carrier, waveform, and MODCOD
    /// table.
    ///
    /// The design margin defaults to zero, propagation losses to none, and the
    /// detection MODCOD to the most robust entry in `modcods`.
    ///
    /// Returns [`LinkAnalysisError::EmptyModCodTable`] when `modcods` is empty.
    pub fn new(
        carrier: Frequency,
        channel: Channel,
        modcods: Vec<ModCod>,
    ) -> Result<Self, LinkAnalysisError> {
        let detection_modcod = most_robust(&modcods)
            .ok_or(LinkAnalysisError::EmptyModCodTable)?
            .clone();
        Ok(Self {
            carrier,
            channel,
            modcods,
            design_margin: Decibel::new(0.0),
            losses: PropagationLosses::none(),
            detection_modcod,
        })
    }

    /// Sets the design margin applied on top of the MODCOD threshold.
    pub fn with_design_margin(mut self, design_margin: Decibel) -> Self {
        self.design_margin = design_margin;
        self
    }

    /// Sets the propagation losses applied to every sample.
    pub fn with_losses(mut self, losses: PropagationLosses) -> Self {
        self.losses = losses;
        self
    }

    /// Sets the MODCOD whose margin defines the window boundaries.
    ///
    /// Overrides the default choice of the most robust entry in the table.
    /// The detection MODCOD must itself be in the table: window boundaries are
    /// roots of its margin, so a MODCOD the analysis never selects from would
    /// open windows that report no throughput.
    ///
    /// Returns [`LinkAnalysisError::DetectionModCodNotInTable`] when `modcod`
    /// is absent from the table.
    pub fn with_detection_modcod(mut self, modcod: ModCod) -> Result<Self, LinkAnalysisError> {
        if !self.modcods.contains(&modcod) {
            return Err(LinkAnalysisError::DetectionModCodNotInTable(
                modcod.mode().name().to_string(),
            ));
        }
        self.detection_modcod = modcod;
        Ok(self)
    }

    /// Returns the carrier frequency.
    pub fn carrier(&self) -> Frequency {
        self.carrier
    }

    /// Returns the waveform the link is evaluated on.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Returns the MODCOD table used for adaptive coding and modulation.
    pub fn modcods(&self) -> &[ModCod] {
        &self.modcods
    }

    /// Returns the design margin.
    pub fn design_margin(&self) -> Decibel {
        self.design_margin
    }

    /// Returns the propagation losses.
    pub fn losses(&self) -> &PropagationLosses {
        &self.losses
    }

    /// Returns the MODCOD whose margin defines the window boundaries.
    pub fn detection_modcod(&self) -> &ModCod {
        &self.detection_modcod
    }
}

/// Indexes assets by id.
///
/// Returns [`LinkAnalysisError::DuplicateAssetId`] when two of them share one,
/// which would otherwise resolve a visibility pair to an arbitrary one of the
/// two.
fn index_by_id<'a, A>(
    assets: &'a [A],
    id: impl Fn(&'a A) -> &'a AssetId,
) -> Result<HashMap<&'a AssetId, &'a A>, LinkAnalysisError> {
    let mut index = HashMap::with_capacity(assets.len());
    for asset in assets {
        if index.insert(id(asset), asset).is_some() {
            return Err(LinkAnalysisError::DuplicateAssetId(id(asset).clone()));
        }
    }
    Ok(index)
}

/// Returns the entry of `table` that closes at the lowest Es/N0.
fn most_robust(table: &[ModCod]) -> Option<&ModCod> {
    table.iter().min_by(|a, b| {
        a.required_es_n0()
            .as_f64()
            .total_cmp(&b.required_es_n0().as_f64())
    })
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

/// The pair of ends a link spans, and how to resolve the range between them.
pub enum LinkGeometry<'a, O: CoordinateOrigin, R: ReferenceFrame> {
    /// A ground station and a spacecraft.
    GroundSpace {
        gs: &'a EllipsoidLocation,
        sc: &'a Trajectory<O, R>,
    },
    /// Two spacecraft.
    InterSatellite {
        sc1: &'a Trajectory<O, R>,
        sc2: &'a Trajectory<O, R>,
    },
}

impl<O, R> LinkGeometry<'_, O, R>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Returns the slant range between the two ends at the given time.
    pub fn range(&self, time: Time) -> Result<Distance, EvalError> {
        match self {
            Self::GroundSpace { gs, sc } => {
                let state = sc.at(time.into_dynamic());
                let state = state
                    .try_to_frame(gs.frame(), &DefaultRotationProvider)
                    .map_err(|e| EvalError::Rotation(Box::new(e)))?;
                Ok(gs.range_to(state))
            }
            Self::InterSatellite { sc1, sc2 } => {
                let r1 = sc1.at(time.into_dynamic()).position();
                let r2 = sc2.at(time.into_dynamic()).position();
                Ok(Distance::meters((r1 - r2).length()))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Link
// ---------------------------------------------------------------------------

/// One directed link between two terminals, with the visibility windows it is
/// refined within.
pub struct Link<'a, O: CoordinateOrigin, R: ReferenceFrame> {
    pub tx_id: TerminalId,
    pub rx_id: TerminalId,
    pub direction: LinkDirection,
    pub tx: &'a TxTerminal,
    pub rx: &'a RxTerminal,
    pub plan: &'a CarrierPlan,
    pub geometry: LinkGeometry<'a, O, R>,
    pub visibility: &'a [TimeInterval],
}

impl<O, R> Link<'_, O, R>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Returns the modulation-agnostic budget at the given time.
    fn budget(&self, time: Time) -> Result<LinkBudget, LinkEvalError> {
        let range = self.geometry.range(time)?;
        let conditions = LinkConditions::builder(self.plan.carrier, range)
            .losses(self.plan.losses.clone())
            .direction(self.direction)
            .build()
            .map_err(LinkBudgetError::from)?;
        Ok(LinkBudget::new(self.tx, self.rx, &conditions)?)
    }

    /// Returns the link margin in dB against the plan's detection MODCOD.
    fn margin(&self, time: Time) -> Result<f64, LinkEvalError> {
        Ok(self
            .budget(time)?
            .modulate(
                &self.plan.channel,
                &self.plan.detection_modcod,
                self.plan.design_margin,
            )
            .margin
            .as_f64())
    }
}

/// Link margin against the plan's detection MODCOD, in dB.
///
/// Positive while the link closes, negative while it does not, so its roots are
/// the window boundaries.
pub struct LinkMarginDetectFn<'a, O: CoordinateOrigin, R: ReferenceFrame>(pub &'a Link<'a, O, R>);

impl<O, R> DetectFn for LinkMarginDetectFn<'_, O, R>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = LinkEvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        self.0.margin(time)
    }
}

// ---------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------

/// An interval in which a link closes, with the throughput it supports.
#[derive(Debug, Clone)]
pub struct LinkWindow {
    interval: TimeInterval,
    direction: LinkDirection,
    peak_margin: Decibel,
    min_margin: Decibel,
    peak_modcod: Option<ModCod>,
    symbol_rate: Frequency,
    data_volume: f64,
}

impl LinkWindow {
    /// Returns the interval over which the link closes.
    pub fn interval(&self) -> &TimeInterval {
        &self.interval
    }

    /// Returns the direction of the link.
    pub fn direction(&self) -> LinkDirection {
        self.direction
    }

    /// Returns the highest margin reached within the window, against the
    /// plan's detection MODCOD.
    pub fn peak_margin(&self) -> Decibel {
        self.peak_margin
    }

    /// Returns the lowest margin within the window.
    ///
    /// Zero at a boundary the root finder placed, and positive only where the
    /// window was clipped against the start or the end of a visibility
    /// interval.
    pub fn min_margin(&self) -> Decibel {
        self.min_margin
    }

    /// Returns the MODCOD adaptive coding and modulation selects at the peak,
    /// or `None` when no entry in the table closes there.
    pub fn peak_modcod(&self) -> Option<&ModCod> {
        self.peak_modcod.as_ref()
    }

    /// Returns the information rate supported at the peak, or zero when no
    /// entry in the table closes there.
    pub fn peak_information_rate(&self) -> Frequency {
        self.peak_modcod
            .as_ref()
            .map(|modcod| modcod.mode().information_rate(self.symbol_rate))
            .unwrap_or(Frequency::hertz(0.0))
    }

    /// Returns the data volume deliverable over the window in bits,
    /// integrating the adaptively selected information rate.
    pub fn data_volume(&self) -> f64 {
        self.data_volume
    }
}

/// Link windows keyed by the pair of terminals they connect.
pub type LinkWindowMap = HashMap<(TerminalId, TerminalId), Vec<LinkWindow>>;

/// Results of a link budget analysis.
#[derive(Debug, Clone, Default)]
pub struct LinkBudgetResults {
    windows: LinkWindowMap,
}

impl LinkBudgetResults {
    /// Returns the windows in which the link between two terminals closes.
    ///
    /// Returns an empty slice for a pair that is not a link.
    pub fn windows_for(&self, tx: &TerminalId, rx: &TerminalId) -> &[LinkWindow] {
        self.windows
            .get(&(tx.clone(), rx.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns all windows keyed by terminal pair.
    pub fn all_windows(&self) -> &LinkWindowMap {
        &self.windows
    }

    /// Iterates over the terminal pairs that form a link.
    pub fn link_ids(&self) -> impl Iterator<Item = &(TerminalId, TerminalId)> {
        self.windows.keys()
    }

    /// Returns `true` when no link closed anywhere.
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Returns the number of links that closed at least once.
    pub fn num_links(&self) -> usize {
        self.windows.len()
    }

    /// Returns the total number of windows across all links.
    pub fn total_windows(&self) -> usize {
        self.windows.values().map(|w| w.len()).sum()
    }

    /// Returns the total data volume in bits deliverable over a link.
    pub fn data_volume_for(&self, tx: &TerminalId, rx: &TerminalId) -> f64 {
        self.windows_for(tx, rx)
            .iter()
            .map(LinkWindow::data_volume)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// LinkBudgetAnalysis
// ---------------------------------------------------------------------------

/// Refines visibility windows into the intervals in which links close.
///
/// Pairs every transmit terminal with every receive terminal on the other
/// asset whose band contains the plan's carrier, in both directions, and scans
/// each resulting link inside the visibility windows the pair already has.
pub struct LinkBudgetAnalysis<'a, O: CoordinateOrigin, R: ReferenceFrame> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    visibility: &'a VisibilityResults,
    plan: CarrierPlan,
    overrides: HashMap<(TerminalId, TerminalId), CarrierPlan>,
    step: TimeDelta,
}

impl<'a, O, R> LinkBudgetAnalysis<'a, O, R>
where
    O: TrySpheroid + Copy + Send + Sync,
    R: ReferenceFrame + Copy + Send + Sync,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Creates a link budget analysis refining the given visibility results.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, O, R>,
        visibility: &'a VisibilityResults,
        plan: CarrierPlan,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            visibility,
            plan,
            overrides: HashMap::new(),
            step: TimeDelta::from_seconds(60),
        }
    }

    /// Sets the time step for sampling and event detection.
    ///
    /// The step must be positive; [`compute`](Self::compute) returns
    /// [`LinkAnalysisError::NonPositiveStep`] otherwise.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Overrides the carrier plan for one directed terminal pair.
    pub fn with_override(mut self, tx: TerminalId, rx: TerminalId, plan: CarrierPlan) -> Self {
        self.overrides.insert((tx, rx), plan);
        self
    }

    /// Returns the time step.
    pub fn step(&self) -> TimeDelta {
        self.step
    }

    /// Computes the intervals in which each compatible link closes.
    ///
    /// # Errors
    ///
    /// Returns [`LinkAnalysisError::NonPositiveStep`] for a non-positive step,
    /// [`LinkAnalysisError::MissingTrajectory`] when a spacecraft the
    /// visibility results cover has not been propagated, and
    /// [`LinkAnalysisError::DuplicateAssetId`] when two assets share an id.
    pub fn compute(&self) -> Result<LinkBudgetResults, LinkAnalysisError> {
        if !self.step.is_positive() {
            return Err(LinkAnalysisError::NonPositiveStep(
                self.step.to_seconds().to_f64(),
            ));
        }

        let links = self.links()?;
        let step = self.step;

        let results: Result<Vec<_>, LinkAnalysisError> =
            parallel::try_map(&links, |link| Self::compute_link(link, step));

        let windows: LinkWindowMap = results?
            .into_iter()
            .filter(|(_, w): &(_, Vec<LinkWindow>)| !w.is_empty())
            .collect();

        Ok(LinkBudgetResults { windows })
    }

    /// Returns the carrier plan governing a directed terminal pair.
    fn plan_for(&self, tx: &TerminalId, rx: &TerminalId) -> &CarrierPlan {
        if self.overrides.is_empty() {
            return &self.plan;
        }
        self.overrides
            .get(&(tx.clone(), rx.clone()))
            .unwrap_or(&self.plan)
    }

    /// Returns the propagated trajectory of a spacecraft.
    fn trajectory(&self, id: &AssetId) -> Result<&'a Trajectory<O, R>, LinkAnalysisError> {
        self.ensemble
            .get(id)
            .ok_or_else(|| LinkAnalysisError::MissingTrajectory(id.clone()))
    }

    /// Enumerates every compatible link implied by the visibility results.
    ///
    /// Pairs come from the visibility results rather than being re-derived from
    /// the scenario, so the two cannot disagree over which assets see each
    /// other or over the order a pair is keyed in. A pair whose assets this
    /// scenario does not carry is skipped: a scenario narrower than the one
    /// visibility ran over analyses only its own assets.
    fn links<'s>(&'s self) -> Result<Vec<Link<'s, O, R>>, LinkAnalysisError> {
        let ground_stations = index_by_id(self.scenario.ground_stations(), GroundStation::id)?;
        let spacecraft = index_by_id(self.scenario.spacecraft(), Spacecraft::id)?;

        let mut links = Vec::new();
        for ((id1, id2), visibility) in self.visibility.all_intervals() {
            if visibility.is_empty() {
                continue;
            }
            match self.visibility.pair_type(id1, id2) {
                Some(PairType::GroundSpace) => {
                    let (Some(gs), Some(sc)) = (ground_stations.get(id1), spacecraft.get(id2))
                    else {
                        continue;
                    };
                    let sc_traj = self.trajectory(sc.id())?;
                    let geometry = || LinkGeometry::GroundSpace {
                        gs: gs.location(),
                        sc: sc_traj,
                    };
                    self.push_links(
                        &mut links,
                        sc.id(),
                        sc.tx_terminals(),
                        gs.id(),
                        gs.rx_terminals(),
                        LinkDirection::Downlink,
                        visibility,
                        geometry,
                    );
                    self.push_links(
                        &mut links,
                        gs.id(),
                        gs.tx_terminals(),
                        sc.id(),
                        sc.rx_terminals(),
                        LinkDirection::Uplink,
                        visibility,
                        geometry,
                    );
                }
                Some(PairType::InterSatellite) => {
                    let (Some(sc1), Some(sc2)) = (spacecraft.get(id1), spacecraft.get(id2)) else {
                        continue;
                    };
                    let (traj1, traj2) = (self.trajectory(sc1.id())?, self.trajectory(sc2.id())?);
                    let geometry = || LinkGeometry::InterSatellite {
                        sc1: traj1,
                        sc2: traj2,
                    };
                    self.push_links(
                        &mut links,
                        sc1.id(),
                        sc1.tx_terminals(),
                        sc2.id(),
                        sc2.rx_terminals(),
                        LinkDirection::Crosslink,
                        visibility,
                        geometry,
                    );
                    self.push_links(
                        &mut links,
                        sc2.id(),
                        sc2.tx_terminals(),
                        sc1.id(),
                        sc1.rx_terminals(),
                        LinkDirection::Crosslink,
                        visibility,
                        geometry,
                    );
                }
                None => continue,
            }
        }

        Ok(links)
    }

    /// Adds every band-compatible combination of the given terminal maps.
    #[allow(clippy::too_many_arguments)]
    fn push_links<'s, F>(
        &'s self,
        links: &mut Vec<Link<'s, O, R>>,
        tx_asset: &AssetId,
        tx_terminals: &'s BTreeMap<String, TxTerminal>,
        rx_asset: &AssetId,
        rx_terminals: &'s BTreeMap<String, RxTerminal>,
        direction: LinkDirection,
        visibility: &'s [TimeInterval],
        geometry: F,
    ) where
        F: Fn() -> LinkGeometry<'s, O, R>,
    {
        for (tx_name, tx) in tx_terminals {
            for (rx_name, rx) in rx_terminals {
                let tx_id = TerminalId::new(tx_asset.clone(), tx_name.clone());
                let rx_id = TerminalId::new(rx_asset.clone(), rx_name.clone());
                let plan = self.plan_for(&tx_id, &rx_id);
                // Checking the carrier against both bands here means a link is
                // only ever scanned when its budget is computable, so
                // `CarrierOutOfBand` cannot surface per sample.
                if !tx.band().contains(plan.carrier) || !rx.band().contains(plan.carrier) {
                    continue;
                }
                links.push(Link {
                    tx_id,
                    rx_id,
                    direction,
                    tx,
                    rx,
                    plan,
                    geometry: geometry(),
                    visibility,
                });
            }
        }
    }

    /// Scans one link inside its visibility windows.
    fn compute_link(
        link: &Link<'_, O, R>,
        step: TimeDelta,
    ) -> Result<((TerminalId, TerminalId), Vec<LinkWindow>), LinkAnalysisError> {
        let detector = LinkMarginDetectFn(link);
        let mut windows = Vec::new();

        for visibility in link.visibility {
            for window in detector.iter_intervals(UniformSampler::new(step), *visibility) {
                windows.push(summarize(link, window?, step)?);
            }
        }

        Ok(((link.tx_id.clone(), link.rx_id.clone()), windows))
    }
}

/// Samples a closing window and derives its margin extremes and throughput.
///
/// Uses at least two sub-intervals so that a window shorter than the step still
/// gets an interior sample, and integrates the adaptively selected information
/// rate by the trapezoidal rule.
///
/// Returns [`LinkEvalError::NonFiniteMargin`] when the margin is not finite at
/// any sample.
pub fn summarize<O, R>(
    link: &Link<'_, O, R>,
    interval: TimeInterval,
    step: TimeDelta,
) -> Result<LinkWindow, LinkEvalError>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    let plan = link.plan;
    let start = interval.start();
    let duration = interval.duration().to_seconds().to_f64();
    let step_seconds = step.to_seconds().to_f64();
    let subdivisions = if step_seconds > 0.0 {
        ((duration / step_seconds).ceil() as usize).max(2)
    } else {
        2
    };
    let dt = duration / subdivisions as f64;

    let mut peak_margin = f64::NEG_INFINITY;
    let mut min_margin = f64::INFINITY;
    let mut peak_modcod = None;
    let mut data_volume = 0.0;
    let mut previous_rate: Option<f64> = None;

    // The rate the link is guaranteed at a boundary, where the detection
    // MODCOD's margin is zero by construction.
    let detection_rate = plan
        .detection_modcod
        .mode()
        .information_rate(plan.channel.symbol_rate())
        .to_hertz();

    for i in 0..=subdivisions {
        let time = start + TimeDelta::from_seconds_f64(i as f64 * dt);
        let budget = link.budget(time)?;
        let margin = budget
            .modulate(&plan.channel, &plan.detection_modcod, plan.design_margin)
            .margin
            .as_f64();
        if !margin.is_finite() {
            return Err(LinkEvalError::NonFiniteMargin { time });
        }

        let best = budget.modulate_best(&plan.channel, &plan.modcods, plan.design_margin);
        // At the two boundary samples the detection MODCOD sits on its own
        // root, so whichever side of zero the root finder landed on decides
        // whether adaptive selection picks anything at all. The link closes
        // there by definition, at the detection MODCOD's rate — taking zero
        // instead would drop a whole trapezoid cell from the integral.
        let rate = best
            .as_ref()
            .map(|b| b.information_rate().to_hertz())
            .unwrap_or(detection_rate);

        if margin > peak_margin {
            peak_margin = margin;
            peak_modcod = best.as_ref().map(|b| b.modcod.clone());
        }
        min_margin = min_margin.min(margin);

        if let Some(previous) = previous_rate {
            data_volume += 0.5 * (previous + rate) * dt;
        }
        previous_rate = Some(rate);
    }

    Ok(LinkWindow {
        interval,
        direction: link.direction,
        peak_margin: Decibel::new(peak_margin),
        min_margin: Decibel::new(min_margin),
        peak_modcod,
        symbol_rate: plan.channel.symbol_rate(),
        data_volume,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use lox_approx::assert_approx_eq;
    use lox_bodies::Origin;
    use lox_comms::antenna::Antenna;
    use lox_comms::modcod::dvb_s2;
    use lox_comms::receiver::CascadeReceiver;
    use lox_comms::terminal::{EirpModel, GtModel, RxChain, TxChain};
    use lox_comms::transmitter::AmplifierTransmitter;
    use lox_core::comms::{FrequencyBand, FrequencyRange};
    use lox_core::coords::LonLatAlt;
    use lox_core::units::{Power, Temperature};
    use lox_orbits::propagators::OrbitSource;
    use lox_test_utils::read_data_file;
    use lox_time::time_scales::TimeScale;

    use super::*;
    use crate::visibility::VisibilityAnalysis;

    const GS: &str = "cebreros";
    const SC: &str = "lunar";
    const SC1: &str = "relay";
    const SC2: &str = "leo";
    const CARRIER: Frequency = Frequency::gigahertz(8.2);

    /// Geometry shared by every test: visibility depends only on the location,
    /// mask, and trajectory, never on the terminals, so it is computed once and
    /// reused across scenarios that differ only in their terminal fit-out.
    struct Fixture {
        location: EllipsoidLocation,
        trajectory: Trajectory,
        ensemble: Ensemble<AssetId, Origin, Frame>,
        visibility: VisibilityResults,
    }

    /// Loads a trajectory from the workspace fixture directory.
    fn trajectory(file: &str) -> Trajectory {
        Trajectory::from_csv_dynamic(&read_data_file(file), Origin::Earth, Frame::Icrf).unwrap()
    }

    /// Pins a dynamic trajectory to TAI so it can go into an ensemble.
    fn typed(trajectory: &Trajectory) -> Trajectory {
        let (epoch, origin, frame, data) = trajectory.clone().into_parts();
        Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data)
    }

    fn fixture() -> &'static Fixture {
        static FIXTURE: OnceLock<Fixture> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
            let location = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
            let trajectory = trajectory("trajectory_lunar.csv");

            let mut map = HashMap::new();
            map.insert(AssetId::new(SC), typed(&trajectory));
            let ensemble = Ensemble::new(map);

            let bare = scenario(&location, &trajectory, &[], &[]);
            let visibility = VisibilityAnalysis::new(&bare, &ensemble).compute().unwrap();

            Fixture {
                location,
                trajectory,
                ensemble,
                visibility,
            }
        })
    }

    /// Two spacecraft and the inter-satellite visibility between them, keyed
    /// in `(SC1, SC2)` order by the scenario it was computed from.
    struct CrosslinkFixture {
        sc1: Trajectory,
        sc2: Trajectory,
        ensemble: Ensemble<AssetId, Origin, Frame>,
        visibility: VisibilityResults,
    }

    fn crosslink_fixture() -> &'static CrosslinkFixture {
        static FIXTURE: OnceLock<CrosslinkFixture> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let sc1 = trajectory("trajectory_lunar.csv");
            let sc2 = trajectory("trajectory_cebr.csv");

            let mut map = HashMap::new();
            map.insert(AssetId::new(SC1), typed(&sc1));
            map.insert(AssetId::new(SC2), typed(&sc2));
            let ensemble = Ensemble::new(map);

            let interval = TimeInterval::new(sc1.start_time(), sc1.end_time());
            let bare = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
                .with_spacecraft(&[
                    Spacecraft::new(SC1, OrbitSource::Trajectory(sc1.clone())),
                    Spacecraft::new(SC2, OrbitSource::Trajectory(sc2.clone())),
                ]);
            let visibility = VisibilityAnalysis::new(&bare, &ensemble)
                .with_inter_satellite()
                .compute()
                .unwrap();

            CrosslinkFixture {
                sc1,
                sc2,
                ensemble,
                visibility,
            }
        })
    }

    /// Builds a crosslink scenario from `SC1` to `SC2`, listing the two
    /// spacecraft in the given order.
    fn crosslink_scenario(f: &CrosslinkFixture, reversed: bool) -> Scenario<Origin, Frame> {
        let (tx, rx) = lumped(200.0, 40.0);
        let sc1 = Spacecraft::new(SC1, OrbitSource::Trajectory(f.sc1.clone()))
            .with_tx_terminal("crosslink", tx);
        let sc2 = Spacecraft::new(SC2, OrbitSource::Trajectory(f.sc2.clone()))
            .with_rx_terminal("crosslink", rx);
        let spacecraft = if reversed { [sc2, sc1] } else { [sc1, sc2] };
        let interval = TimeInterval::new(f.sc1.start_time(), f.sc1.end_time());
        Scenario::with_interval(interval, Origin::Earth, Frame::Icrf).with_spacecraft(&spacecraft)
    }

    /// Builds a scenario whose spacecraft carries `sc_tx` transmit terminals and
    /// whose ground station carries `gs_rx` receive terminals.
    fn scenario(
        location: &EllipsoidLocation,
        trajectory: &Trajectory,
        sc_tx: &[(&str, TxTerminal)],
        gs_rx: &[(&str, RxTerminal)],
    ) -> Scenario<Origin, Frame> {
        let mut gs = GroundStation::new(GS, location.clone());
        for (name, terminal) in gs_rx {
            gs = gs.with_rx_terminal(*name, terminal.clone());
        }
        let mut sc = Spacecraft::new(SC, OrbitSource::Trajectory(trajectory.clone()));
        for (name, terminal) in sc_tx {
            sc = sc.with_tx_terminal(*name, terminal.clone());
        }
        let interval = TimeInterval::new(trajectory.start_time(), trajectory.end_time());
        Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(&[gs])
            .with_spacecraft(&[sc])
    }

    /// A lumped downlink with the given EIRP and G/T, both in X band.
    fn lumped(eirp_dbw: f64, gt_db_k: f64) -> (TxTerminal, RxTerminal) {
        let band = FrequencyBand::X;
        (
            EirpModel::new(band, Decibel::new(eirp_dbw)).unwrap().into(),
            GtModel::new(band, Decibel::new(gt_db_k)).unwrap().into(),
        )
    }

    fn plan() -> CarrierPlan {
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        CarrierPlan::new(CARRIER, channel, dvb_s2().to_vec()).unwrap()
    }

    fn downlink_ids() -> (TerminalId, TerminalId) {
        (
            TerminalId::new(AssetId::new(SC), "downlink"),
            TerminalId::new(AssetId::new(GS), "main"),
        )
    }

    /// Runs the analysis over a lumped downlink with the given figures.
    fn run(eirp_dbw: f64, gt_db_k: f64, plan: CarrierPlan) -> LinkBudgetResults {
        let f = fixture();
        let (tx, rx) = lumped(eirp_dbw, gt_db_k);
        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx)],
            &[("main", rx)],
        );
        LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan)
            .compute()
            .unwrap()
    }

    fn visibility_windows() -> &'static [TimeInterval] {
        fixture()
            .visibility
            .intervals_for(&AssetId::new(GS), &AssetId::new(SC))
            .unwrap()
    }

    // -----------------------------------------------------------------------

    #[test]
    fn test_carrier_plan_rejects_empty_modcod_table() {
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        assert!(matches!(
            CarrierPlan::new(CARRIER, channel, vec![]),
            Err(LinkAnalysisError::EmptyModCodTable)
        ));
    }

    #[test]
    fn test_detection_modcod_defaults_to_most_robust() {
        let plan = plan();
        let weakest = dvb_s2()
            .iter()
            .map(|mc| mc.required_es_n0().as_f64())
            .fold(f64::INFINITY, f64::min);
        assert_approx_eq!(
            plan.detection_modcod().required_es_n0().as_f64(),
            weakest,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_no_link_when_carrier_outside_terminal_bands() {
        // Terminals are X band; the plan puts the carrier in S band.
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        let s_band =
            CarrierPlan::new(Frequency::gigahertz(2.2), channel, dvb_s2().to_vec()).unwrap();
        let results = run(100.0, 40.0, s_band);
        assert!(
            results.is_empty(),
            "a carrier outside both bands cannot form a link"
        );
    }

    #[test]
    fn test_overspecified_link_reproduces_visibility_windows() {
        // With an absurd EIRP the margin never goes negative, so refinement
        // can only return the visibility windows unchanged.
        let results = run(200.0, 40.0, plan());
        let (tx, rx) = downlink_ids();
        let windows = results.windows_for(&tx, &rx);
        let visibility = visibility_windows();

        assert_eq!(windows.len(), visibility.len());
        for (window, expected) in windows.iter().zip(visibility) {
            assert_eq!(window.interval().start(), expected.start());
            assert_eq!(window.interval().end(), expected.end());
            assert_eq!(window.direction(), LinkDirection::Downlink);
        }
    }

    #[test]
    fn test_design_margin_can_close_nothing() {
        let results = run(200.0, 40.0, plan().with_design_margin(Decibel::new(1000.0)));
        assert!(results.is_empty());
    }

    #[test]
    fn test_windows_are_subsets_of_visibility() {
        // 40 dB of combined EIRP + G/T closes at short range but not at long,
        // so the scan genuinely refines rather than passing windows through.
        let results = run(20.0, 20.0, plan());
        let (tx, rx) = downlink_ids();
        let windows = results.windows_for(&tx, &rx);
        let visibility = visibility_windows();

        assert!(!windows.is_empty(), "expected the link to close somewhere");
        assert!(
            windows.len() < visibility.len()
                || windows
                    .iter()
                    .zip(visibility)
                    .any(|(w, v)| w.interval().duration() < v.duration()),
            "expected refinement to shrink at least one window"
        );

        for window in windows {
            let contained = visibility.iter().any(|v| {
                window.interval().start() >= v.start() && window.interval().end() <= v.end()
            });
            assert!(
                contained,
                "link window {:?} escapes every visibility window",
                window.interval()
            );
        }
    }

    #[test]
    fn test_margin_vanishes_at_an_interior_boundary() {
        let f = fixture();
        let (tx, rx) = lumped(20.0, 20.0);
        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx)],
            &[("main", rx)],
        );
        let analysis = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan());
        let links = analysis.links().unwrap();
        let link = links.first().expect("one downlink");
        let results = analysis.compute().unwrap();
        let (tx_id, rx_id) = downlink_ids();
        let visibility = visibility_windows();

        // A boundary strictly inside a visibility window is a root of the
        // margin, not a clip against the end of the scan.
        let mut checked = 0;
        for window in results.windows_for(&tx_id, &rx_id) {
            for edge in [window.interval().start(), window.interval().end()] {
                let interior = visibility
                    .iter()
                    .any(|v| edge > v.start() && edge < v.end());
                if interior {
                    assert_approx_eq!(link.margin(edge).unwrap(), 0.0, atol <= 1e-3);
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "expected at least one interior boundary");
    }

    #[test]
    fn test_data_volume_is_non_decreasing_in_eirp() {
        let (tx, rx) = downlink_ids();
        let volumes: Vec<f64> = [10.0, 20.0, 30.0, 40.0]
            .into_iter()
            .map(|eirp| run(eirp, 20.0, plan()).data_volume_for(&tx, &rx))
            .collect();

        assert!(
            volumes.last().unwrap() > &0.0,
            "the strongest link delivers data"
        );
        for pair in volumes.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "data volume fell from {} to {} as EIRP rose",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn test_empty_results_report_no_volume() {
        let results = run(200.0, 40.0, plan().with_design_margin(Decibel::new(1000.0)));
        let (tx, rx) = downlink_ids();
        assert_eq!(results.data_volume_for(&tx, &rx), 0.0);
        assert_eq!(results.total_windows(), 0);
        assert_eq!(results.num_links(), 0);
        assert!(results.windows_for(&tx, &rx).is_empty());
    }

    #[test]
    fn test_uplink_and_downlink_are_distinct_links() {
        let f = fixture();
        let band = FrequencyBand::X;
        let sc_tx: TxTerminal = EirpModel::new(band, Decibel::new(200.0)).unwrap().into();
        let sc_rx: RxTerminal = GtModel::new(band, Decibel::new(40.0)).unwrap().into();
        let gs_tx: TxTerminal = EirpModel::new(band, Decibel::new(200.0)).unwrap().into();
        let gs_rx: RxTerminal = GtModel::new(band, Decibel::new(40.0)).unwrap().into();

        let gs = GroundStation::new(GS, f.location.clone())
            .with_rx_terminal("main", gs_rx)
            .with_tx_terminal("uplink", gs_tx);
        let sc = Spacecraft::new(SC, OrbitSource::Trajectory(f.trajectory.clone()))
            .with_tx_terminal("downlink", sc_tx)
            .with_rx_terminal("cmd", sc_rx);
        let interval = TimeInterval::new(f.trajectory.start_time(), f.trajectory.end_time());
        let scenario = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(&[gs])
            .with_spacecraft(&[sc]);

        let results = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan())
            .compute()
            .unwrap();

        assert_eq!(results.num_links(), 2);
        let (sc_down, gs_main) = downlink_ids();
        let down = results.windows_for(&sc_down, &gs_main);
        let up = results.windows_for(
            &TerminalId::new(AssetId::new(GS), "uplink"),
            &TerminalId::new(AssetId::new(SC), "cmd"),
        );
        assert!(!down.is_empty() && !up.is_empty());
        assert_eq!(down[0].direction(), LinkDirection::Downlink);
        assert_eq!(up[0].direction(), LinkDirection::Uplink);
    }

    #[test]
    fn test_override_applies_to_one_pair_only() {
        let f = fixture();
        let (tx, rx) = lumped(20.0, 20.0);
        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx)],
            &[("main", rx)],
        );
        let (tx_id, rx_id) = downlink_ids();

        let baseline = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan())
            .compute()
            .unwrap();
        let overridden = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan())
            .with_override(
                tx_id.clone(),
                rx_id.clone(),
                plan().with_design_margin(Decibel::new(1000.0)),
            )
            .compute()
            .unwrap();

        assert!(!baseline.windows_for(&tx_id, &rx_id).is_empty());
        assert!(overridden.windows_for(&tx_id, &rx_id).is_empty());
    }

    /// The budget the analysis evaluates must be the one `lox-comms` computes
    /// for the same geometry — including the downlink tag, which costs ~1.8 dB
    /// of rain-degraded G/T on a component receive chain.
    #[test]
    fn test_budget_matches_lox_comms_including_direction() {
        let f = fixture();
        let eess =
            FrequencyRange::new(Frequency::gigahertz(8.025), Frequency::gigahertz(8.4)).unwrap();
        let tx: TxTerminal = TxChain::new(
            Antenna::parabolic(Distance::meters(0.25), 0.6).unwrap(),
            AmplifierTransmitter::new(Power::watts(2.0), Decibel::new(0.5)).unwrap(),
            Decibel::new(0.8),
            eess,
        )
        .unwrap()
        .into();
        let front_end = CascadeReceiver::builder()
            .stage(Decibel::new(35.0), Temperature::kelvin(50.0))
            .stage(Decibel::new(0.0), Temperature::kelvin(1540.0))
            .demodulator_loss(Decibel::new(0.5))
            .implementation_loss(Decibel::new(0.5))
            .build()
            .unwrap();
        let rx: RxTerminal = RxChain::new(
            Antenna::parabolic(Distance::meters(3.7), 0.6).unwrap(),
            front_end,
            Decibel::new(0.3),
            Temperature::kelvin(60.0),
            eess,
        )
        .unwrap()
        .into();

        let losses = PropagationLosses::builder()
            .rain(Decibel::new(1.2))
            .gaseous(Decibel::new(0.4))
            .build()
            .unwrap();
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        let plan = CarrierPlan::new(CARRIER, channel, dvb_s2().to_vec())
            .unwrap()
            .with_losses(losses.clone());

        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx.clone())],
            &[("main", rx.clone())],
        );
        let analysis = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan);
        let links = analysis.links().unwrap();
        let link = links.first().expect("one downlink");

        let time = visibility_windows()[0].start();
        let range = link.geometry.range(time).unwrap();
        let actual = link.budget(time).unwrap();

        let downlink = LinkConditions::builder(CARRIER, range)
            .losses(losses.clone())
            .direction(LinkDirection::Downlink)
            .build()
            .unwrap();
        let expected = LinkBudget::new(&tx, &rx, &downlink).unwrap();
        assert_approx_eq!(actual.c_n0.as_f64(), expected.c_n0.as_f64(), atol <= 1e-12);
        assert!(
            actual.gt_degraded.is_some(),
            "a downlink onto a component receive chain degrades G/T"
        );

        // Without the downlink tag the very same geometry gives a different
        // answer, so the assertion above is not vacuous.
        let untagged = LinkConditions::builder(CARRIER, range)
            .losses(losses)
            .build()
            .unwrap();
        let clear_sky = LinkBudget::new(&tx, &rx, &untagged).unwrap();
        assert!(clear_sky.c_n0.as_f64() > expected.c_n0.as_f64() + 1.0);
    }

    #[test]
    fn test_detection_modcod_must_be_in_the_table() {
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        let table = dvb_s2();
        let plan = CarrierPlan::new(CARRIER, channel, table[..2].to_vec()).unwrap();

        assert!(matches!(
            plan.clone().with_detection_modcod(table[5].clone()),
            Err(LinkAnalysisError::DetectionModCodNotInTable(_))
        ));

        let member = table[1].clone();
        let plan = plan.with_detection_modcod(member.clone()).unwrap();
        assert_eq!(plan.detection_modcod(), &member);
    }

    #[test]
    fn test_non_positive_step_is_rejected() {
        let f = fixture();
        let (tx, rx) = lumped(200.0, 40.0);
        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx)],
            &[("main", rx)],
        );

        for step in [TimeDelta::from_seconds(0), TimeDelta::from_seconds(-60)] {
            let result = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan())
                .with_step(step)
                .compute();
            assert!(
                matches!(result, Err(LinkAnalysisError::NonPositiveStep(_))),
                "a step of {step:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_unpropagated_spacecraft_is_reported() {
        let f = fixture();
        let (tx, rx) = lumped(200.0, 40.0);
        let scenario = scenario(
            &f.location,
            &f.trajectory,
            &[("downlink", tx)],
            &[("main", rx)],
        );
        let empty = Ensemble::new(HashMap::new());

        let result = LinkBudgetAnalysis::new(&scenario, &empty, &f.visibility, plan()).compute();
        assert!(
            matches!(&result, Err(LinkAnalysisError::MissingTrajectory(id)) if id.as_str() == SC),
            "a spacecraft missing from the ensemble must be reported, got {result:?}"
        );
    }

    #[test]
    fn test_duplicate_asset_ids_are_rejected() {
        let f = fixture();
        let (tx, rx) = lumped(200.0, 40.0);
        let gs = GroundStation::new(GS, f.location.clone()).with_rx_terminal("main", rx);
        // Two spacecraft sharing an id resolve to the same pair of terminals,
        // which must not silently overwrite one another.
        let sc = Spacecraft::new(SC, OrbitSource::Trajectory(f.trajectory.clone()))
            .with_tx_terminal("downlink", tx);
        let interval = TimeInterval::new(f.trajectory.start_time(), f.trajectory.end_time());
        let scenario = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(&[gs])
            .with_spacecraft(&[sc.clone(), sc]);

        let result =
            LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan()).compute();
        assert!(
            matches!(&result, Err(LinkAnalysisError::DuplicateAssetId(id)) if id.as_str() == SC),
            "a duplicate asset id must be reported, got {result:?}"
        );
    }

    #[test]
    fn test_data_volume_covers_the_whole_window() {
        // With a single-entry MODCOD table adaptive selection has no choice to
        // make: the rate is that entry's at every sample inside the window, and
        // at the two boundaries, where its margin is zero by construction. The
        // integral must therefore cover the window exactly, rather than losing
        // the two half-cells at its ends.
        let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
        let table = vec![most_robust(dvb_s2()).unwrap().clone()];
        let plan = CarrierPlan::new(CARRIER, channel, table).unwrap();
        let rate = plan
            .detection_modcod()
            .mode()
            .information_rate(plan.channel().symbol_rate())
            .to_hertz();

        let results = run(20.0, 20.0, plan);
        let (tx, rx) = downlink_ids();
        let windows = results.windows_for(&tx, &rx);
        assert!(!windows.is_empty(), "expected the link to close somewhere");

        for window in windows {
            let duration = window.interval().duration().to_seconds().to_f64();
            assert_approx_eq!(window.data_volume(), rate * duration, rtol <= 1e-12);
        }
    }

    #[test]
    fn test_crosslink_found_under_reversed_spacecraft_order() {
        let f = crosslink_fixture();
        assert!(
            f.visibility
                .intervals_for(&AssetId::new(SC1), &AssetId::new(SC2))
                .is_some_and(|i| !i.is_empty()),
            "the fixture must have inter-satellite visibility under the (SC1, SC2) key"
        );

        // The visibility results key the pair in the order the scenario they
        // were computed from listed it; this scenario lists it the other way
        // round.
        for reversed in [false, true] {
            let scenario = crosslink_scenario(f, reversed);
            let analysis = LinkBudgetAnalysis::new(&scenario, &f.ensemble, &f.visibility, plan());
            let links = analysis.links().unwrap();
            assert_eq!(links.len(), 1, "reversed = {reversed}");
            assert_eq!(links[0].direction, LinkDirection::Crosslink);
        }
    }
}
