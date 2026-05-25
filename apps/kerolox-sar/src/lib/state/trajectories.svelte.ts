// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { SvelteMap } from "svelte/reactivity";
import { Keplerian, Origin, type SampledTrajectory } from "@lox-space/wasm";
import type { Scenario } from "./scenario.svelte";
import type { SatelliteElements } from "$lib/walker.svelte";

export interface SampledTrajectoryView {
  /** Unix epoch ms per sample. */
  epochsMs: Float64Array;
  /** Interleaved Three.js ECI positions (Y-up), km. */
  eciKm: Float64Array;
  /** Interleaved lat/lon, degrees. */
  groundDeg: Float64Array;
}

export const trajectoryById = new SvelteMap<string, SampledTrajectoryView>();
let currentHash: string | null = null;

export function resetTrajectories(): void {
  trajectoryById.clear();
  currentHash = null;
}

export function scenarioHash(s: Scenario, sats: SatelliteElements[]): string {
  const satParts = sats.map((x) =>
    `${x.plane}.${x.indexInPlane}.${x.smaM.toFixed(1)}.${x.ecc.toFixed(6)}.${x.incRad.toFixed(9)}.${x.raanRad.toFixed(9)}.${x.aopRad.toFixed(9)}.${x.trueAnomalyRad.toFixed(9)}`,
  );
  return `${s.startTimeIso}|${s.durationHours}|${satParts.join(";")}`;
}

/**
 * Ensure the trajectory cache reflects the current scenario + satellites.
 * Does nothing if the scenario hash hasn't changed.
 */
export async function ensureTrajectories(
  s: Scenario,
  sats: SatelliteElements[],
): Promise<void> {
  const hash = scenarioHash(s, sats);
  if (hash === currentHash && trajectoryById.size === sats.length) return;
  trajectoryById.clear();
  currentHash = hash;

  const earth = new Origin("Earth");
  try {
    for (const sat of sats) {
      const kep = new Keplerian(
        sat.smaM,
        sat.ecc,
        sat.incRad,
        sat.raanRad,
        sat.aopRad,
        sat.trueAnomalyRad,
        earth,
      );
      try {
        const sampled = kep.propagateSampled(
          s.startTimeIso,
          s.durationHours * 3600,
          30, // 30 s step
        ) as SampledTrajectory;
        const id = `p${sat.plane}-s${sat.indexInPlane}`;
        trajectoryById.set(id, {
          epochsMs: new Float64Array(sampled.epochsMs()),
          eciKm: new Float64Array(sampled.eciThreejsBufferKm()),
          groundDeg: new Float64Array(sampled.groundLatLonDeg()),
        });
      } finally {
        kep.free();
      }
    }
  } finally {
    earth.free();
  }
}
