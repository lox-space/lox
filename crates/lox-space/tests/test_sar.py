# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""Integration tests for SAR access analysis using a Sentinel-1 TLE."""

import pytest

import lox_space as lox

# Sentinel-1A TLE — epoch 2026-079 (synthetic placeholder, matches the Rust
# integration test in lox-analysis/src/imaging/sar.rs).
SENTINEL_1A_TLE = """\
SENTINEL-1A
1 39634U 14016A   26079.20000000  .00000050  00000+0  37000-4 0  9991
2 39634  98.1817 105.0000 0001300  90.0000 270.0000 14.59197557600008"""

# Sentinel-1 IW mode: incidence ~29°–46°, right-looking.
SAR_PAYLOAD = lox.SarPayload.with_incidence_angles(
    29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
)

# Western Europe — large enough AOI that an SSO satellite will overfly within hours.
EUROPE_AOI = lox.Aoi(
    [(-10.0, 35.0), (20.0, 35.0), (20.0, 60.0), (-10.0, 60.0), (-10.0, 35.0)]
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def s1a():
    return lox.SGP4(SENTINEL_1A_TLE)


@pytest.fixture(scope="module")
def six_hour_window(s1a):
    t0 = s1a.time()
    return t0, t0 + 6 * lox.hours


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_sar_payload_constructor_look_angles():
    p = lox.SarPayload.with_look_angles(20.0 * lox.deg, 45.0 * lox.deg, lox.LookSide.Either)
    assert p.side() == lox.LookSide.Either


def test_sar_payload_constructor_incidence_angles():
    p = lox.SarPayload.with_incidence_angles(22.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
    assert p.side() == lox.LookSide.Right


def test_sar_payload_rejects_invalid_range():
    with pytest.raises(ValueError):
        lox.SarPayload.with_look_angles(45.0 * lox.deg, 20.0 * lox.deg, lox.LookSide.Right)


def test_sar_payload_rejects_out_of_range_angle():
    with pytest.raises(ValueError):
        lox.SarPayload.with_incidence_angles(20.0 * lox.deg, 90.0 * lox.deg, lox.LookSide.Right)


def test_sentinel1_over_europe(s1a, six_hour_window):
    t0, t1 = six_hour_window
    sc = lox.Spacecraft("s1a", s1a, sar_payload=SAR_PAYLOAD)
    scenario = lox.Scenario(t0, t1, spacecraft=[sc])
    analysis = lox.SarAccessAnalysis(
        scenario,
        aois=[("europe", EUROPE_AOI)],
        step=30 * lox.seconds,
    )
    results = analysis.compute()
    windows = results.intervals("s1a", "europe")
    assert len(windows) > 0, "Sentinel-1A should image Western Europe within 6 hours"
    for iv in windows:
        dur = float(iv.duration())
        assert dur > 0, "zero-length SAR window"
        assert dur < 600, f"SAR window too long ({dur:.0f}s)"
