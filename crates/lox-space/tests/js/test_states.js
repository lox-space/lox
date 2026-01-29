// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it } from 'node:test';
import { lox as bindings, approxEqual, loadEphemeris, assertVecClose } from './fixtures.js';

const { Frame, Origin, State, UTC } = bindings;

describe('state transforms', () => {
    it('converts a state to ground location', () => {
        const time = UTC.fromISO('2024-07-05T09:09:18.173').toScale('TAI');
        const position = [-5530.01774359, -3487.0895338, -1850.03476185];
        const velocity = [1.29534407, -5.02456882, 5.6391936];

        const state = new State(time, position, velocity, new Origin('Earth'), new Frame('ICRF'))
            .toFrame(new Frame('IAU_EARTH'));

        const expectedPosition = [-5740.259426667957, 3121.1360727954725, -1863.1826563318027];
        const expectedVelocity = [-3.53237875783652, -3.152377656863808, 5.642296713889555];

        assertVecClose(state.position(), expectedPosition, 1e-6);
        assertVecClose(state.velocity(), expectedVelocity, 1e-6);

        const ground = state.toGroundLocation();

        approxEqual(ground.longitude(), 2.643578045424445, 1e-9);
        approxEqual(ground.latitude(), -0.27944957125091063, 1e-9);
        approxEqual(ground.altitude(), 417.8524151150059, 1e-9);
    });

    it('changes origin with ephemeris data', async () => {
        const ephemeris = await loadEphemeris();

        const rVenus = [
            1.001977553295792e8,
            2.200234656010247e8,
            9.391473630346918e7,
        ];
        const vVenus = [-59.08617935009049, 22.682387107225292, 12.05029567478702];
        const r = [6068279.27, -1692843.94, -2516619.18].map((v) => v / 1e3);
        const v = [-660.415582, 5495.938726, -5303.093233].map((v) => v / 1e3);

        const rExp = r.map((value, idx) => value - rVenus[idx]);
        const vExp = v.map((value, idx) => value - vVenus[idx]);
        const tai = UTC.fromISO('2016-05-30T12:00:00.000').toScale('TAI');

        const sEarth = new State(tai, r, v);
        const sVenus = sEarth.toOrigin(new Origin('Venus'), ephemeris);

        const rAct = sVenus.position();
        const vAct = sVenus.velocity();

        assertVecClose(rAct, rExp, 1e-5);
        assertVecClose(vAct, vExp, 1e-5);
    });
});
