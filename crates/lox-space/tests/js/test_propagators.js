// SPDX-FileCopyrightText: 2026 Hadrien Develay <hadrien.develay@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it } from "node:test";
import { lox as bindings, deg2rad, approxEqual, assertVecClose } from './fixtures.js';

const { GroundLocation, GroundPropagator, Origin, SGP4, TimeDelta, UTC } = bindings;

describe("propagators", () => {
  it("computes SGP4 orbital period", () => {
    const issTle = `ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731`;

    const sgp4 = new SGP4(issTle);
    const t1 = sgp4.time().add(TimeDelta.fromMinutes(92.821));
    const s1 = sgp4.propagate_at(t1);
    const k1 = s1.toKeplerian();

    const actualPeriod = k1.orbitalPeriod().toDecimalSeconds();
    const expectedPeriod = 92.821 * 60;

    approxEqual(actualPeriod, expectedPeriod, 1e-4);
  });

  it("propagates ground location state", () => {
    const lat = deg2rad(40.4527);
    const lon = deg2rad(-4.3676);
    const tai = UTC.fromISO("2022-01-31T23:00:00").toScale("TAI");
    const loc = new GroundLocation(new Origin("Earth"), lon, lat, 0.0);
    const ground = new GroundPropagator(loc);

    const state = ground.propagateAt(tai);
    const position = state.position();
    const expected = [
      -1765.9535510583582, 4524.585984442561, 4120.189198495323,
    ];

    assertVecClose(position, expected, 1e-6);
  });
});
