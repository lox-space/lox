// SPDX-FileCopyrightText: 2026 Hadrien Develay <hadrien.develay@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it } from "node:test";
import { lox, deg2rad, approxEqual, assertVecClose, loadEOPProvider } from './fixtures.js';

const { GroundLocation, GroundPropagator, Origin, SGP4, TimeDelta, UTC } = lox;

describe("propagators", () => {
  it("computes SGP4 orbital period", () => {
    const issTle = `ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731`;

    const sgp4 = new SGP4(issTle);
    const t1 = sgp4.time.add(TimeDelta.fromMinutes(92.821));
    const s1 = sgp4.propagate_at(t1, new lox.EopConfiguration());
    const k1 = s1.toKeplerian();

    const actualPeriod = k1.orbitalPeriod().toDecimalSeconds();
    const expectedPeriod = 92.821 * 60;

    approxEqual(actualPeriod, expectedPeriod, 1e-4);
  });

  it("Propagates SGP4 states to UT1 times with EOP provider", async () => {
    const tle = `ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731`;

    const eopProvider = await loadEOPProvider();
    const eopProviderConfig = new lox.EopConfiguration();
    eopProviderConfig.set_eop_provider(eopProvider);

    const sgp4 = new SGP4(tle);
    const tTai = UTC.fromISO("2024-06-18T12:00:00").toScale("TAI");
    const tUt1 = tTai.toScale("UT1", eopProvider);

    const state = sgp4.propagate_at(tUt1, new lox.EopConfiguration());
    const position = state.position;
    const expected = [
      -2183.7175807963053, -5563.486798643622, 2173.201809201659,
    ];

    assertVecClose(position, expected, 1e-6);
  });

  it("propagates ground location state", () => {
    const lat = deg2rad(40.4527);
    const lon = deg2rad(-4.3676);
    const tai = UTC.fromISO("2022-01-31T23:00:00").toScale("TAI");
    const loc = new GroundLocation(new Origin("Earth"), lon, lat, 0.0);
    const ground = new GroundPropagator(loc);

    const state = ground.propagateAt(tai);
    const position = state.position;
    const expected = [
      -1765.9535510583582, 4524.585984442561, 4120.189198495323,
    ];

    assertVecClose(position, expected, 1e-6);
  });
});
