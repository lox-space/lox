// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import * as bindings from '../../pkg/lox_space.js';

// Bundler target auto-initializes wasm on import.
// After js_name renames, exports should be Angle/Distance/Frequency/Velocity.
const {
  Angle,
  Distance,
  Frequency,
  Velocity,
} = bindings;

describe('units', () => {
  it('angle from radians', () => {
    const a = Angle.radians(Math.PI);
    assert.equal(a.toString(), '180 deg');
    assert.equal(a.repr(), 'Angle(3.141592653589793)');
    assert.equal(a.rawValue(), Math.PI);
    assert.equal(a.toRadians(), Math.PI);
    assert.equal(a.asInt(), 3n);
  });

  it('angle from degrees', () => {
    const a = Angle.degrees(180);
    assert.equal(a.toString(), '180 deg');
    assert.equal(a.repr(), 'Angle(3.141592653589793)');
    assert.equal(a.rawValue(), Math.PI);
    assert.equal(a.toDegrees(), 180);
    assert.equal(a.asInt(), 3n);
  });

  it('distance from kilometers', () => {
    const d = Distance.kilometers(1024);
    assert.equal(d.toString(), '1024 km');
    assert.equal(d.repr(), 'Distance(1024000)');
    assert.equal(d.rawValue(), 1_024_000);
    assert.equal(d.toKilometers(), 1024);
    assert.equal(d.asInt(), 1_024_000n);
  });

  it('distance from meters', () => {
    const d = Distance.meters(2048);
    assert.equal(d.toString(), '2.048 km');
    assert.equal(d.repr(), 'Distance(2048)');
    assert.equal(d.rawValue(), 2048);
    assert.equal(d.toMeters(), 2048);
    assert.equal(d.asInt(), 2_048n);
  });

  it('frequency from hertz', () => {
    const f = Frequency.hertz(1_073_741_824);
    assert.equal(f.toString(), '1.073741824 GHz');
    assert.equal(f.repr(), 'Frequency(1073741824)');
    assert.equal(f.rawValue(), 1_073_741_824);
    assert.equal(f.toHertz(), 1_073_741_824);
    assert.equal(f.asInt(), 1_073_741_824n);
  });

  it('frequency from kilohertz', () => {
    const f = Frequency.kilohertz(2_000_000);
    assert.equal(f.toString(), '2 GHz');
    assert.equal(f.repr(), 'Frequency(2000000000)');
    assert.equal(f.rawValue(), 2_000_000_000);
    assert.equal(f.toKilohertz(), 2_000_000);
    assert.equal(f.asInt(), 2_000_000_000n);
  });

  it('velocity from m/s', () => {
    const v = Velocity.metersPerSecond(262_144);
    assert.equal(v.toString(), '262.144 km/s');
    assert.equal(v.repr(), 'Velocity(262144)');
    assert.equal(v.rawValue(), 262_144);
    assert.equal(v.toMetersPerSecond(), 262_144);
    assert.equal(v.asInt(), 262_144n);
  });

  it('velocity from km/s', () => {
    const v = Velocity.kilometersPerSecond(16);
    assert.equal(v.toString(), '16 km/s');
    assert.equal(v.repr(), 'Velocity(16000)');
    assert.equal(v.rawValue(), 16_000);
    assert.equal(v.toKilometersPerSecond(), 16);
    assert.equal(v.asInt(), 16_000n);
  });
});
