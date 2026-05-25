// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect } from "vitest";
import { computeStats, type AccessWindowLite } from "./access.svelte";

const scenarioStart = new Date("2026-06-01T00:00:00Z").getTime();
const scenarioEnd = scenarioStart + 24 * 3600 * 1000;

function mkWindow(startOffsetSec: number, endOffsetSec: number): AccessWindowLite {
  return {
    scId: "p0-s0",
    startMs: scenarioStart + startOffsetSec * 1000,
    endMs: scenarioStart + endOffsetSec * 1000,
    direction: "ascending",
  };
}

describe("computeStats", () => {
  it("returns zeros for empty window list", () => {
    const s = computeStats([], scenarioStart, scenarioEnd);
    expect(s.count).toBe(0);
    expect(s.totalAccessSeconds).toBe(0);
    expect(s.meanGapSeconds).toBeNull();
    expect(s.medianGapSeconds).toBeNull();
    expect(s.maxGapSeconds).toBeNull();
  });

  it("computes count and total access for non-empty list", () => {
    const wins = [mkWindow(0, 60), mkWindow(120, 180)];
    const s = computeStats(wins, scenarioStart, scenarioEnd);
    expect(s.count).toBe(2);
    expect(s.totalAccessSeconds).toBe(120);
  });

  it("computes gap stats correctly", () => {
    const wins = [mkWindow(0, 60), mkWindow(120, 180), mkWindow(360, 420)];
    const s = computeStats(wins, scenarioStart, scenarioEnd);
    expect(s.meanGapSeconds).toBeCloseTo(120, 6);
    expect(s.medianGapSeconds).toBeCloseTo(120, 6);
    expect(s.maxGapSeconds).toBeCloseTo(180, 6);
  });

  it("sorts windows by start time before computing gaps", () => {
    const wins = [mkWindow(120, 180), mkWindow(0, 60)];
    const s = computeStats(wins, scenarioStart, scenarioEnd);
    expect(s.maxGapSeconds).toBeCloseTo(60, 6);
  });
});
