// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { SvelteMap } from "svelte/reactivity";
import type { Scenario } from "./scenario.svelte";
import type { SatelliteElements } from "$lib/walker.svelte";
import { runPropagateTrajectories } from "$lib/rpc/client";
import type { PropagateRequest } from "@kerolox/proto-ts";

export interface SampledTrajectoryView {
  /** Unix epoch ms per sample. */
  epochsMs: Float64Array;
  /** Interleaved Three.js ECI positions (Y-up), km. */
  eciKm: Float64Array;
  /** Interleaved lat/lon, degrees. */
  groundDeg: Float64Array;
}

export const trajectoryById = new SvelteMap<string, SampledTrajectoryView>();
export const currentSatellites = $state<SatelliteElements[]>([]);

let currentAbort: AbortController | null = null;
let currentHash: string | null = null;

export function resetTrajectories(): void {
  trajectoryById.clear();
  currentSatellites.length = 0;
  currentHash = null;
  if (currentAbort) currentAbort.abort();
  currentAbort = null;
}

export function scenarioHash(s: Scenario, sats: SatelliteElements[]): string {
  const satParts = sats.map((x) =>
    `${x.plane}.${x.indexInPlane}.${x.smaM.toFixed(1)}.${x.ecc.toFixed(6)}.${x.incRad.toFixed(9)}.${x.raanRad.toFixed(9)}.${x.aopRad.toFixed(9)}.${x.trueAnomalyRad.toFixed(9)}`,
  );
  return `${s.startTimeIso}|${s.durationHours}|${satParts.join(";")}`;
}

/**
 * Stream trajectories for the current scenario from the engine. Cancels any
 * in-flight propagation; populates trajectoryById as messages arrive.
 *
 * This function is SYNCHRONOUS — it kicks off the stream and returns
 * immediately. Results arrive asynchronously via the trajectory cache.
 */
export function ensureTrajectories(s: Scenario, sats: SatelliteElements[]): void {
  const hash = scenarioHash(s, sats);
  if (hash === currentHash && trajectoryById.size === sats.length) return;
  if (currentAbort) currentAbort.abort();

  trajectoryById.clear();
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
  } as unknown as PropagateRequest;

  void runPropagateTrajectories(req, {
    onStart: () => {},
    onTrajectory: (msg) => {
      trajectoryById.set(msg.scId, {
        epochsMs: new Float64Array(msg.epochsMs),
        eciKm: new Float64Array(msg.eciThreejsBufferKm),
        groundDeg: new Float64Array(msg.groundLatLonDeg),
      });
    },
    onDone: () => {},
    onCancel: () => {},
    onError: (err) => {
      console.error("trajectory propagation failed:", err);
    },
  }, ctl.signal);
}
