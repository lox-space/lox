// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Scenario } from "./scenario.svelte";
import { defaultScenario } from "./scenario.svelte";

vi.mock("$lib/rpc/client", () => ({
  runPropagateTrajectories: vi.fn().mockImplementation(async (req, cb, signal) => {
    if (signal.aborted) { cb.onCancel(); return; }
    cb.onStart();
    // Simulate the engine streaming one trajectory per satellite.
    for (const sat of req.satellites) {
      cb.onTrajectory({
        scId: sat.id,
        epochsMs: [0, 30_000, 60_000],
        eciThreejsBufferKm: [7000, 0, 0,  7000, 1, 0,  7000, 2, 0],
        groundLatLonDeg: [0, 0,  0.1, 0,  0.2, 0],
      });
    }
    cb.onDone(0);
  }),
}));

import { runPropagateTrajectories } from "$lib/rpc/client";
import {
  trajectoryById, ensureTrajectories, resetTrajectories, scenarioHash,
} from "./trajectories.svelte";

beforeEach(() => {
  resetTrajectories();
  vi.mocked(runPropagateTrajectories).mockClear();
});

describe("ensureTrajectories", () => {
  it("populates the trajectory cache on first call", async () => {
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    ensureTrajectories(s, sats);
    // Yield to allow the async stream mock to flush.
    await new Promise(r => setTimeout(r, 0));
    expect(trajectoryById.size).toBe(1);
    const t = trajectoryById.get("p0-s0");
    expect(t?.epochsMs.length).toBe(3);
    expect(vi.mocked(runPropagateTrajectories)).toHaveBeenCalledTimes(1);
  });

  it("returns from cache on identical inputs", async () => {
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    ensureTrajectories(s, sats);
    await new Promise(r => setTimeout(r, 0));
    ensureTrajectories(s, sats);
    await new Promise(r => setTimeout(r, 0));
    expect(vi.mocked(runPropagateTrajectories)).toHaveBeenCalledTimes(1);
  });

  it("scenarioHash differs when satellites change", () => {
    const s: Scenario = defaultScenario();
    const a = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    const b = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.93, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    expect(scenarioHash(s, a)).not.toEqual(scenarioHash(s, b));
  });
});
