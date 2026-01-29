// SPDX-FileCopyrightText: 2026 Hadrien Develay <hadrien.develay@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { lox as bindings, assertVecClose } from "./fixtures.js";

const { Keplerian, State, TimeDelta, Times, UTC, Vallado, Trajectory } =
  bindings;

const assertCloseRel = (actual, expected, rel = 1e-8) => {
  const diff = Math.abs(actual - expected);
  const tol = Math.abs(expected) * rel;
  assert.ok(
    diff <= tol,
    `actual=${actual}, expected=${expected}, |diff|=${diff} > tol=${tol}`
  );
};

const assertCloseAbs = (actual, expected, abs = 1e-8) => {
  const diff = Math.abs(actual - expected);
  assert.ok(
    diff <= abs,
    `actual=${actual}, expected=${expected}, |diff|=${diff} > abs=${abs}`
  );
};

const buildOrbit = () => {
  const utc = new UTC(2023, 3, 25, 21, 8, 0.0);
  const time = utc.toScale("TDB");
  return new Keplerian(
    time,
    24464.56,
    0.7311,
    0.122138,
    1.00681,
    3.10686,
    0.44369564302687126
  );
};

const buildTrajectory = (orbit) => {
  const dt = orbit.orbitalPeriod();
  const totalSeconds = Math.ceil(dt.toDecimalSeconds());
  const start = orbit.time();
  const end = start.add(TimeDelta.fromSeconds(totalSeconds));
  const times = Times.generateTimes(start, end, TimeDelta.fromSeconds(1));
  const s0 = orbit.toCartesian();
  const prop = new Vallado(s0);
  return prop.propagate(times);
};

describe("trajectories", () => {
  it("builds trajectories from arrays", () => {
    const utc = new UTC(2023, 3, 25, 21, 8, 0.0);
    const time = utc.toScale("TDB");
    const states = [
      [0.0, 1e3, 1e3, 1e3, 1.0, 1.0, 1.0],
      [1.0, 2e3, 2e3, 2e3, 2.0, 2.0, 2.0],
      [2.0, 3e3, 3e3, 3e3, 3.0, 3.0, 3.0],
      [3.0, 4e3, 4e3, 4e3, 4.0, 4.0, 4.0],
    ];

    const jsStates = states.map(([dt, x, y, z, vx, vy, vz]) => {
      const delta = TimeDelta.fromSeconds(Math.trunc(dt));
      const t = time.add(delta);
      return new State(t, [x, y, z], [vx, vy, vz]);
    });

    const trajectory = new Trajectory(jsStates);
    const interpolated = trajectory.interpolateAt(time.add(new TimeDelta(1.5)));
    assertVecClose(interpolated.position(), [2.5e3, 2.5e3, 2.5e3], 1e-6);

    const roundTrip = trajectory.toArray();
    assert.equal(roundTrip.length, states.length);
    roundTrip.forEach((row, idx) => {
      assertVecClose(row, states[idx], 1e-12);
    });
  });

  it("interpolates orbital elements over a period", () => {
    const orbit = buildOrbit();
    const trajectory = buildTrajectory(orbit);
    const dt = orbit.orbitalPeriod();
    const s1 = trajectory.interpolateDelta(dt);
    const k1 = s1.toKeplerian();

    assertCloseRel(k1.semiMajorAxis(), orbit.semiMajorAxis(), 1e-8);
    assertCloseRel(k1.eccentricity(), orbit.eccentricity(), 1e-8);
    assertCloseRel(k1.inclination(), orbit.inclination(), 1e-8);
    assertCloseRel(
      k1.longitudeOfAscendingNode(),
      orbit.longitudeOfAscendingNode(),
      1e-8
    );
    assertCloseRel(k1.argumentOfPeriapsis(), orbit.argumentOfPeriapsis(), 1e-8);
    assertCloseRel(k1.trueAnomaly(), orbit.trueAnomaly(), 1e-8);
  });

  it("finds apsis events", () => {
    const orbit = buildOrbit();
    const trajectory = buildTrajectory(orbit);

    const events = trajectory.findEvents((state) => {
      const position = state.position();
      const velocity = state.velocity();
      return (
        position[0] * velocity[0] +
        position[1] * velocity[1] +
        position[2] * velocity[2]
      );
    });

    assert.equal(events.length, 2);

    const k1 = trajectory.interpolateAt(events[0].time()).toKeplerian();
    assertCloseRel(k1.trueAnomaly(), Math.PI, 1e-8);

    const k2 = trajectory.interpolateAt(events[1].time()).toKeplerian();
    assertCloseAbs(k2.trueAnomaly(), 0.0, 1e-8);
  });

  it("finds above-equator windows", () => {
    const orbit = buildOrbit();
    const trajectory = buildTrajectory(orbit);
    const windows = trajectory.findWindows((state) => state.position()[2]);
    assert.equal(windows.length, 1);
  });

  it("propagates errors from event callbacks", () => {
    const orbit = buildOrbit();
    const trajectory = buildTrajectory(orbit);
    assert.throws(() => {
      trajectory.findEvents(() => {
        throw new Error("boom in events");
      });
    }, /boom in events/);
  });

  it("propagates errors from window callbacks", () => {
    const orbit = buildOrbit();
    const trajectory = buildTrajectory(orbit);
    assert.throws(() => {
      trajectory.findWindows(() => {
        throw new Error("boom in windows");
      });
    }, /boom in windows/);
  });
});
