// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the rpc client: each ComputeAccess yields 1 user pair per AOI with a
// single 60 s window, so the metric is deterministic.
const calls: unknown[] = [];
vi.mock("$lib/rpc/client", () => ({
  runComputeAccess: vi.fn().mockImplementation(async (req, cb, signal) => {
    calls.push(req);
    if (signal.aborted) { cb.onCancel(); return; }
    cb.onStart();
    for (const aoi of ["hormuz", "black_sea"]) {
      cb.onPair({
        scId: "p0-s0", aoiId: aoi, source: 1, comparatorId: "",
        windows: [{ startIso: "2026-06-01T00:00:00Z", endIso: "2026-06-01T00:01:00Z", direction: 1 }],
      });
    }
    cb.onDone(1);
  }),
}));

// Mock the walker so the WASM binary is not needed in the test environment.
vi.mock("$lib/walker.svelte", () => ({
  runWalker: vi.fn().mockReturnValue([
    { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.9250245, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
  ]),
}));

import { runSweep, sweepPoints, resetSweep, type SweepConfig } from "./sweep.svelte";
import { defaultScenario } from "./scenario.svelte";

beforeEach(() => {
  resetSweep();
  calls.length = 0;
});

describe("runSweep", () => {
  it("produces one point per AOI per sweep value", async () => {
    const cfg: SweepConfig = { param: "satsPerPlane", min: 2, max: 4, step: 1, metric: "meanGap", concurrency: 2 };
    await runSweep(defaultScenario(), cfg, new AbortController().signal);
    // values 2,3,4 → 3 scenarios; each yields hormuz + black_sea points.
    expect(sweepPoints.get("hormuz")?.length).toBe(3);
    expect(sweepPoints.get("black_sea")?.length).toBe(3);
  });

  it("fires one ComputeAccess per sweep value", async () => {
    const cfg: SweepConfig = { param: "satsPerPlane", min: 2, max: 4, step: 1, metric: "count", concurrency: 2 };
    await runSweep(defaultScenario(), cfg, new AbortController().signal);
    expect(calls.length).toBe(3);
  });

  it("stops early when aborted", async () => {
    const ctl = new AbortController();
    ctl.abort();
    const cfg: SweepConfig = { param: "planes", min: 1, max: 10, step: 1, metric: "count", concurrency: 2 };
    await runSweep(defaultScenario(), cfg, ctl.signal);
    expect(calls.length).toBe(0);
  });
});
