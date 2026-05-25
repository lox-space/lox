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

# Small spot in the Pacific — unlikely to be hit in a short window.
PACIFIC_AOI = lox.Aoi(
    [
        (-175.0, -5.0),
        (-174.0, -5.0),
        (-174.0, -4.0),
        (-175.0, -4.0),
        (-175.0, -5.0),
    ]
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


@pytest.fixture(scope="module")
def scenario_single(s1a, six_hour_window):
    t0, t1 = six_hour_window
    sc = lox.Spacecraft("s1a", s1a, sar_payload=SAR_PAYLOAD)
    return lox.Scenario(t0, t1, spacecraft=[sc])


# ---------------------------------------------------------------------------
# TestSarPayload
# ---------------------------------------------------------------------------


class TestSarPayload:
    def test_with_look_angles(self):
        p = lox.SarPayload.with_look_angles(20.0 * lox.deg, 45.0 * lox.deg, lox.LookSide.Either)
        assert p.side() == lox.LookSide.Either

    def test_with_incidence_angles(self):
        p = lox.SarPayload.with_incidence_angles(22.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
        assert p.side() == lox.LookSide.Right

    def test_repr(self):
        p = lox.SarPayload.with_incidence_angles(29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
        assert repr(p) == "SarPayload(...)"

    def test_side_accessor_roundtrip(self):
        for side in (lox.LookSide.Left, lox.LookSide.Right, lox.LookSide.Either):
            p = lox.SarPayload.with_look_angles(20.0 * lox.deg, 45.0 * lox.deg, side)
            assert p.side() == side

    def test_spacecraft_roundtrip(self, s1a):
        payload = lox.SarPayload.with_incidence_angles(
            29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
        )
        sc = lox.Spacecraft("test", s1a, sar_payload=payload)
        assert sc.sar_payload() is not None

    def test_spacecraft_no_payload(self, s1a):
        sc = lox.Spacecraft("test", s1a)
        assert sc.sar_payload() is None

    def test_spacecraft_carries_both_payloads(self, s1a):
        optical = lox.OpticalPayload.nadir_only(290.0 * lox.km)
        sar = lox.SarPayload.with_incidence_angles(
            29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
        )
        sc = lox.Spacecraft("dual", s1a, optical_payload=optical, sar_payload=sar)
        assert sc.optical_payload() is not None
        assert sc.sar_payload() is not None

    def test_rejects_invalid_range(self):
        with pytest.raises(ValueError):
            lox.SarPayload.with_look_angles(45.0 * lox.deg, 20.0 * lox.deg, lox.LookSide.Right)

    def test_rejects_out_of_range_angle(self):
        with pytest.raises(ValueError):
            lox.SarPayload.with_incidence_angles(20.0 * lox.deg, 90.0 * lox.deg, lox.LookSide.Right)


# ---------------------------------------------------------------------------
# TestLookSide
# ---------------------------------------------------------------------------


class TestLookSide:
    def test_variants(self):
        assert lox.LookSide.Left != lox.LookSide.Right
        assert lox.LookSide.Left != lox.LookSide.Either
        assert lox.LookSide.Right != lox.LookSide.Either

    def test_equality(self):
        assert lox.LookSide.Left == lox.LookSide.Left
        assert lox.LookSide.Right == lox.LookSide.Right
        assert lox.LookSide.Either == lox.LookSide.Either
        assert lox.LookSide.Left != lox.LookSide.Right


# ---------------------------------------------------------------------------
# TestSarAccessAnalysis
# ---------------------------------------------------------------------------


class TestSarAccessAnalysis:
    def test_single_spacecraft_over_europe(self, scenario_single):
        analysis = lox.SarAccessAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        windows = results.windows("s1a", "europe")
        assert len(windows) > 0, "Sentinel-1A should image Western Europe within 6 hours"

    def test_interval_durations(self, scenario_single):
        analysis = lox.SarAccessAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        for iv in results.windows("s1a", "europe"):
            dur = float(iv.interval().duration())
            assert dur > 0, "zero-length SAR window"
            assert dur < 600, f"SAR window too long ({dur:.0f}s)"

    def test_multiple_aois(self, scenario_single):
        analysis = lox.SarAccessAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI), ("pacific", PACIFIC_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        all_ivs = results.all_windows()
        # Should have entries for both AOIs (even if pacific has zero windows)
        assert len(all_ivs) == 2

    def test_no_payload_skips_spacecraft(self, s1a, six_hour_window):
        t0, t1 = six_hour_window
        sc = lox.Spacecraft("bare", s1a)  # no sar_payload
        scenario = lox.Scenario(t0, t1, spacecraft=[sc])
        analysis = lox.SarAccessAnalysis(
            scenario,
            aois=[("europe", EUROPE_AOI)],
        )
        results = analysis.compute()
        assert len(results.all_windows()) == 0

    def test_left_vs_right_side_differ(self, s1a, six_hour_window):
        t0, t1 = six_hour_window
        payload_left = lox.SarPayload.with_incidence_angles(
            29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Left
        )
        payload_right = lox.SarPayload.with_incidence_angles(
            29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
        )
        sc_l = lox.Spacecraft("s1a_l", s1a, sar_payload=payload_left)
        sc_r = lox.Spacecraft("s1a_r", s1a, sar_payload=payload_right)
        scenario = lox.Scenario(t0, t1, spacecraft=[sc_l, sc_r])
        analysis = lox.SarAccessAnalysis(
            scenario,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        windows_l = results.windows("s1a_l", "europe")
        windows_r = results.windows("s1a_r", "europe")
        assert len(windows_l) > 0, "Left-looking should have access over Europe"
        assert len(windows_r) > 0, "Right-looking should have access over Europe"

        # Sides should see different opportunities: at least one window on one
        # side must not overlap any window on the other. Robust to TLE refreshes
        # (unlike a sum-of-durations check, which can coincidentally match).
        def overlaps(a, b):
            return a.interval().start() < b.interval().end() and b.interval().start() < a.interval().end()

        left_has_unique = any(
            not any(overlaps(l, r) for r in windows_r) for l in windows_l
        )
        right_has_unique = any(
            not any(overlaps(r, l) for l in windows_l) for r in windows_r
        )
        assert left_has_unique or right_has_unique, (
            "Every Left window overlaps a Right window and vice versa — sides not differentiated"
        )

    def test_repr(self, scenario_single):
        analysis = lox.SarAccessAnalysis(scenario_single, aois=[("europe", EUROPE_AOI)])
        assert "spacecraft" in repr(analysis)
        assert "AOI" in repr(analysis)

    def test_results_repr(self, scenario_single):
        results = lox.SarAccessAnalysis(
            scenario_single, aois=[("europe", EUROPE_AOI)], step=30 * lox.seconds
        ).compute()
        assert "pair" in repr(results)

    def test_multiple_spacecraft(self, s1a, six_hour_window):
        t0, t1 = six_hour_window
        sc_a = lox.Spacecraft("s1a_a", s1a, sar_payload=SAR_PAYLOAD)
        sc_b = lox.Spacecraft("s1a_b", s1a, sar_payload=SAR_PAYLOAD)
        scenario = lox.Scenario(t0, t1, spacecraft=[sc_a, sc_b])
        analysis = lox.SarAccessAnalysis(
            scenario,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        all_ivs = results.all_windows()
        assert len(all_ivs) == 2, "Both spacecraft-AOI pairs should be present in results"

    def test_pass_direction_populated(self, scenario_single):
        analysis = lox.SarAccessAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        for window in results.windows("s1a", "europe"):
            d = window.direction()
            assert d in (lox.PassDirection.Ascending, lox.PassDirection.Descending), (
                f"unexpected direction: {d}"
            )

    def test_both_directions_observed(self, s1a):
        # LookSide.Either: a single spacecraft sees both sides → both directions
        # over Europe in a long-enough window. 6h was insufficient at the test
        # TLE's RAAN; 12h reliably catches both ascending and descending overflights.
        t0 = s1a.time()
        t1 = t0 + 12 * lox.hours
        payload = lox.SarPayload.with_incidence_angles(
            29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Either
        )
        sc = lox.Spacecraft("s1a", s1a, sar_payload=payload)
        scenario = lox.Scenario(t0, t1, spacecraft=[sc])
        analysis = lox.SarAccessAnalysis(
            scenario,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        directions = {w.direction() for w in results.windows("s1a", "europe")}
        assert lox.PassDirection.Ascending in directions, "missing ascending pass"
        assert lox.PassDirection.Descending in directions, "missing descending pass"
