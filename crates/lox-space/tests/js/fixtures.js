// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import * as lox from '../../pkg/lox_space.js';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export { lox };

export const DATA_DIR = path.resolve(__dirname, '../../../../data');

export const deg2rad = (deg) => (deg * Math.PI) / 180;

export const approxEqual = (actual, expected, rel = 1e-2) => {
  const tol = Math.abs(expected) * rel;
  assert.ok(
    Math.abs(actual - expected) <= tol,
    `actual=${actual}, expected=${expected}, |diff|=${Math.abs(actual - expected)} > tol=${tol}`
  );
};

export const assertVecClose = (actual, expected, atol = 1e-6) => {
    assert.equal(actual.length, expected.length);
    actual.forEach((v, idx) => {
        const diff = Math.abs(v - expected[idx]);
        assert.ok(
            diff <= atol,
            `mismatch at idx ${idx}: actual=${v}, expected=${expected[idx]}, |diff|=${diff} > atol=${atol}`
        );
    });
};

export const assertVecRelClose = (actual, expected, rtol, atol = 0) => {
    if (actual.length !== expected.length) {
        throw new Error(`Vector length mismatch: got ${actual.length}, expected ${expected.length}`);
    }
    actual.forEach((v, i) => {
        const e = expected[i];
        const diff = Math.abs(v - e);
        const tol = atol + rtol * Math.abs(e);
        if (diff > tol) {
            throw new Error(`Vector mismatch at index ${i}: got ${v}, expected ${e}, diff ${diff}, tol ${tol}`);
        }
    });
};

export async function loadOneweb() {
  const txt = await readFile(path.join(DATA_DIR, 'oneweb_tle.txt'), 'utf8');
  const lines = txt.split(/\r?\n/).filter(Boolean);

  const t0 = new lox.SGP4(lines.slice(0, 3).join('\n')).time();
  const times = lox.Times.generateTimes(
    t0,
    t0.add(lox.TimeDelta.fromSeconds(86400n)),
    lox.TimeDelta.fromSeconds(60n)
  );

  const trajectories = [];
  for (let i = 0; i < lines.length; i += 3) {
    const tle = lines.slice(i, i + 3).join('\n');
    const name = lines[i].trim();
    const trajectory = new lox.SGP4(tle).propagate(times);
    trajectories.push([name, trajectory]);
  }
  return Object.fromEntries(trajectories);
}

export function loadEstrack() {
  const stations = [
    ['Kiruna', 67.858428, 20.966880],
    ['Esrange Space Center', 67.8833, 21.1833],
    ['Kourou', 5.2360, -52.7686],
    ['Redu', 50.00205516, 5.14518047],
    ['Cebreros', 40.3726, -4.4739],
    ['New Norcia', -30.9855, 116.2041],
  ];

  return stations.map(([name, lat, lon]) => {
    const origin = new lox.Origin('Earth');
    const loc = new lox.GroundLocation(origin, deg2rad(lon), deg2rad(lat), 0);
    const mask = lox.ElevationMask.fixed(0);
    return new lox.GroundStation(name, loc, mask); // adjust ctor if different
  });
}

export async function loadEphemeris() {
  const buf = await readFile(path.join(DATA_DIR, 'spice/de440s.bsp'));
  return lox.SPK.fromBytes(new Uint8Array(buf));
}

export async function loadEOPProvider() {
  const buf = await readFile(path.join(DATA_DIR, 'iers', 'finals2000A.all.csv'));
  return lox.EOPProvider.fromBytes(new Uint8Array(buf));
}
