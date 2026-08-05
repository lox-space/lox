// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! The eager access analysis, staged for deletion. See [`crate::legacy`].

use std::collections::HashMap;

use core::marker::PhantomData;

use lox_bodies::{CoordinateOrigin, Origin, TryMeanRadius, TrySpheroid};
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::orbits::Ensemble;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::TimeScale;

use crate::assets::{AssetId, Scenario, Spacecraft};
use crate::events::{DetectFnExt as _, UniformSampler};
use crate::imaging::analysis::{
    AccessDetectFn, AccessError, AccessPayload, PayloadAccessor, pass_direction_of, sub_sat_sample,
};
use crate::imaging::aoi::{Aoi, AoiId};
use crate::imaging::optical::OpticalPayload;
use crate::imaging::results::AccessWindow;
use crate::imaging::sar::SarPayload;
use crate::par::try_map;

type WindowMap = HashMap<(AssetId, AoiId), Vec<AccessWindow>>;

/// Results of an access analysis.
pub struct AccessResults {
    windows: WindowMap,
}

impl AccessResults {
    pub(super) fn new(windows: WindowMap) -> Self {
        Self { windows }
    }

    /// Returns access windows for a specific (spacecraft, AOI) pair.
    pub fn windows(&self, sc_id: &AssetId, aoi_id: &AoiId) -> &[AccessWindow] {
        self.windows
            .get(&(sc_id.clone(), aoi_id.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns all (spacecraft, AOI) pairs and their access windows.
    pub fn all_windows(&self) -> &WindowMap {
        &self.windows
    }

    /// Returns `true` if no access windows were found.
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Returns the number of (spacecraft, AOI) pairs.
    pub fn num_pairs(&self) -> usize {
        self.windows.len()
    }
}

// ---------------------------------------------------------------------------
// AccessAnalysis orchestrator
// ---------------------------------------------------------------------------

/// Generic access analysis: computes per-(spacecraft, AOI) windows for spacecraft
/// carrying a payload of type `P`.
pub struct AccessAnalysis<'a, P, O: CoordinateOrigin, R: ReferenceFrame>
where
    P: AccessPayload + Copy + Send + Sync,
    Spacecraft: PayloadAccessor<P>,
{
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, O, R>,
    aois: Vec<(AoiId, Aoi)>,
    step: TimeDelta,
    body_fixed_frame: Frame,
    _marker: PhantomData<P>,
}

impl<'a, P, O, R> AccessAnalysis<'a, P, O, R>
where
    P: AccessPayload + Copy + Send + Sync,
    Spacecraft: PayloadAccessor<P>,
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<Origin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<Frame>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    /// Creates a new access analysis. The body-fixed frame defaults to the
    /// scenario origin's IAU frame.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, O, R>,
        aois: Vec<(AoiId, Aoi)>,
    ) -> Self {
        let body_fixed_frame = Frame::Iau(scenario.origin().into());
        Self {
            scenario,
            ensemble,
            aois,
            step: TimeDelta::from_seconds(60),
            body_fixed_frame,
            _marker: PhantomData,
        }
    }

    /// Overrides the time step for event detection (default 60 s).
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Overrides the body-fixed frame (default IAU of scenario origin).
    pub fn with_body_fixed_frame(mut self, frame: Frame) -> Self {
        self.body_fixed_frame = frame;
        self
    }

    /// Computes per-(spacecraft, AOI) access windows.
    pub fn compute(&self) -> Result<AccessResults, AccessError> {
        let interval = *self.scenario.interval();

        let with_payload: Vec<(&Spacecraft, P)> = self
            .scenario
            .spacecraft()
            .iter()
            .filter_map(|sc| <Spacecraft as PayloadAccessor<P>>::extract(sc).map(|p| (sc, p)))
            .collect();

        let pairs: Vec<(&Spacecraft, P, &(AoiId, Aoi))> = with_payload
            .iter()
            .flat_map(|&(sc, p)| self.aois.iter().map(move |aoi| (sc, p, aoi)))
            .collect();

        let compute_one = |&(sc, payload, (aoi_id, aoi)): &(&Spacecraft, P, &(AoiId, Aoi))| {
            let key = (sc.id().clone(), aoi_id.clone());
            let traj = self.ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let detect_fn = AccessDetectFn {
                payload,
                aoi,
                trajectory: traj,
                origin: self.scenario.origin(),
                body_fixed_frame: self.body_fixed_frame,
            };
            let intervals = detect_fn.intervals(UniformSampler::new(self.step), interval)?;
            let origin = self.scenario.origin();
            let body_fixed_frame = self.body_fixed_frame;
            let mut windows: Vec<AccessWindow> = Vec::with_capacity(intervals.len());
            for iv in intervals {
                let midpoint = iv.start() + 0.5 * (iv.end() - iv.start());
                let sample = sub_sat_sample(traj, midpoint, origin, body_fixed_frame)?;
                let direction = pass_direction_of(&sample);
                windows.push(AccessWindow {
                    interval: iv,
                    direction,
                });
            }
            Ok::<_, AccessError>((key, windows))
        };

        let results: Result<Vec<_>, AccessError> = try_map(&pairs, pairs.len() > 100, compute_one);

        let windows_by_pair: HashMap<_, _> = results?.into_iter().collect();
        Ok(AccessResults::new(windows_by_pair))
    }
}

/// Type alias for the optical access analysis (parameterised by [`OpticalPayload`]).
pub type OpticalAccessAnalysis<'a, O, R> = AccessAnalysis<'a, OpticalPayload, O, R>;

/// Type alias for the SAR access analysis (parameterised by [`SarPayload`]).
pub type SarAccessAnalysis<'a, O, R> = AccessAnalysis<'a, SarPayload, O, R>;
