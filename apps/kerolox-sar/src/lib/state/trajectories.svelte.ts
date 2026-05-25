// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { untrack } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import type { Scenario } from "./scenario.svelte";
import type { SatelliteElements } from "$lib/walker.svelte";
import { runPropagateTrajectories } from "$lib/rpc/client";
import type { PropagateRequest } from "@kerolox/proto-ts";
import { propagationStatus } from "./status.svelte";

export interface SampledTrajectoryView {
  /** Unix epoch ms per sample. */
  epochsMs: Float64Array;
  /** Interleaved Three.js ECI positions (Y-up), km. */
  eciKm: Float64Array;
  /** Interleaved lat/lon, degrees. */
  groundDeg: Float64Array;
}

export const trajectoryById = new SvelteMap<string, SampledTrajectoryView>();
/** Comparator (e.g. ICEYE/SGP4) trajectories, keyed by namespaced sc_id
 *  "iceye/<name>". Rendered in a distinct amber family in the viewports. */
export const comparatorTrajectoryById = new SvelteMap<string, SampledTrajectoryView>();
export const currentSatellites = $state<SatelliteElements[]>([]);

let currentAbort: AbortController | null = null;
let currentHash: string | null = null;

export function resetTrajectories(): void {
  trajectoryById.clear();
  comparatorTrajectoryById.clear();
  currentSatellites.length = 0;
  currentHash = null;
  if (currentAbort) currentAbort.abort();
  currentAbort = null;
}

export function scenarioHash(s: Scenario, sats: SatelliteElements[], compareIceye: boolean): string {
  const satParts = sats.map((x) =>
    `${x.plane}.${x.indexInPlane}.${x.smaM.toFixed(1)}.${x.ecc.toFixed(6)}.${x.incRad.toFixed(9)}.${x.raanRad.toFixed(9)}.${x.aopRad.toFixed(9)}.${x.trueAnomalyRad.toFixed(9)}`,
  );
  return `${s.startTimeIso}|${s.durationHours}|${compareIceye ? "+iceye" : ""}|${satParts.join(";")}`;
}

/**
 * Stream trajectories for the current scenario from the engine. Cancels any
 * in-flight propagation; populates trajectoryById as messages arrive.
 *
 * This function is SYNCHRONOUS — it kicks off the stream and returns
 * immediately. Results arrive asynchronously via the trajectory cache.
 */
export function ensureTrajectories(s: Scenario, sats: SatelliteElements[], compareIceye: boolean): void {
  // Wrap the body in untrack so reads and writes of trajectoryById and
  // currentSatellites inside don't become dependencies of any caller's
  // $effect — otherwise the early-return read of trajectoryById.size + the
  // trajectoryById.clear() write below trip Svelte's same-state cycle
  // detector. The caller is responsible for reading whatever scenario
  // properties drive re-runs (it already does).
  untrack(() => {
    const hash = scenarioHash(s, sats, compareIceye);
    // The size guard only applies when no comparators are requested (their
    // count is unknown client-side); the hash captures compareIceye otherwise.
    if (hash === currentHash && (compareIceye || trajectoryById.size === sats.length)) return;
    if (currentAbort) currentAbort.abort();

    trajectoryById.clear();
    comparatorTrajectoryById.clear();
    currentSatellites.length = 0;
    currentSatellites.push(...sats);
    currentHash = hash;

    const ctl = new AbortController();
    currentAbort = ctl;

    const req: PropagateRequest = {
      startTimeIso: s.startTimeIso,
      durationSeconds: s.durationHours * 3600,
      stepSeconds: 30,
      satellites: sats.map((sat) => ({
        id: `p${sat.plane}-s${sat.indexInPlane}`,
        smaM: sat.smaM,
        ecc: sat.ecc,
        incRad: sat.incRad,
        raanRad: sat.raanRad,
        aopRad: sat.aopRad,
        trueAnomalyRad: sat.trueAnomalyRad,
        plane: sat.plane,
        indexInPlane: sat.indexInPlane,
      })) as unknown as PropagateRequest["satellites"],
      comparators: compareIceye ? ["iceye"] : [],
    } as unknown as PropagateRequest;

    // Pending views split by destination map: user trajectories vs comparator
    // (non-empty comparatorId) trajectories.
    const pending = new Map<string, SampledTrajectoryView>();
    const pendingComparator = new Map<string, SampledTrajectoryView>();
    let flushScheduled = false;

    const flush = (): void => {
      flushScheduled = false;
      if (ctl.signal.aborted || (pending.size === 0 && pendingComparator.size === 0)) return;
      untrack(() => {
        for (const [id, view] of pending) {
          trajectoryById.set(id, view);
        }
        for (const [id, view] of pendingComparator) {
          comparatorTrajectoryById.set(id, view);
        }
      });
      pending.clear();
      pendingComparator.clear();
    };

    const totalExpected = sats.length;
    void runPropagateTrajectories(req, {
      onStart: () => propagationStatus.markStart(totalExpected),
      onTrajectory: (msg) => {
        const view: SampledTrajectoryView = {
          epochsMs: new Float64Array(msg.epochsMs),
          eciKm: new Float64Array(msg.eciThreejsBufferKm),
          groundDeg: new Float64Array(msg.groundLatLonDeg),
        };
        if (msg.comparatorId) {
          pendingComparator.set(msg.scId, view);
        } else {
          pending.set(msg.scId, view);
        }
        propagationStatus.bump();
        if (!flushScheduled) {
          flushScheduled = true;
          requestAnimationFrame(flush);
        }
      },
      onDone: (ms) => {
        // Final flush in case any messages arrived after the last rAF.
        flush();
        propagationStatus.markDone(ms);
      },
      onCancel: () => propagationStatus.markCancelled(),
      onError: (err) => {
        propagationStatus.markError(err.message);
        console.error("trajectory propagation failed:", err);
      },
    }, ctl.signal);
  });
}
