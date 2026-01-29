// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0


import assert from 'node:assert/strict';
import { describe, it, before } from 'node:test';
import { lox, loadOnewebSlim, loadEstrack, loadEphemeris } from './fixtures.js';

describe('visibility benchmark', () => {
  /** @type {Record<string, any>} */
  let estrack;
  /** @type {Record<string, any>} */
  let oneweb;
  /** @type {any} */
  let ephemeris;
  /** @type {any} */
  let ensemble;
  /** @type {any[]} */
  let times;

  before(async () => {
    estrack = loadEstrack();
    ephemeris = await loadEphemeris();
    oneweb = await loadOnewebSlim();
    const firstSc = Object.values(oneweb)[0];
    const t0 = firstSc.states()[0].time();
    const names = Object.keys(oneweb);
    const trajectories = Object.values(oneweb);
    ensemble = new lox.Ensemble(names, trajectories);
    times = lox.Times.generateTimes(
      t0,
      t0.add(lox.TimeDelta.fromSeconds(86400)),
      lox.TimeDelta.fromSeconds(60)
    );
  });

it('computes visibility for all passes', () => {
    const passes = lox.visibilityAll(times, estrack, ensemble, ephemeris, [new lox.Origin("Earth")]);
    assert.strictEqual(Object.keys(passes).length, Object.keys(oneweb).length);
    const estrackCount = Object.keys(estrack).length;

    for (const scPasses of Object.values(passes)) {
        assert.strictEqual(Object.keys(scPasses).length, estrackCount);
    }
  });
});
