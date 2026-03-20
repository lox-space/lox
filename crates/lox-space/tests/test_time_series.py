# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import pytest


def test_time_series_basic():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 60 * lox.seconds, epoch + 120 * lox.seconds]
    ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])

    # Interpolate at midpoint
    t_mid = epoch + 30 * lox.seconds
    assert abs(ts.interpolate(t_mid) - 1.5) < 1e-12

    # Interpolate at data points
    assert abs(ts.interpolate(epoch) - 1.0) < 1e-12
    assert abs(ts.interpolate(epoch + 60 * lox.seconds) - 2.0) < 1e-12
    assert abs(ts.interpolate(epoch + 120 * lox.seconds) - 3.0) < 1e-12


def test_time_series_epoch():
    epoch = lox.Time("TAI", 2024, 6, 15)
    times = [epoch, epoch + 1 * lox.seconds]
    ts = lox.TimeSeries(times, [10.0, 20.0])
    assert ts.epoch == epoch


def test_time_series_times():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 60 * lox.seconds, epoch + 120 * lox.seconds]
    ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])
    result = ts.times()
    assert len(result) == 3
    assert result[0] == epoch
    assert result[1] == epoch + 60 * lox.seconds
    assert result[2] == epoch + 120 * lox.seconds


def test_time_series_values():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 1 * lox.seconds]
    ts = lox.TimeSeries(times, [5.0, 10.0])
    assert ts.values() == [5.0, 10.0]


def test_time_series_first_last():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 100 * lox.seconds, epoch + 200 * lox.seconds]
    ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])

    first_time, first_val = ts.first()
    assert first_time == epoch
    assert first_val == 1.0

    last_time, last_val = ts.last()
    assert last_time == epoch + 200 * lox.seconds
    assert last_val == 3.0


def test_time_series_cubic():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch + i * lox.seconds for i in range(5)]
    y = [0.0, 1.0, 4.0, 9.0, 16.0]
    ts = lox.TimeSeries(times, y, interpolation="cubic")

    # Cubic spline should give good results for quadratic data
    t_mid = epoch + 2.5 * lox.seconds
    val = ts.interpolate(t_mid)
    assert abs(val - 6.25) < 0.5  # Approximate for spline


def test_time_series_from_offsets():
    epoch = lox.Time("TAI", 2024, 1, 1)
    ts = lox.TimeSeries.from_offsets(epoch, [0.0, 60.0, 120.0], [1.0, 2.0, 3.0])

    assert ts.epoch == epoch
    assert abs(ts.interpolate(epoch + 30 * lox.seconds) - 1.5) < 1e-12


def test_time_series_empty_times():
    with pytest.raises(ValueError, match="times must not be empty"):
        lox.TimeSeries([], [])


def test_time_series_unknown_interpolation():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 1 * lox.seconds]
    with pytest.raises(ValueError, match="unknown interpolation type"):
        lox.TimeSeries(times, [1.0, 2.0], interpolation="quadratic")


def test_time_series_from_offsets_unknown_interpolation():
    epoch = lox.Time("TAI", 2024, 1, 1)
    with pytest.raises(ValueError, match="unknown interpolation type"):
        lox.TimeSeries.from_offsets(
            epoch, [0.0, 1.0], [1.0, 2.0], interpolation="spline"
        )


def test_series_unknown_interpolation():
    with pytest.raises(ValueError, match="unknown interpolation type"):
        lox.Series([0.0, 1.0], [1.0, 2.0], interpolation="quadratic")


def test_time_series_repr():
    epoch = lox.Time("TAI", 2024, 1, 1)
    times = [epoch, epoch + 60 * lox.seconds, epoch + 120 * lox.seconds]
    ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])
    r = repr(ts)
    assert "TimeSeries" in r
    assert "3 points" in r
