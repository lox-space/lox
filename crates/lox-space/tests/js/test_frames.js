// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import assert from 'node:assert/strict';
import { describe, it, before } from 'node:test';
import fs from 'node:fs';
import path from 'node:path';
import { lox as bindings } from './fixtures.js';

const {
  Time,
  State,
  Frame,
} = bindings;


const expected = JSON.parse(
  fs.readFileSync(
    path.join(path.dirname(new URL(import.meta.url).pathname), "./data/iau_frames_expected_frames.json"),
    "utf8"
  )
);
const frames = Object.keys(expected);

const assertVecClose = (a, b, atol = 1e-6) => {
  assert.equal(a.length, b.length);
  a.forEach((v, i) => {
    const d = Math.abs(v - b[i]);
    assert.ok(
      d <= atol,
      `mismatch at idx ${i}: actual=${v}, expected=${b[i]}, |diff|=${d} > atol=${atol}`
    );
  });
};

describe("IAU frame transforms", () => {
  for (const frame of frames) {
    it(`converts J2000 -> ${frame}`, () => {
      const t = new Time("TDB", 2000, 1, 1, 0, 0, 0);
      const r0 = [6068.27927, -1692.84394, -2516.61918];
      const v0 = [-0.660415582, 5.495938726, -5.303093233];

      const s0 = new State(t, r0, v0);
      const s1 = s0.toFrame(new Frame(frame));

      const r1Act = s1.position();
      const v1Act = s1.velocity();
      const r1Exp = expected[frame].r;
      const v1Exp = expected[frame].v;

      assertVecClose(r1Act, r1Exp);
      assertVecClose(v1Act, v1Exp, 1e-3);
    });
  }
});
