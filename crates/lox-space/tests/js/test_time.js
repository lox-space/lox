// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, before } from 'node:test';
import assert from 'node:assert/strict';
import { lox as bindings, approxEqual, loadEOPProvider } from './fixtures.js';

const {
    Time,
    TimeDelta,
    UTC,
    EOPProvider,
    EopParserError,
    EopProviderError,
} = bindings;

const assertTimeEqual = (a, b) => {
    assert.ok(a.isClose(b));
};

const assertTimeClose = (a, b, rel = 1e-9) => {
    assert.ok(a.isClose(b, rel));
};

describe('time', () => {
      /** @type {any} */
      let provider;


      before(async () => {
        provider = await loadEOPProvider();
      });

    it('handles time scale conversions and arithmetic', () => {
        const taiExp = new Time('TAI', 2000, 1, 1);
        let taiAct = Time.fromISO('2000-01-01T00:00:00.000 TAI');
        assertTimeEqual(taiExp, taiAct);

        taiAct = taiExp.toScale('TAI');
        assertTimeEqual(taiExp, taiAct);

        taiAct = taiExp.toScale('TCB').toScale('TAI');
        assertTimeClose(taiExp, taiAct);

        taiAct = taiExp.toScale('TCG').toScale('TAI');
        assertTimeClose(taiExp, taiAct);

        taiAct = taiExp.toScale('TDB').toScale('TAI');
        assertTimeClose(taiExp, taiAct);

        taiAct = taiExp.toScale('TT').toScale('TAI');
        assertTimeClose(taiExp, taiAct);

        taiAct = taiExp.toScale('UT1', provider).toScale('TAI', provider);
        assertTimeClose(taiExp, taiAct);

        const tai1 = new Time('TAI', 2000, 1, 1, 0, 0, 0.5);
        assert.ok(tai1.julianDate('jd', 'seconds') > taiExp.julianDate('jd', 'seconds'));
        const dt = new TimeDelta(0.5);
        assertTimeClose(taiExp.add(dt), tai1);
        assertTimeClose(tai1.subtractDelta(dt), taiExp);
        const diff = tai1.subtractTime(taiExp);
        approxEqual(diff.toDecimalSeconds(), dt.toDecimalSeconds(), 1e-15);
    });

    it('parses and converts UTC', () => {
        const utcExp = new UTC(2000, 1, 1);

        let utcAct = UTC.fromISO('2000-01-01T00:00:00.000');
        assertTimeEqual(utcExp, utcAct);

        utcAct = UTC.fromISO('2000-01-01T00:00:00.000Z');
        assertTimeEqual(utcExp, utcAct);

        utcAct = UTC.fromISO('2000-01-01T00:00:00.000 UTC');
        assertTimeEqual(utcExp, utcAct);

        utcAct = utcExp.toScale('TAI').toUtc();
        assertTimeEqual(utcExp, utcAct);

        utcAct = utcExp.toScale('TCB').toUtc();
        assertTimeClose(utcExp, utcAct);

        utcAct = utcExp.toScale('TCG').toUtc();
        assertTimeClose(utcExp, utcAct);

        utcAct = utcExp.toScale('TDB').toUtc();
        assertTimeClose(utcExp, utcAct);

        utcAct = utcExp.toScale('TT').toUtc();
        assertTimeEqual(utcExp, utcAct);

        utcAct = utcExp.toScaleWithProvider('UT1', provider).toUtcWithProvider(provider);
        assertTimeClose(utcExp, utcAct);
    });

    it('handles TimeDelta basics', () => {
        const delta = new TimeDelta(1.5);
        assert.equal(String(delta), '1.5 seconds');
        assert.equal(delta.toString(), '1.5 seconds'); // in case String calls toString
        assert.equal(delta.toString(), '1.5 seconds');

        assert.equal(delta.seconds(), 1);
        assert.equal(delta.subsecond(), 0.5);

        assert.equal(String(delta.add(delta)), '3 seconds');
        assert.equal(String(delta.subtract(delta)), '0 seconds');

        const neg = new TimeDelta(-1.5);
        assert.equal(String(neg), '-1.5 seconds');

        //assert.throws(() => TimeDelta.fromSeconds(Number.NaN).seconds(), { name: 'NonFiniteTimeDeltaError' });
        // fromSeconds does not accept floats, it is i64
    });

    it('constructs TimeDelta from various units', () => {
        let td = TimeDelta.fromSeconds(123);
        assert.equal(td.toDecimalSeconds(), 123);

        td = TimeDelta.fromMinutes(2);
        assert.equal(td.toDecimalSeconds(), 120);

        td = TimeDelta.fromHours(2);
        assert.equal(td.toDecimalSeconds(), 7200);

        td = TimeDelta.fromDays(2);
        assert.equal(td.toDecimalSeconds(), 172800);

        td = TimeDelta.fromJulianYears(2);
        assert.equal(td.toDecimalSeconds(), 63115200);

        td = TimeDelta.fromJulianCenturies(2);
        assert.equal(td.toDecimalSeconds(), 6311520000);
    });

    it('stringifies Time correctly', () => {
        const time = new Time('TAI', 2000, 1, 1, 0, 0, 12.123456789123);
        assert.equal(String(time), '2000-01-01T00:00:12.123 TAI');
        assert.equal(time.toString(), '2000-01-01T00:00:12.123 TAI');
    });

    it('exposes Time accessors', () => {
        const time = new Time('TAI', 2000, 1, 1, 0, 0, 12.123456789123);
        assert.equal(time.scale.abbreviation, 'TAI');
        assert.equal(time.year, 2000);
        assert.equal(time.month, 1);
        assert.equal(time.day, 1);
        assert.equal(time.hour, 0);
        assert.equal(time.minute, 0);
        assert.equal(time.second, 12);
        assert.equal(time.millisecond, 123);
        assert.equal(time.microsecond, 456);
        assert.equal(time.nanosecond, 789);
        assert.equal(time.picosecond, 123);
        approxEqual(time.decimalSeconds(), 12.123456789123, 1e-15);
    });

    it('rejects invalid dates and hours', () => {
        assert.throws(() => new Time('TAI', 2000, 13, 1), /invalid date/);
        assert.throws(() => new Time('TAI', 2000, 12, 1, 24, 0, 0), /hour must be in the range/);
    });

    it('disallows subtracting different time scales', () => {
        const t1 = new Time('TAI', 2000, 1, 1, 0, 0, 1.0);
        const t0 = new Time('TT', 2000, 1, 1, 0, 0, 1.0);
        assert.throws(() => t1.subtractTime(t0), /cannot subtract.*different time scales/i);
    });

    it('disallows isclose on different time scales', () => {
        const t0 = new Time('TAI', 2000, 1, 1);
        const t1 = new Time('TT', 2000, 1, 1);
        assert.throws(() => t0.isClose(t1), /cannot compare.*different time scales/i);
    });

    it('rejects invalid ISO strings and scales', () => {
        assert.throws(() => Time.fromISO('2000-01-01X00:00:00 TAI'), /invalid ISO/);
        assert.throws(() => Time.fromISO('2000-01-01T00:00:00 UTC'), /invalid ISO/);
        assert.throws(() => Time.fromISO('2000-01-01T00:00:00 TAI', 'UTC'), /unknown time scale: UTC/);
    });

    it('computes Julian dates', () => {
        let time = Time.fromJulianDate('TAI', 0.0, 'j2000');
        assert.equal(time.julianDate('j2000', 'seconds'), 0.0);
        assert.equal(time.julianDate('j2000', 'days'), 0.0);
        assert.equal(time.julianDate('j2000', 'centuries'), 0.0);
        assert.equal(time.julianDate('jd', 'days'), 2451545.0);
        assert.equal(time.julianDate('mjd', 'days'), 51544.5);
        assert.equal(time.julianDate('j1950', 'days'), 18262.5);

        time = Time.fromJulianDate('TAI', 0.0, 'j1950');
        assert.equal(time.julianDate('j1950', 'days'), 0.0);

        time = Time.fromJulianDate('TAI', 0.0, 'mjd');
        assert.equal(time.julianDate('mjd', 'days'), 0.0);

        time = Time.fromJulianDate('TAI', 0.0, 'jd');
        assert.equal(time.julianDate('jd', 'days'), 0.0);
    });

    it('rejects invalid epochs and units', () => {
        const time = new Time('TAI', 2000, 1, 1);
        assert.throws(() => time.julianDate('unknown', 'days'), /unknown epoch: unknown/);
        assert.throws(() => time.julianDate('jd', 'unknown'), /unknown unit: unknown/);
    });

    it('converts to/from two-part Julian dates', () => {
        const expected = new Time('TAI', 2024, 7, 11, 8, 2, 14.0);
        const [jd1, jd2] = expected.twoPartJulianDate();
        const actual = Time.fromTwoPartJulianDate('TAI', jd1, jd2);
        assertTimeClose(expected, actual);
    });

    it('converts from day-of-year', () => {
        const expected = new Time('TAI', 2024, 12, 31);
        const actual = Time.fromDayOfYear('TAI', 2024, 366);
        assertTimeEqual(actual, expected);
    });

    it('exposes UTC accessors', () => {
        const utc = new UTC(2000, 1, 1, 12, 13, 14.123456789123);
        assert.equal(utc.year, 2000);
        assert.equal(utc.month, 1);
        assert.equal(utc.day, 1);
        assert.equal(utc.hour, 12);
        assert.equal(utc.minute, 13);
        assert.equal(utc.second, 14);
        assert.equal(utc.millisecond, 123);
        assert.equal(utc.microsecond, 456);
        assert.equal(utc.nanosecond, 789);
        assert.equal(utc.picosecond, 123);
        approxEqual(utc.decimalSeconds(), 14.123456789123, 1e-15);
        assert.equal(String(utc), '2000-01-01T12:13:14.123 UTC');
        assert.equal(utc.debug(), 'UTC(2000, 1, 1, 12, 13, 14.123456789123)');
    });

    it('rejects invalid UTC inputs', () => {
        assert.throws(() => new UTC(2000, 0, 1), /invalid date/);
        assert.throws(() => UTC.fromISO('2000-01-01X00:00:00 UTC'), /invalid ISO/);
    });

    it('handles EOP provider errors', () => {
        assert.throws(() => new EOPProvider('invalid_path'), EopParserError);

        const provider = loadEOPProvider();
        const tai = new Time('TAI', 2100, 1, 1);
        assert.throws(() => tai.toScaleWithProvider('UT1', provider), EopProviderError);
    });
});
