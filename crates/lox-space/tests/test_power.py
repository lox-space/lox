# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math

import pytest

import lox_space as lox

ISS_TLE = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def iss_sgp4():
    return lox.SGP4(ISS_TLE)


@pytest.fixture(scope="module")
def t0(iss_sgp4):
    return iss_sgp4.time()


@pytest.fixture(scope="module")
def t1(t0):
    return t0 + lox.TimeDelta(86400)  # 24-hour window


@pytest.fixture(scope="module")
def scenario(t0, t1, iss_sgp4):
    sc = lox.Spacecraft("ISS", iss_sgp4)
    return lox.Scenario(t0, t1, spacecraft=[sc])


# ---------------------------------------------------------------------------
# Analytical (no ephemeris) path
# ---------------------------------------------------------------------------


class TestPowerBudgetAnalytical:
    @pytest.fixture(scope="class")
    def results(self, scenario):
        analysis = lox.PowerBudgetAnalysis(scenario)
        return analysis.compute()  # no ephemeris → analytical Sun

    def test_eclipse_intervals_exist(self, results):
        intervals = results.eclipse_intervals("ISS")
        # ISS in LEO should have multiple eclipses per day
        assert len(intervals) > 0

    def test_eclipse_fraction_in_range(self, results):
        frac = results.eclipse_fraction("ISS")
        assert frac is not None
        # ISS eclipse fraction should be roughly 30-40% but allow wider range
        assert 0.1 < frac < 0.6, f"unexpected eclipse fraction: {frac}"

    def test_sunlit_fraction_complement(self, results):
        eclipse = results.eclipse_fraction("ISS")
        sunlit = results.sunlit_fraction("ISS")
        assert eclipse is not None
        assert sunlit is not None
        assert abs(eclipse + sunlit - 1.0) < 1e-10

    def test_beta_angles(self, results):
        ts = results.beta_angles("ISS")
        assert ts is not None
        assert len(ts.times()) > 0
        assert len(ts.values()) == len(ts.times())
        for a in ts.values():
            assert -math.pi / 2 <= a <= math.pi / 2

    def test_beta_angles_interpolation(self, results):
        ts = results.beta_angles("ISS")
        assert ts is not None
        times = ts.times()
        # Interpolate at first sample — should match first value
        assert abs(ts.interpolate(times[0]) - ts.values()[0]) < 1e-10

    def test_solar_flux(self, results):
        ts = results.solar_flux("ISS")
        assert ts is not None
        assert len(ts.times()) > 0
        assert len(ts.values()) == len(ts.times())
        for f in ts.values():
            # Solar flux near Earth should be ~1361 W/m²
            assert 1300 < f < 1420, f"unexpected solar flux: {f}"

    def test_unknown_spacecraft(self, results):
        assert results.eclipse_intervals("NONEXISTENT") == []
        assert results.eclipse_fraction("NONEXISTENT") is None
        assert results.sunlit_fraction("NONEXISTENT") is None
        assert results.beta_angles("NONEXISTENT") is None
        assert results.solar_flux("NONEXISTENT") is None


# ---------------------------------------------------------------------------
# SPK ephemeris path
# ---------------------------------------------------------------------------


class TestPowerBudgetSPK:
    @pytest.fixture(scope="class")
    def results(self, scenario, ephemeris):
        analysis = lox.PowerBudgetAnalysis(scenario)
        return analysis.compute(ephemeris)

    def test_eclipse_intervals_exist(self, results):
        intervals = results.eclipse_intervals("ISS")
        assert len(intervals) > 0

    def test_eclipse_fraction_in_range(self, results):
        frac = results.eclipse_fraction("ISS")
        assert frac is not None
        assert 0.1 < frac < 0.6

    def test_beta_angles(self, results):
        ts = results.beta_angles("ISS")
        assert ts is not None
        assert len(ts.times()) > 0

    def test_solar_flux(self, results):
        ts = results.solar_flux("ISS")
        assert ts is not None
        assert len(ts.times()) > 0


# ---------------------------------------------------------------------------
# Spacecraft filtering
# ---------------------------------------------------------------------------


class TestPowerBudgetFiltering:
    def test_spacecraft_ids_filter(self, t0, t1, iss_sgp4):
        sc1 = lox.Spacecraft("ISS", iss_sgp4)
        sc2 = lox.Spacecraft("OTHER", iss_sgp4)
        scenario = lox.Scenario(t0, t1, spacecraft=[sc1, sc2])

        # Only analyse ISS
        analysis = lox.PowerBudgetAnalysis(scenario, spacecraft_ids=["ISS"])
        results = analysis.compute()

        # Should have results for ISS
        assert results.eclipse_fraction("ISS") is not None
        # Should NOT have results for OTHER
        assert results.eclipse_fraction("OTHER") is None

    def test_constellation_id_filter(self, t0, t1, iss_sgp4):
        sc1 = lox.Spacecraft("SAT-1", iss_sgp4, constellation_id="mysat")
        sc2 = lox.Spacecraft("SAT-2", iss_sgp4, constellation_id="mysat")
        sc3 = lox.Spacecraft("CUSTOMER", iss_sgp4, constellation_id="other")
        scenario = lox.Scenario(t0, t1, spacecraft=[sc1, sc2, sc3])

        analysis = lox.PowerBudgetAnalysis(
            scenario, constellation_id="mysat"
        )
        results = analysis.compute()

        assert results.eclipse_fraction("SAT-1") is not None
        assert results.eclipse_fraction("SAT-2") is not None
        assert results.eclipse_fraction("CUSTOMER") is None

    def test_mutually_exclusive_filters(self, scenario):
        with pytest.raises(ValueError, match="mutually exclusive"):
            lox.PowerBudgetAnalysis(
                scenario,
                spacecraft_ids=["ISS"],
                constellation_id="foo",
            )


# ---------------------------------------------------------------------------
# Custom step size
# ---------------------------------------------------------------------------


class TestPowerBudgetCustomStep:
    def test_custom_step(self, scenario):
        analysis = lox.PowerBudgetAnalysis(
            scenario, step=lox.TimeDelta(120)
        )
        results = analysis.compute()
        # Should still produce valid results
        ts = results.beta_angles("ISS")
        assert ts is not None
        times = ts.times()
        # With a 120s step over 86400s, expect ~720 samples
        assert 600 < len(times) < 800


# ---------------------------------------------------------------------------
# Repr
# ---------------------------------------------------------------------------


class TestPowerBudgetRepr:
    def test_analysis_repr(self, scenario):
        analysis = lox.PowerBudgetAnalysis(scenario)
        assert "PowerBudgetAnalysis" in repr(analysis)

    def test_results_repr(self, scenario):
        results = lox.PowerBudgetAnalysis(scenario).compute()
        assert "PowerBudgetResults" in repr(results)
