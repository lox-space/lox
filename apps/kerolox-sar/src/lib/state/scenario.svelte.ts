// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export type LookSide = "LEFT" | "RIGHT";

export interface WalkerConfig {
  t: number;
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
}

export function defaultScenario(): Scenario {
  return {
    startTimeIso: "2026-06-01T00:00:00Z",
    durationHours: 24,
    walker: { t: 24, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 },
    sar: { lookSide: "RIGHT", minIncidenceDeg: 20, maxIncidenceDeg: 45 },
  };
}

export function walkerProductMatchesT(w: WalkerConfig): boolean {
  return w.p > 0 && w.t > 0 && w.t % w.p === 0;
}

export function isWalkerValid(w: WalkerConfig): boolean {
  return (
    Number.isInteger(w.t) &&
    Number.isInteger(w.p) &&
    Number.isInteger(w.f) &&
    walkerProductMatchesT(w) &&
    w.f >= 0 &&
    w.f < w.p &&
    w.altitudeKm > 0 &&
    Number.isFinite(w.inclinationDeg)
  );
}

// The single runes-state holder. Other modules import `scenario` and bind
// directly to its fields.
export const scenario = $state<Scenario>(defaultScenario());
