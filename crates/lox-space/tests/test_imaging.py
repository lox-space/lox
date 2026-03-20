# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""Integration tests for AOI imaging analysis using Sentinel-2 TLEs."""

import pytest

import lox_space as lox

# ---------------------------------------------------------------------------
# Sentinel-2 TLEs
# ---------------------------------------------------------------------------

SENTINEL_2A_TLE = """\
SENTINEL-2A
1 40697U 15028A   26079.19377485 -.00000072  00000+0 -11026-4 0  9994
2 40697  98.5642 155.3327 0001269  98.1407 261.9920 14.30816376561005"""

SENTINEL_2B_TLE = """\
SENTINEL-2B
1 42063U 17013A   26079.18648189  .00000015  00000+0  22231-4 0  9995
2 42063  98.5694 155.2271 0001161  93.5553 266.5763 14.30810963471912"""

SENTINEL_2C_TLE = """\
SENTINEL-2C
1 60989U 24157A   26079.22152607  .00000102  00000+0  55488-4 0  9996
2 60989  98.5671 155.2832 0000963 112.7762 247.3522 14.30817874 80264"""

# Sentinel-2: 290 km swath, nadir-only imaging
SENTINEL2_PAYLOAD = lox.ImagingPayload.nadir_only(290.0 * lox.km)

# Western Europe — large AOI that any SSO satellite will overfly within hours
EUROPE_AOI = lox.Aoi(
    [
        (-10.0, 35.0),
        (20.0, 35.0),
        (20.0, 60.0),
        (-10.0, 60.0),
        (-10.0, 35.0),
    ]
)

# Small spot in the Pacific — unlikely to be hit in a short window
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
def s2a():
    return lox.SGP4(SENTINEL_2A_TLE)


@pytest.fixture(scope="module")
def s2b():
    return lox.SGP4(SENTINEL_2B_TLE)


@pytest.fixture(scope="module")
def s2c():
    return lox.SGP4(SENTINEL_2C_TLE)


@pytest.fixture(scope="module")
def t0(s2a):
    return s2a.time()


@pytest.fixture(scope="module")
def t1(t0):
    return t0 + 6 * lox.hours


@pytest.fixture(scope="module")
def scenario_single(t0, t1, s2a):
    sc = lox.Spacecraft("S2A", s2a, imaging_payload=SENTINEL2_PAYLOAD)
    return lox.Scenario(t0, t1, spacecraft=[sc])


@pytest.fixture(scope="module")
def scenario_constellation(t0, t1, s2a, s2b, s2c):
    scs = [
        lox.Spacecraft("S2A", s2a, imaging_payload=SENTINEL2_PAYLOAD),
        lox.Spacecraft("S2B", s2b, imaging_payload=SENTINEL2_PAYLOAD),
        lox.Spacecraft("S2C", s2c, imaging_payload=SENTINEL2_PAYLOAD),
    ]
    return lox.Scenario(t0, t1, spacecraft=scs)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestImagingPayload:
    def test_nadir_only(self):
        p = lox.ImagingPayload.nadir_only(20.0 * lox.km)
        assert repr(p) == "ImagingPayload(...)"

    def test_off_nadir(self):
        p = lox.ImagingPayload.off_nadir(20.0 * lox.km, 30.0 * lox.deg)
        assert repr(p) == "ImagingPayload(...)"

    def test_spacecraft_roundtrip(self, s2a):
        payload = lox.ImagingPayload.nadir_only(290.0 * lox.km)
        sc = lox.Spacecraft("test", s2a, imaging_payload=payload)
        assert sc.imaging_payload() is not None

    def test_spacecraft_no_payload(self, s2a):
        sc = lox.Spacecraft("test", s2a)
        assert sc.imaging_payload() is None


class TestAoi:
    def test_from_coords(self):
        aoi = lox.Aoi([(10, 45), (11, 45), (11, 46), (10, 46), (10, 45)])
        assert "5 vertices" in repr(aoi)

    def test_from_geojson(self):
        geojson = '{"type":"Polygon","coordinates":[[[10,45],[11,45],[11,46],[10,46],[10,45]]]}'
        aoi = lox.Aoi.from_geojson(geojson)
        assert "5 vertices" in repr(aoi)

    def test_from_geojson_invalid(self):
        with pytest.raises(ValueError):
            lox.Aoi.from_geojson("not json")


class TestImagingAnalysis:
    def test_single_spacecraft_over_europe(self, scenario_single):
        analysis = lox.ImagingAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        intervals = results.intervals("S2A", "europe")
        assert len(intervals) > 0, "S2A should image Western Europe within 6 hours"

    def test_interval_durations(self, scenario_single):
        analysis = lox.ImagingAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        for iv in results.intervals("S2A", "europe"):
            dur = float(iv.duration())
            assert dur > 0, "zero-length interval"
            assert dur < 600, f"imaging window too long ({dur:.0f}s)"

    def test_constellation_over_europe(self, scenario_constellation):
        analysis = lox.ImagingAnalysis(
            scenario_constellation,
            aois=[("europe", EUROPE_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        total = sum(
            len(results.intervals(sc_id, "europe")) for sc_id in ("S2A", "S2B", "S2C")
        )
        assert total > 0, "at least one Sentinel-2 should image Europe within 6 hours"
        # All three pairs should exist in results
        assert len(results.all_intervals()) == 3

    def test_multiple_aois(self, scenario_single):
        analysis = lox.ImagingAnalysis(
            scenario_single,
            aois=[("europe", EUROPE_AOI), ("pacific", PACIFIC_AOI)],
            step=30 * lox.seconds,
        )
        results = analysis.compute()
        all_ivs = results.all_intervals()
        # Should have entries for both AOIs
        assert len(all_ivs) == 2

    def test_no_payload_skips_spacecraft(self, t0, t1, s2a):
        sc = lox.Spacecraft("bare", s2a)  # no imaging_payload
        scenario = lox.Scenario(t0, t1, spacecraft=[sc])
        analysis = lox.ImagingAnalysis(
            scenario,
            aois=[("europe", EUROPE_AOI)],
        )
        results = analysis.compute()
        assert len(results.all_intervals()) == 0

    def test_off_nadir_wider_than_nadir(self, t0, t1, s2a):
        nadir = lox.ImagingPayload.nadir_only(290.0 * lox.km)
        off_nadir = lox.ImagingPayload.off_nadir(290.0 * lox.km, 30.0 * lox.deg)

        sc_nadir = lox.Spacecraft("nadir", s2a, imaging_payload=nadir)
        sc_off_nadir = lox.Spacecraft("off_nadir", s2a, imaging_payload=off_nadir)

        scenario_n = lox.Scenario(t0, t1, spacecraft=[sc_nadir])
        scenario_a = lox.Scenario(t0, t1, spacecraft=[sc_off_nadir])

        res_n = lox.ImagingAnalysis(
            scenario_n, aois=[("europe", EUROPE_AOI)], step=30 * lox.seconds
        ).compute()
        res_a = lox.ImagingAnalysis(
            scenario_a, aois=[("europe", EUROPE_AOI)], step=30 * lox.seconds
        ).compute()

        dur_nadir = sum(
            float(iv.duration()) for iv in res_n.intervals("nadir", "europe")
        )
        dur_off_nadir = sum(
            float(iv.duration()) for iv in res_a.intervals("off_nadir", "europe")
        )
        assert dur_off_nadir >= dur_nadir - 1.0, (
            f"off-nadir ({dur_off_nadir:.0f}s) should have >= nadir ({dur_nadir:.0f}s) coverage"
        )

    def test_repr(self, scenario_single):
        analysis = lox.ImagingAnalysis(scenario_single, aois=[("europe", EUROPE_AOI)])
        assert "1 spacecraft" in repr(analysis)
        assert "1 AOI)" in repr(analysis)

    def test_results_repr(self, scenario_single):
        results = lox.ImagingAnalysis(
            scenario_single, aois=[("europe", EUROPE_AOI)], step=30 * lox.seconds
        ).compute()
        assert "1 pair)" in repr(results)
