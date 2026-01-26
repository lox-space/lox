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
    assert.equal(a.to_string_js(), '180 deg');
    assert.equal(a.repr(), 'Angle(3.141592653589793)');
    assert.equal(a.raw_value(), Math.PI);
    assert.equal(a.to_radians(), Math.PI);
    assert.equal(a.as_int(), 3n);
  });

  it('angle from degrees', () => {
    const a = Angle.degrees(180);
    assert.equal(a.to_string_js(), '180 deg');
    assert.equal(a.repr(), 'Angle(3.141592653589793)');
    assert.equal(a.raw_value(), Math.PI);
    assert.equal(a.to_degrees(), 180);
    assert.equal(a.as_int(), 3n);
  });

  it('distance from kilometers', () => {
    const d = Distance.kilometers(1024);
    assert.equal(d.toString(), '1024 km');
    assert.equal(d.repr(), 'Distance(1024000)');
    assert.equal(d.raw_value(), 1_024_000);
    assert.equal(d.to_kilometers(), 1024);
    assert.equal(d.as_int(), 1_024_000n);
  });

  it('distance from meters', () => {
    const d = Distance.meters(2048);
    assert.equal(d.toString(), '2.048 km');
    assert.equal(d.repr(), 'Distance(2048)');
    assert.equal(d.raw_value(), 2048);
    assert.equal(d.to_meters(), 2048);
    assert.equal(d.as_int(), 2_048n);
  });

  it('frequency from hertz', () => {
    const f = Frequency.hertz(1_073_741_824);
    assert.equal(f.toString(), '1.073741824 GHz');
    assert.equal(f.repr(), 'Frequency(1073741824)');
    assert.equal(f.raw_value(), 1_073_741_824);
    assert.equal(f.to_hertz(), 1_073_741_824);
    assert.equal(f.as_int(), 1_073_741_824n);
  });

  it('frequency from kilohertz', () => {
    const f = Frequency.kilohertz(2_000_000);
    assert.equal(f.toString(), '2 GHz');
    assert.equal(f.repr(), 'Frequency(2000000000)');
    assert.equal(f.raw_value(), 2_000_000_000);
    assert.equal(f.to_kilohertz(), 2_000_000);
    assert.equal(f.as_int(), 2_000_000_000n);
  });

  it('velocity from m/s', () => {
    const v = Velocity.meters_per_second(262_144);
    assert.equal(v.toString(), '262.144 km/s');
    assert.equal(v.repr(), 'Velocity(262144)');
    assert.equal(v.raw_value(), 262_144);
    assert.equal(v.to_meters_per_second(), 262_144);
    assert.equal(v.as_int(), 262_144n);
  });

  it('velocity from km/s', () => {
    const v = Velocity.kilometers_per_second(16);
    assert.equal(v.toString(), '16 km/s');
    assert.equal(v.repr(), 'Velocity(16000)');
    assert.equal(v.raw_value(), 16_000);
    assert.equal(v.to_kilometers_per_second(), 16);
    assert.equal(v.as_int(), 16_000n);
  });
});
