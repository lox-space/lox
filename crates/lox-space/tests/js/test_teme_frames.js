// SPDX-FileCopyrightText: 2025 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

/**
 * Test TEME frame transformations.
 *
 * Reference data from GitHub issue #197:
 * https://github.com/lox-space/lox/issues/197
 *
 * this is a port of test/test_teme_frames.py, it is mostly here for exposing surface area of the wasm bindings.
 */

import { describe, it } from 'node:test';
import { lox as bindings, assertVecRelClose } from './fixtures.js';

const { Frame, Origin, State, UTC } = bindings;

describe('TEME frame transformations', () => {
    it('Test TEME <-> ICRF transformation roundtrip preserves state.', () => {
        const time = UTC.fromISO('2025-01-27T00:00:00').toScale('TAI');
        const position = [-40755.396, -10823.119, 12.227];
        const velocity = [0.789, -2.971, 0.0];

        const stateICRF = new State(time, position, velocity, new Origin('Earth'), new Frame('ICRF'));

        const stateTEME = stateICRF.toFrame(new Frame('TEME'));
        const stateICRFBack = stateTEME.toFrame(new Frame('ICRF'));

        assertVecRelClose(stateICRF.position, stateICRFBack.position, 1e-10);
        assertVecRelClose(stateICRF.velocity, stateICRFBack.velocity, 1e-10, 1e-15);
    });

    it('Test that TEME differs from PEF by a small z-axis rotation (Equation of Equinoxes).', () => {
        const time = UTC.fromISO('2025-01-27T00:00:00').toScale('TAI');

        const position = [42164.0, 0.0, 0.0];
        const velocity = [0.0, 3.075, 0.0];

        const stateICRF = new State(time, position, velocity, new Origin('Earth'), new Frame('ICRF'));
        const stateTEME = stateICRF.toFrame(new Frame('TEME'));

        const normICRF = Math.hypot(...stateICRF.position);
        const normTEME = Math.hypot(...stateTEME.position);

        const diff = Math.abs(normICRF - normTEME);
        const tol = 1e-12 * Math.abs(normICRF);
        if (diff > tol) {
            throw new Error(`Norm mismatch: got ${normTEME}, expected ${normICRF}, diff ${diff}, tol ${tol}`);
        }
    });

    it('Test that TEME frame transformations are implemented (not todo!).', () => {
        const time = UTC.fromISO('2025-01-27T00:00:00').toScale('TAI');
        const position = [42164.0, 0.0, 0.0];
        const velocity = [0.0, 3.075, 0.0];

        const stateICRF = new State(time, position, velocity, new Origin('Earth'), new Frame('ICRF'));

        const stateTEME = stateICRF.toFrame(new Frame('TEME'));
        if (!stateTEME) throw new Error('TEME transform returned null/undefined');

        const stateICRFBack = stateTEME.toFrame(new Frame('ICRF'));
        if (!stateICRFBack) throw new Error('ICRF transform after TEME returned null/undefined');
    });
});
