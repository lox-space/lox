// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import init, { WalkerDelta } from "@lox-space/wasm";
import { isWalkerValid, type Scenario } from "./state/scenario.svelte";

// Earth mean equatorial radius, kilometres (matches lox-bodies value).
const EARTH_MEAN_RADIUS_KM = 6378.137;

export interface SatelliteElements {
  plane: number;
  indexInPlane: number;
  smaM: number;
  ecc: number;
  incRad: number;
  raanRad: number;
  aopRad: number;
  trueAnomalyRad: number;
}

let initPromise: Promise<unknown> | null = null;
async function ensureWasm(): Promise<void> {
  if (!initPromise) initPromise = init();
  await initPromise;
}

// Eager init at module load so the synchronous `runWalker` works once it's
// been awaited once at app startup.
void ensureWasm();

/**
 * Synchronous Walker-delta evaluation. Returns [] if the config is invalid
 * (e.g. T not divisible by P). Throws if the WASM module hasn't initialised
 * yet — call `await ensureWalkerReady()` once at app startup.
 */
export function runWalker(s: Scenario): SatelliteElements[] {
  if (!isWalkerValid(s.walker)) return [];
  const smaM = (s.walker.altitudeKm + EARTH_MEAN_RADIUS_KM) * 1000;
  const incRad = (s.walker.inclinationDeg * Math.PI) / 180;
  const arr = WalkerDelta.build(s.walker.t, s.walker.p, s.walker.f, smaM, 0, incRad);
  const out: SatelliteElements[] = [];
  for (const obj of arr as unknown[]) {
    const sat = obj as {
      plane(): number;
      indexInPlane(): number;
      smaMeters(): number;
      ecc(): number;
      incRad(): number;
      raanRad(): number;
      aopRad(): number;
      trueAnomalyRad(): number;
    };
    out.push({
      plane: sat.plane(),
      indexInPlane: sat.indexInPlane(),
      smaM: sat.smaMeters(),
      ecc: sat.ecc(),
      incRad: sat.incRad(),
      raanRad: sat.raanRad(),
      aopRad: sat.aopRad(),
      trueAnomalyRad: sat.trueAnomalyRad(),
    });
  }
  return out;
}

/**
 * Awaits the one-shot WASM module initialisation. Call this once at app
 * startup (e.g. from `+page.svelte`'s `onMount`) before any `runWalker`
 * call to ensure the WASM module has loaded. Subsequent calls are no-ops.
 */
export async function ensureWalkerReady(): Promise<void> {
  await ensureWasm();
}
