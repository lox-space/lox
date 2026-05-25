// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { SvelteMap } from "svelte/reactivity";
import { untrack } from "svelte";
import type { Scenario } from "./scenario.svelte";
import { runWalker } from "$lib/walker.svelte";
import { runComputeAccess } from "$lib/rpc/client";
import { computeStats, type AccessWindowLite, type AoiStats } from "./access.svelte";
import type { AccessRequest } from "@kerolox/proto-ts";

export type SweepParam = "satsPerPlane" | "planes" | "phasing" | "altitudeKm" | "inclinationDeg";
export type SweepMetric = "meanGap" | "medianGap" | "maxGap" | "count" | "totalAccess";

export interface SweepConfig {
  param: SweepParam;
  min: number;
  max: number;
  step: number;
  metric: SweepMetric;
  concurrency: number;
}

export interface SweepPoint {
  x: number;       // the swept parameter value
  y: number;       // the metric value
}

/** Chart points keyed by AOI id, in ascending x order. */
export const sweepPoints = new SvelteMap<string, SweepPoint[]>();
export const sweepRunning = $state({ value: false, done: 0, total: 0 });

export function resetSweep(): void {
  sweepPoints.clear();
  sweepRunning.value = false;
  sweepRunning.done = 0;
  sweepRunning.total = 0;
}

function sweepValues(cfg: SweepConfig): number[] {
  const out: number[] = [];
  for (let v = cfg.min; v <= cfg.max + 1e-9; v += cfg.step) out.push(Number(v.toFixed(6)));
  return out;
}

/** Apply a sweep value to a scenario copy, returning a fresh scenario. */
function withParam(base: Scenario, param: SweepParam, value: number): Scenario {
  // `base` may be the reactive `$state` scenario proxy, which structuredClone
  // cannot handle. $state.snapshot yields a plain, deeply-cloned object.
  const s: Scenario = $state.snapshot(base) as Scenario;
  if (param === "satsPerPlane") s.walker.satsPerPlane = Math.round(value);
  else if (param === "planes") s.walker.p = Math.round(value);
  else if (param === "phasing") s.walker.f = Math.round(value);
  else if (param === "altitudeKm") s.walker.altitudeKm = value;
  else if (param === "inclinationDeg") s.walker.inclinationDeg = value;
  return s;
}

function metricOf(stats: AoiStats, metric: SweepMetric): number {
  switch (metric) {
    case "meanGap": return stats.meanGapSeconds ?? 0;
    case "medianGap": return stats.medianGapSeconds ?? 0;
    case "maxGap": return stats.maxGapSeconds ?? 0;
    case "count": return stats.count;
    case "totalAccess": return stats.totalAccessSeconds;
  }
}

/**
 * Run a parameter sweep. For each value, fires a ComputeAccess, aggregates
 * the per-AOI windows into the chosen metric, and appends one point per AOI.
 * Bounded to `cfg.concurrency` in-flight scenarios. Honours `signal`.
 */
export async function runSweep(base: Scenario, cfg: SweepConfig, signal: AbortSignal): Promise<void> {
  if (signal.aborted) return;
  const values = sweepValues(cfg);
  untrack(() => {
    sweepPoints.clear();
    sweepRunning.value = true;
    sweepRunning.done = 0;
    sweepRunning.total = values.length;
  });

  const scenarioStartMs = Date.parse(base.startTimeIso);
  const scenarioEndMs = scenarioStartMs + base.durationHours * 3600 * 1000;

  let cursor = 0;
  async function worker(): Promise<void> {
    while (cursor < values.length && !signal.aborted) {
      const value = values[cursor++];
      const scenario = withParam(base, cfg.param, value);
      const sats = runWalker(scenario);
      if (sats.length === 0) continue;

      // Accumulate windows per AOI for this single scenario.
      const byAoi = new Map<string, AccessWindowLite[]>();
      const req = buildRequest(scenario, sats);
      await runComputeAccess(req, {
        onStart: () => {},
        onPair: (p) => {
          const arr = byAoi.get(p.aoiId) ?? [];
          for (const w of p.windows) {
            arr.push({
              scId: p.scId,
              startMs: Date.parse(w.startIso),
              endMs: Date.parse(w.endIso),
              direction: "ascending",
              source: "user",
            });
          }
          byAoi.set(p.aoiId, arr);
        },
        onDone: () => {},
        onCancel: () => {},
        onError: (err) => console.error(`sweep point ${cfg.param}=${value} failed:`, err),
      }, signal);

      // If the stream was aborted mid-flight, drop the partial scenario
      // rather than charting an incomplete point.
      if (signal.aborted) break;

      untrack(() => {
        for (const [aoiId, windows] of byAoi) {
          const stats = computeStats(windows, scenarioStartMs, scenarioEndMs);
          const y = metricOf(stats, cfg.metric);
          // Build a NEW array rather than mutating in place: SvelteMap.set with
          // an identity-unchanged value does not notify subscribers, so an
          // in-place push + re-set would never update the chart past the first
          // point. A fresh array (matching the access store's concat pattern)
          // makes each set a genuine change.
          const points = [...(sweepPoints.get(aoiId) ?? []), { x: value, y }];
          points.sort((a, b) => a.x - b.x);
          sweepPoints.set(aoiId, points);
        }
        sweepRunning.done += 1;
      });
    }
  }

  const workers = Array.from({ length: Math.max(1, cfg.concurrency) }, () => worker());
  await Promise.all(workers);
  untrack(() => { sweepRunning.value = false; });
}

function buildRequest(scenario: Scenario, sats: ReturnType<typeof runWalker>): AccessRequest {
  return {
    startTimeIso: scenario.startTimeIso,
    durationSeconds: scenario.durationHours * 3600,
    satellites: sats.map((s) => ({
      id: `p${s.plane}-s${s.indexInPlane}`,
      smaM: s.smaM, ecc: s.ecc, incRad: s.incRad, raanRad: s.raanRad,
      aopRad: s.aopRad, trueAnomalyRad: s.trueAnomalyRad, plane: s.plane, indexInPlane: s.indexInPlane,
    })) as unknown as AccessRequest["satellites"],
    sar: {
      lookSide: scenario.sar.lookSide === "LEFT" ? 1 : 2,
      minIncidenceDeg: scenario.sar.minIncidenceDeg,
      maxIncidenceDeg: scenario.sar.maxIncidenceDeg,
    } as unknown as AccessRequest["sar"],
    aoiIds: ["hormuz", "black_sea"],
    comparators: [],
    stepSeconds: 30,
  } as unknown as AccessRequest;
}
