// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Scenario } from "./state/scenario.svelte";

// Mock the WASM binding before importing the module under test.
vi.mock("@lox-space/wasm", async () => {
  const sat = (plane: number, idx: number) => ({
    plane: () => plane,
    indexInPlane: () => idx,
    smaMeters: () => 6_978_137.0,
    ecc: () => 0,
    incRad: () => (53 * Math.PI) / 180,
    raanRad: () => (plane * 2 * Math.PI) / 3,
    aopRad: () => 0,
    trueAnomalyRad: () => (idx * 2 * Math.PI) / 8,
  });
  const buildArr: unknown[] = [];
  for (let p = 0; p < 3; p++) {
    for (let i = 0; i < 8; i++) buildArr.push(sat(p, i));
  }
  return {
    default: vi.fn().mockResolvedValue(undefined),
    WalkerDelta: {
      build: vi.fn().mockReturnValue(buildArr),
    },
  };
});

import { runWalker, type SatelliteElements } from "./walker.svelte";
import { defaultScenario } from "./state/scenario.svelte";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("runWalker", () => {
  it("calls WalkerDelta.build with meters and radians", async () => {
    const wasm = await import("@lox-space/wasm");
    const s: Scenario = defaultScenario();
    runWalker(s);
    const expectedSmaM = (s.walker.altitudeKm + 6378.137) * 1000;
    expect(wasm.WalkerDelta.build).toHaveBeenCalledWith(
      s.walker.t,
      s.walker.p,
      s.walker.f,
      expect.closeTo(expectedSmaM, 1),
      0,
      expect.closeTo((s.walker.inclinationDeg * Math.PI) / 180, 1e-12),
    );
  });

  it("returns N=T satellites with correct plane/index counts", () => {
    const s = defaultScenario();
    const out = runWalker(s) as SatelliteElements[];
    expect(out.length).toBe(s.walker.t);
    const byPlane = new Map<number, number>();
    for (const sat of out) byPlane.set(sat.plane, (byPlane.get(sat.plane) ?? 0) + 1);
    expect(byPlane.size).toBe(s.walker.p);
    for (const [, count] of byPlane) expect(count).toBe(s.walker.t / s.walker.p);
  });

  it("returns SI units (meters and radians) in the projected elements", () => {
    const out = runWalker(defaultScenario()) as SatelliteElements[];
    expect(out[0].smaM).toBeGreaterThan(6_000_000);
    expect(out[0].smaM).toBeLessThan(50_000_000);
    expect(Math.abs(out[0].incRad)).toBeLessThan(Math.PI);
  });

  it("returns [] when the walker config is invalid", () => {
    const s = { ...defaultScenario(), walker: { t: 23, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 } };
    const out = runWalker(s);
    expect(out).toEqual([]);
  });
});
