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
  trajectoryById, comparatorTrajectoryById, ensureTrajectories, resetTrajectories, scenarioHash,
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
    ensureTrajectories(s, sats, false);
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
    ensureTrajectories(s, sats, false);
    await new Promise(r => setTimeout(r, 0));
    ensureTrajectories(s, sats, false);
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
    expect(scenarioHash(s, a, false)).not.toEqual(scenarioHash(s, b, false));
  });

  it("scenarioHash differs when the ICEYE comparison toggles", () => {
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    expect(scenarioHash(s, sats, false)).not.toEqual(scenarioHash(s, sats, true));
  });

  it("routes comparator trajectories into the comparator map", async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(runPropagateTrajectories).mockImplementationOnce((async (req: any, cb: any, signal: any) => {
      if (signal.aborted) { cb.onCancel(); return; }
      cb.onStart();
      cb.onTrajectory({
        scId: "p0-s0", comparatorId: "",
        epochsMs: [0, 30_000], eciThreejsBufferKm: [7000, 0, 0, 7000, 1, 0], groundLatLonDeg: [0, 0, 0.1, 0],
      });
      cb.onTrajectory({
        scId: "iceye/ICEYE-X2", comparatorId: "iceye",
        epochsMs: [0, 30_000], eciThreejsBufferKm: [7000, 0, 0, 7000, 1, 0], groundLatLonDeg: [0, 0, 0.1, 0],
      });
      cb.onDone(0);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    }) as any);
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    ensureTrajectories(s, sats, true);
    await new Promise(r => setTimeout(r, 0));
    expect(trajectoryById.has("p0-s0")).toBe(true);
    expect(comparatorTrajectoryById.has("iceye/ICEYE-X2")).toBe(true);
    expect(trajectoryById.has("iceye/ICEYE-X2")).toBe(false);
  });
});
