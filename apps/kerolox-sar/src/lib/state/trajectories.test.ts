// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Scenario } from "./scenario.svelte";
import { defaultScenario } from "./scenario.svelte";

const mockSample = {
  len: () => 3,
  epochsMs: () => new Float64Array([0, 30_000, 60_000]),
  eciThreejsBufferKm: () => new Float64Array([7000, 0, 0,  7000, 1, 0,  7000, 2, 0]),
  groundLatLonDeg: () => new Float64Array([0, 0,  0.1, 0,  0.2, 0]),
};

const propagateSampledMock = vi.fn().mockReturnValue(mockSample);

vi.mock("@lox-space/wasm", async () => {
  return {
    default: vi.fn().mockResolvedValue(undefined),
    Origin: class {
      free(): void {}
      mean_radius(): number { return 6_371_000; }
    },
    Keplerian: class {
      free(): void {}
      propagateSampled = propagateSampledMock;
    },
  };
});

import {
  trajectoryById, ensureTrajectories, resetTrajectories, scenarioHash,
} from "./trajectories.svelte";

beforeEach(() => {
  resetTrajectories();
  propagateSampledMock.mockClear();
});

describe("ensureTrajectories", () => {
  it("populates the trajectory cache on first call", async () => {
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    await ensureTrajectories(s, sats);
    expect(trajectoryById.size).toBe(1);
    const t = trajectoryById.get("p0-s0");
    expect(t?.epochsMs.length).toBe(3);
    expect(propagateSampledMock).toHaveBeenCalledTimes(1);
  });

  it("returns from cache on identical inputs", async () => {
    const s: Scenario = defaultScenario();
    const sats = [
      { plane: 0, indexInPlane: 0, smaM: 6_978_137, ecc: 0, incRad: 0.92, raanRad: 0, aopRad: 0, trueAnomalyRad: 0 },
    ];
    await ensureTrajectories(s, sats);
    await ensureTrajectories(s, sats);
    expect(propagateSampledMock).toHaveBeenCalledTimes(1);
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
