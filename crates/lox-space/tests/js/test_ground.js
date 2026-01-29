// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it } from 'node:test';
import { lox as bindings, deg2rad, approxEqual } from './fixtures.js';

const {
  GroundLocation,
  Origin,
  Time,
  State,
  Frame,
  ElevationMask,
} = bindings;

describe('Ground observables', () => {
  it('computes observables', () => {
    const longitude = deg2rad(-4);
    const latitude = deg2rad(41);
    const location = new GroundLocation(new Origin('Earth'), longitude, latitude, 0);

    const position = [3359.927, -2398.072, 5153.0];
    const velocity = [5.0657, 5.485, -0.744];
    const time = new Time('TDB', 2012n, 7, 1, 0, 0, 0);
    const state = new State(time, position, velocity, new Origin('Earth'), new Frame('IAU_EARTH'));

    const observables = location.observables(state);

    const expectedRange = 2707.7;
    const expectedRangeRate = -7.16;
    const expectedAzimuth = deg2rad(-53.418);
    const expectedElevation = deg2rad(-7.077);

    approxEqual(observables.range(), expectedRange, 1e-2);
    approxEqual(observables.rangeRate(), expectedRangeRate, 1e-2);
    approxEqual(observables.azimuth(), expectedAzimuth, 1e-2);
    approxEqual(observables.elevation(), expectedElevation, 1e-2);
  });

  it('computes elevation mask minimum', () => {
    const mask = ElevationMask.variable(
      [-Math.PI, 0.0, Math.PI],
      [0.0, 5.0, 0.0]
    );
    const minEl = mask.minElevation(Math.PI / 2);
    approxEqual(minEl, 2.5, 1e-12);
  });
});
