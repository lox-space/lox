// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect } from "vitest";
import {
  type Scenario,
  defaultScenario,
  isWalkerValid,
  walkerProductMatchesT,
} from "./scenario.svelte";

describe("scenario defaults", () => {
  it("provides a sensible default scenario", () => {
    const s = defaultScenario();
    expect(s.startTimeIso).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
    expect(s.durationHours).toBeGreaterThan(0);
    expect(s.walker.t).toBeGreaterThan(0);
    expect(s.walker.p).toBeGreaterThan(0);
    expect(s.walker.altitudeKm).toBeGreaterThan(0);
    expect(s.sar.lookSide).toMatch(/^(LEFT|RIGHT)$/);
  });
});

describe("walker validation", () => {
  it("requires T divisible by P", () => {
    const s: Scenario = { ...defaultScenario(), walker: { t: 23, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 } };
    expect(walkerProductMatchesT(s.walker)).toBe(false);
  });

  it("accepts T divisible by P", () => {
    const s: Scenario = { ...defaultScenario(), walker: { t: 24, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 } };
    expect(walkerProductMatchesT(s.walker)).toBe(true);
  });

  it("requires F in [0, P)", () => {
    const s: Scenario = { ...defaultScenario(), walker: { t: 24, p: 3, f: 3, altitudeKm: 600, inclinationDeg: 53 } };
    expect(isWalkerValid(s.walker)).toBe(false);
  });

  it("requires positive altitude", () => {
    const s: Scenario = { ...defaultScenario(), walker: { t: 24, p: 3, f: 1, altitudeKm: 0, inclinationDeg: 53 } };
    expect(isWalkerValid(s.walker)).toBe(false);
  });

  it("accepts a valid Walker config", () => {
    const s: Scenario = { ...defaultScenario(), walker: { t: 24, p: 3, f: 1, altitudeKm: 600, inclinationDeg: 53 } };
    expect(isWalkerValid(s.walker)).toBe(true);
  });
});
