// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export type LookSide = "LEFT" | "RIGHT";

/**
 * Walker delta constellation configuration.
 *
 * Uses the standard Walker notation (T/P/F), where T = satsPerPlane × p:
 * - `satsPerPlane` — satellites per orbital plane (user-specified, ≥ 1).
 *   Total satellites T is derived as `satsPerPlane * p` and is not stored.
 * - `p` — number of orbital planes (≥ 1).
 * - `f` — phasing parameter, integer in `[0, p)`. Sets the relative
 *   true-anomaly offset between satellites in adjacent planes.
 * - `altitudeKm` — circular-orbit altitude above Earth's mean radius, km.
 * - `inclinationDeg` — orbital inclination, degrees.
 *
 * All numeric fields must be JavaScript `number`s, not strings. Forms
 * binding `<input type="number">` via Svelte's `bind:value` coerce
 * automatically; other input paths must coerce before assignment.
 */
export interface WalkerConfig {
  satsPerPlane: number;
  p: number;
  f: number;
  altitudeKm: number;
  inclinationDeg: number;
}

export interface SarConfig {
  lookSide: LookSide;
  minIncidenceDeg: number;
  maxIncidenceDeg: number;
}

export interface Scenario {
  startTimeIso: string;
  durationHours: number;
  walker: WalkerConfig;
  sar: SarConfig;
  /** When true, the fielded ICEYE constellation is run through the same
   *  access analysis for side-by-side comparison. */
  compareIceye: boolean;
}

export function defaultScenario(): Scenario {
  return {
    startTimeIso: "2026-06-01T00:00:00Z",
    durationHours: 6,
    walker: { satsPerPlane: 8, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 },
    sar: { lookSide: "RIGHT", minIncidenceDeg: 20, maxIncidenceDeg: 45 },
    compareIceye: false,
  };
}

/** Total satellites in the Walker design (derived from satsPerPlane × p). */
export function totalSats(w: WalkerConfig): number {
  return w.satsPerPlane * w.p;
}

/**
 * Returns `true` iff the Walker config is a valid input for
 * `WalkerDeltaBuilder` in the engine and WASM binding.
 *
 * Precondition: all numeric fields are JS `number`s (callers responsible
 * for coercion — see {@link WalkerConfig}).
 */
export function isWalkerValid(w: WalkerConfig): boolean {
  return (
    Number.isInteger(w.satsPerPlane) &&
    Number.isInteger(w.p) &&
    Number.isInteger(w.f) &&
    w.satsPerPlane > 0 &&
    w.p > 0 &&
    w.f >= 0 &&
    w.f < w.p &&
    w.altitudeKm > 0 &&
    Number.isFinite(w.inclinationDeg)
  );
}

// The single runes-state holder. Other modules import `scenario` and bind
// directly to its fields.
export const scenario = $state<Scenario>(defaultScenario());
