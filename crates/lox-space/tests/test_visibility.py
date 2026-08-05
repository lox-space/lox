# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math

import numpy as np
import pytest

import lox_space as lox


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def oneweb_subset(oneweb):
    """First 10 OneWeb satellites — enough for pair tests, fast to run."""
    items = list(oneweb.items())[:10]
    return dict(items)


@pytest.fixture(scope="module")
def space_assets(oneweb_subset):
    return [lox.Spacecraft(name, sgp4) for name, sgp4 in oneweb_subset.items()]


@pytest.fixture(scope="module")
def ground_assets(estrack):
    return estrack


@pytest.fixture(scope="module")
def t0(oneweb_subset):
    sgp4 = next(iter(oneweb_subset.values()))
    return sgp4.time()


@pytest.fixture(scope="module")
def t1(t0):
    return t0 + lox.TimeDelta(86400)  # 24-hour window


@pytest.fixture(scope="module")
def scenario(t0, t1, space_assets, ground_assets):
    return lox.Scenario(t0, t1, spacecraft=space_assets, ground_stations=ground_assets)


@pytest.fixture(scope="module")
def analysis(scenario, ephemeris):
    return lox.VisibilityAnalysis(scenario, ephemeris=ephemeris)


@pytest.fixture(scope="module")
def run(analysis):
    return analysis.run()


@pytest.fixture(scope="module")
def analysis_with_los(scenario, ephemeris):
    return lox.VisibilityAnalysis(
        scenario, ephemeris=ephemeris, occulting_bodies=[lox.Origin("Earth")]
    )


@pytest.fixture(scope="module")
def run_with_los(analysis_with_los):
    return analysis_with_los.run()


@pytest.fixture(scope="module")
def run_no_eph(scenario):
    """No occulters, so no ephemeris is needed at all."""
    return lox.VisibilityAnalysis(scenario).run()


@pytest.fixture(scope="module")
def inter_satellite_run(t0, t1, space_assets, ephemeris):
    scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
    return lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris).run()


@pytest.fixture(scope="module")
def first_pair(ground_assets, space_assets):
    return (ground_assets[0].id(), space_assets[0].id())


@pytest.fixture(scope="module")
def passes(run, first_pair):
    return run.passes[first_pair]


@pytest.fixture(scope="module")
def windows(analysis, ground_assets, space_assets):
    return analysis.windows(ground_assets[0], space_assets[0])


@pytest.fixture(scope="module")
def intervals(passes):
    return [p.interval() for p in passes]


# ---------------------------------------------------------------------------
# VisibilityAnalysis construction & compute
# ---------------------------------------------------------------------------


class TestVisibilityAnalysis:
    def test_basic(self, run, ground_assets, space_assets):
        assert len(run.passes) == len(ground_assets) * len(space_assets)
        assert not run.errors

    def test_with_occulting_bodies(self, run_with_los, ground_assets, space_assets):
        assert len(run_with_los.passes) == len(ground_assets) * len(space_assets)

    def test_run_without_ephemeris(self, run_no_eph, run):
        """With no occulters the ephemeris is never consulted, so omitting it
        must give the same result as supplying one."""
        assert len(run_no_eph.passes) == len(run.passes)
        total = lambda r: sum(len(v) for v in r.passes.values())
        assert total(run_no_eph) == total(run)

    def test_construction_raises_when_ephemeris_missing_but_required(self, scenario):
        with pytest.raises(
            ValueError, match="ephemeris is required when occulting_bodies"
        ):
            lox.VisibilityAnalysis(scenario, occulting_bodies=[lox.Origin("Moon")])

    def test_with_custom_step(self, scenario, ephemeris, ground_assets, space_assets):
        run = lox.VisibilityAnalysis(
            scenario, ephemeris=ephemeris, step=lox.TimeDelta(30)
        ).run()
        assert len(run.passes) == len(ground_assets) * len(space_assets)

    def test_with_min_pass_duration(
        self, scenario, ephemeris, ground_assets, space_assets
    ):
        run = lox.VisibilityAnalysis(
            scenario, ephemeris=ephemeris, min_pass_duration=lox.TimeDelta(300)
        ).run()
        assert len(run.passes) == len(ground_assets) * len(space_assets)
        for passes in run.passes.values():
            for p in passes:
                assert p.interval().duration().to_decimal_seconds() >= 300

    def test_min_pass_duration_discards_short_passes(self, scenario, ephemeris):
        """The old implementation only coarsened the scan on this knob; the
        pipeline also filters, so a large threshold must remove passes."""
        unfiltered = lox.VisibilityAnalysis(scenario, ephemeris=ephemeris).run()
        filtered = lox.VisibilityAnalysis(
            scenario, ephemeris=ephemeris, min_pass_duration=lox.TimeDelta(7200)
        ).run()
        assert sum(len(v) for v in filtered.passes.values()) < sum(
            len(v) for v in unfiltered.passes.values()
        )

    def test_single_matches_run(self, analysis, run, ground_assets, space_assets):
        gs, sc = ground_assets[0], space_assets[0]
        single = analysis.single(gs, sc)
        assert len(single) == len(run.passes[(gs.id(), sc.id())])

    def test_windows_are_cheaper_but_agree_with_passes(self, windows, passes):
        """Windows carry no observables, but bound the same intervals."""
        assert len(windows) >= len(passes)
        by_start = {str(w.start) for w in windows}
        for p in passes:
            assert str(p.interval().start()) in by_start

    def test_sequential_and_parallel_agree(self, analysis):
        seq = analysis.run(parallel=False)
        par = analysis.run(parallel=True)
        local = analysis.run(parallel=True, workers=2)
        counts = lambda r: {k: len(v) for k, v in r.passes.items()}
        assert counts(seq) == counts(par) == counts(local)

    def test_missing_trajectory_raises(self, scenario, ground_assets, space_assets):
        """A spacecraft absent from the ensemble is an error, not a panic."""
        ensemble = scenario.propagate()
        lone = lox.Scenario(
            scenario.start(),
            scenario.end(),
            spacecraft=[space_assets[0]],
            ground_stations=[ground_assets[0]],
        )
        # An ensemble built from a different scenario is missing this spacecraft.
        empty_scenario = lox.Scenario(
            scenario.start(), scenario.end(), ground_stations=[ground_assets[0]]
        )
        analysis = lox.VisibilityAnalysis(lone, ensemble=empty_scenario.propagate())
        with pytest.raises(lox.AnalysisError, match=space_assets[0].id()):
            analysis.single(ground_assets[0], space_assets[0])
        del ensemble

    def test_repr(self, scenario):
        analysis = lox.VisibilityAnalysis(scenario)
        assert "VisibilityAnalysis" in repr(analysis)
        assert "ground stations" in repr(analysis)

    def test_run_repr(self, run):
        assert "VisibilityRun" in repr(run)
        assert "passes" in repr(run)

    def test_los_is_subset_of_basic(
        self, analysis, analysis_with_los, ground_assets, space_assets
    ):
        """Adding an occulter can only remove visibility, never add it."""
        gs, sc = ground_assets[0], space_assets[0]
        basic = sum(
            w.duration.to_decimal_seconds() for w in analysis.windows(gs, sc)
        )
        with_los = sum(
            w.duration.to_decimal_seconds() for w in analysis_with_los.windows(gs, sc)
        )
        assert with_los <= basic + 1e-6


class TestInterSatelliteAnalysis:
    def test_pairs_are_unordered_and_complete(self, inter_satellite_run, space_assets):
        n = len(space_assets)
        assert len(inter_satellite_run.windows) == n * (n - 1) // 2
        assert not inter_satellite_run.errors

    def test_keys_are_spacecraft_ids(self, inter_satellite_run, space_assets):
        ids = {sc.id() for sc in space_assets}
        for a, b in inter_satellite_run.windows:
            assert a in ids and b in ids
            assert a != b

    def test_single_matches_run(self, t0, t1, space_assets, ephemeris):
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        analysis = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris)
        run = analysis.run()
        a, b = space_assets[0], space_assets[1]
        single = analysis.single(a, b)
        assert len(single) == len(run.windows[(a.id(), b.id())])

    def test_max_range_only_removes_windows(self, t0, t1, space_assets, ephemeris):
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris).run()
        limited = lox.InterSatelliteAnalysis(
            scenario, ephemeris=ephemeris, max_range=5000.0 * lox.km
        ).run()
        for key, windows in limited.windows.items():
            total = lambda ws: sum(w.duration.to_decimal_seconds() for w in ws)
            assert total(windows) <= total(unlimited.windows[key]) + 1e-6

    def test_windows_have_positive_duration(self, inter_satellite_run):
        for windows in inter_satellite_run.windows.values():
            for w in windows:
                assert w.duration.to_decimal_seconds() > 0
                assert w.start < w.end

    def test_window_repr(self, inter_satellite_run):
        windows = next(
            ws for ws in inter_satellite_run.windows.values() if ws
        )
        assert "Window" in repr(windows[0])

    def test_repr(self, t0, t1, space_assets):
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        assert "InterSatelliteAnalysis" in repr(
            lox.InterSatelliteAnalysis(scenario)
        )

    def test_run_repr(self, inter_satellite_run):
        assert "InterSatelliteRun" in repr(inter_satellite_run)


class TestInterval:
    def _first_interval(self, intervals):
        return intervals[0]

    def test_start_end_duration(self, intervals, passes, t0, t1):
        w = self._first_interval(intervals)
        start = w.start()
        end = w.end()
        duration = w.duration()
        assert isinstance(start, lox.Time)
        assert isinstance(end, lox.Time)
        assert isinstance(duration, lox.TimeDelta)
        assert float(duration) > 0

    def test_repr(self, intervals, passes):
        w = self._first_interval(intervals)
        r = repr(w)
        assert r.startswith("Interval(")
        assert ")" in r

    def test_is_empty(self, intervals, passes):
        w = self._first_interval(intervals)
        assert not w.is_empty()
        # Reversed interval is empty
        empty = lox.Interval(w.end(), w.start())
        assert empty.is_empty()

    def test_contains_time(self, intervals, passes, t0):
        w = self._first_interval(intervals)
        mid = w.start() + lox.TimeDelta(float(w.duration()) / 2.0)
        assert w.contains_time(mid)
        before = w.start() - lox.TimeDelta(86400)
        assert not w.contains_time(before)

    def test_contains(self, intervals, passes):
        w = self._first_interval(intervals)
        assert w.contains(w)

    def test_intersect(self, intervals, passes):
        w = self._first_interval(intervals)
        # Self-intersection equals self
        inter = w.intersect(w)
        assert float(inter.duration()) == pytest.approx(float(w.duration()))
        # Intersection with non-overlapping is empty
        far = lox.Interval(
            w.end() + lox.TimeDelta(86400),
            w.end() + lox.TimeDelta(2 * 86400),
        )
        assert w.intersect(far).is_empty()

    def test_overlaps(self, intervals, passes):
        w = self._first_interval(intervals)
        assert w.overlaps(w)
        far = lox.Interval(
            w.end() + lox.TimeDelta(86400),
            w.end() + lox.TimeDelta(2 * 86400),
        )
        assert not w.overlaps(far)

    def test_step_by(self, intervals, passes):
        w = self._first_interval(intervals)
        step = lox.TimeDelta(60)
        times = w.step_by(step)
        assert all(isinstance(t, lox.Time) for t in times)
        expected_count = int(float(w.duration()) / 60) + 1
        assert abs(len(times) - expected_count) <= 1

    def test_linspace(self, intervals, passes):
        w = self._first_interval(intervals)
        times = w.linspace(5)
        assert len(times) == 5
        assert all(isinstance(t, lox.Time) for t in times)

    def test_step_by_zero_raises(self, intervals, passes):
        w = self._first_interval(intervals)
        with pytest.raises(ValueError):
            w.step_by(lox.TimeDelta(0))

    def test_linspace_one_raises(self, intervals, passes):
        w = self._first_interval(intervals)
        with pytest.raises(ValueError):
            w.linspace(1)


# ---------------------------------------------------------------------------
# Interval set operations
# ---------------------------------------------------------------------------


class TestPass:
    def _first_pass_with_gs(self, passes, ground_assets):
        return passes[0], ground_assets[0]

    def _first_pass(self, passes):
        return passes[0]

    def test_interval(self, intervals, passes):
        p = self._first_pass(passes)
        w = p.interval()
        assert isinstance(w, lox.Interval)
        assert float(w.duration()) > 0

    def test_times(self, intervals, passes):
        p = self._first_pass(passes)
        times = p.times()
        assert len(times) >= 2
        assert all(isinstance(t, lox.Time) for t in times)

    def test_observables(self, intervals, passes):
        p = self._first_pass(passes)
        obs_list = p.observables()
        assert len(obs_list) == len(p.times())
        for obs in obs_list:
            assert isinstance(obs, lox.Observables)
            assert float(obs.range()) > 0
            assert -math.pi <= float(obs.azimuth()) <= math.pi

    def test_observables_above_mask(self, passes, ground_assets):
        """All observables in a pass should be above the elevation mask."""
        p, gs = self._first_pass_with_gs(passes, ground_assets)
        mask = gs.mask()
        for obs in p.observables():
            min_elev = mask.min_elevation(obs.azimuth())
            assert float(obs.elevation()) >= float(min_elev)

    def test_interpolate_within_pass(self, intervals, passes):
        p = self._first_pass(passes)
        mid = p.times()[len(p.times()) // 2]
        obs = p.interpolate(mid)
        assert obs is not None
        assert isinstance(obs, lox.Observables)
        assert float(obs.range()) > 0

    def test_interpolate_outside_pass(self, intervals, passes, t0):
        p = self._first_pass(passes)
        # Well before the pass
        before = t0 - lox.TimeDelta(86400)
        assert p.interpolate(before) is None

    def test_repr(self, intervals, passes):
        p = self._first_pass(passes)
        r = repr(p)
        assert "Pass(" in r
        assert "observables" in r


# ---------------------------------------------------------------------------
# Asset accessors
# ---------------------------------------------------------------------------


class TestAssets:
    def test_ground_station_id(self, ground_assets):
        for ga in ground_assets:
            assert isinstance(ga.id(), str)
            assert len(ga.id()) > 0

    def test_ground_station_location(self, ground_assets):
        for ga in ground_assets:
            loc = ga.location()
            assert isinstance(loc, lox.GroundLocation)

    def test_ground_station_mask(self, ground_assets):
        for ga in ground_assets:
            mask = ga.mask()
            assert isinstance(mask, lox.ElevationMask)

    def test_ground_station_body_fixed_frame_default(self, ground_assets):
        """Default body-fixed frame should be IAU_EARTH."""
        ga = ground_assets[0]
        frame = ga.body_fixed_frame()
        assert isinstance(frame, lox.Frame)
        assert repr(frame) == 'Frame("IAU_EARTH")'

    def test_ground_station_body_fixed_frame_custom(self):
        """Custom body-fixed frame should be preserved."""
        loc = lox.GroundLocation(
            origin=lox.Origin("Earth"),
            longitude=0 * lox.deg,
            latitude=0 * lox.deg,
            altitude=0 * lox.km,
        )
        mask = lox.ElevationMask.fixed(0 * lox.deg)
        itrf = lox.Frame("ITRF")
        gs = lox.GroundStation("test", loc, mask, body_fixed_frame=itrf)
        assert repr(gs.body_fixed_frame()) == 'Frame("ITRF")'

    def test_ground_station_repr(self, ground_assets):
        r = repr(ground_assets[0])
        assert "GroundStation(" in r

    def test_spacecraft_id(self, space_assets):
        for sa in space_assets:
            assert isinstance(sa.id(), str)
            assert len(sa.id()) > 0

    def test_spacecraft_repr(self, space_assets):
        r = repr(space_assets[0])
        assert "Spacecraft(" in r

    def test_spacecraft_max_slew_rate_none(self, space_assets):
        assert space_assets[0].max_slew_rate() is None

    def test_spacecraft_max_slew_rate_set(self, oneweb_subset):
        sgp4 = next(iter(oneweb_subset.values()))
        sc = lox.Spacecraft("test", sgp4, max_slew_rate=5 * lox.deg_per_s)
        assert sc.max_slew_rate() is not None

    def test_ground_station_network_id_none(self, ground_assets):
        assert ground_assets[0].network_id() is None

    def test_ground_station_network_id_set(self):
        loc = lox.GroundLocation(
            origin=lox.Origin("Earth"),
            longitude=0 * lox.deg,
            latitude=0 * lox.deg,
            altitude=0 * lox.km,
        )
        mask = lox.ElevationMask.fixed(0 * lox.deg)
        gs = lox.GroundStation("test", loc, mask, network_id="estrack")
        assert gs.network_id() == "estrack"

    def test_spacecraft_constellation_id_none(self, space_assets):
        assert space_assets[0].constellation_id() is None

    def test_spacecraft_constellation_id_set(self, oneweb_subset):
        sgp4 = next(iter(oneweb_subset.values()))
        sc = lox.Spacecraft("test", sgp4, constellation_id="oneweb")
        assert sc.constellation_id() == "oneweb"


# ---------------------------------------------------------------------------
# Scenario & Ensemble
# ---------------------------------------------------------------------------


class TestScenario:
    def test_repr(self, scenario):
        r = repr(scenario)
        assert "Scenario(" in r

    def test_start_end(self, scenario, t0, t1):
        assert isinstance(scenario.start(), lox.Time)
        assert isinstance(scenario.end(), lox.Time)

    def test_propagate(self, scenario):
        ensemble = scenario.propagate()
        assert isinstance(ensemble, lox.Ensemble)
        assert len(ensemble) > 0

    def test_ensemble_get(self, scenario, space_assets):
        ensemble = scenario.propagate()
        for sa in space_assets:
            traj = ensemble.get(sa.id())
            assert traj is not None
            assert isinstance(traj, lox.Trajectory)

    def test_ensemble_get_missing(self, scenario):
        ensemble = scenario.propagate()
        assert ensemble.get("nonexistent") is None

    def test_ensemble_repr(self, scenario):
        ensemble = scenario.propagate()
        r = repr(ensemble)
        assert "Ensemble(" in r


# ---------------------------------------------------------------------------
# ElevationMask
# ---------------------------------------------------------------------------


class TestElevationMask:
    def test_fixed(self):
        mask = lox.ElevationMask.fixed(np.radians(10) * lox.rad)
        assert float(mask.min_elevation(0 * lox.rad)) == pytest.approx(np.radians(10))
        assert float(mask.min_elevation(np.pi * lox.rad)) == pytest.approx(
            np.radians(10)
        )
        assert float(mask.fixed_elevation()) == pytest.approx(np.radians(10))
        assert mask.azimuth() is None
        assert mask.elevation() is None

    def test_variable(self):
        az = np.array([-np.pi, 0.0, np.pi])
        el = np.array([0.0, np.radians(10), 0.0])
        mask = lox.ElevationMask.variable(az, el)
        assert float(mask.min_elevation(0 * lox.rad)) == pytest.approx(np.radians(10))
        assert float(mask.min_elevation(-np.pi * lox.rad)) == pytest.approx(0.0)
        assert mask.fixed_elevation() is None
        assert mask.azimuth() is not None
        assert mask.elevation() is not None

    def test_constructor_with_min_elevation(self):
        mask = lox.ElevationMask(min_elevation=np.radians(5) * lox.rad)
        assert float(mask.min_elevation(0 * lox.rad)) == pytest.approx(np.radians(5))

    def test_constructor_with_arrays(self):
        az = np.array([-np.pi, 0.0, np.pi])
        el = np.array([0.0, np.radians(10), 0.0])
        mask = lox.ElevationMask(azimuth=az, elevation=el)
        assert float(mask.min_elevation(0 * lox.rad)) == pytest.approx(np.radians(10))

    def test_constructor_invalid(self):
        with pytest.raises(ValueError):
            lox.ElevationMask()

    def test_equality(self):
        a = lox.ElevationMask.fixed(0.1 * lox.rad)
        b = lox.ElevationMask.fixed(0.1 * lox.rad)
        c = lox.ElevationMask.fixed(0.2 * lox.rad)
        assert a == b
        assert a != c


# ---------------------------------------------------------------------------
# Inter-satellite range filtering
# ---------------------------------------------------------------------------


class TestInterSatelliteRangeFiltering:
    def test_max_range_restricts_intervals(self, t0, t1, space_assets, ephemeris):
        """A tight max_range should produce fewer/shorter intervals than no limit."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris)
        limited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, max_range=500 * lox.km
        )
        res_unlimited = unlimited.run()
        res_limited = limited.run()
        # Every pair in the limited result should have at most as many intervals
        # as the unlimited result (usually fewer or shorter).
        for pair in res_unlimited.windows:
            id1, id2 = pair
            ivs_unlim = [w.interval for w in res_unlimited.windows[(id1, id2)]]
            ivs_lim = [w.interval for w in res_limited.windows[(id1, id2)]]
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim <= dur_unlim + 1e-6

    def test_large_max_range_matches_unlimited(self, t0, t1, space_assets, ephemeris):
        """A very large max_range should not remove any intervals."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris)
        limited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, max_range=1_000_000 * lox.km)
        res_unlimited = unlimited.run()
        res_limited = limited.run()
        assert len(res_limited.windows) == len(res_unlimited.windows)
        for pair in res_unlimited.windows:
            id1, id2 = pair
            ivs_unlim = [w.interval for w in res_unlimited.windows[(id1, id2)]]
            ivs_lim = [w.interval for w in res_limited.windows[(id1, id2)]]
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim == pytest.approx(dur_unlim, abs=1.0)

    def test_min_range_restricts_intervals(self, t0, t1, space_assets, ephemeris):
        """A positive min_range should produce fewer/shorter intervals than no limit."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris)
        limited = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, min_range=1000 * lox.km
        )
        res_unlimited = unlimited.run()
        res_limited = limited.run()
        for pair in res_unlimited.windows:
            id1, id2 = pair
            ivs_unlim = [w.interval for w in res_unlimited.windows[(id1, id2)]]
            ivs_lim = [w.interval for w in res_limited.windows[(id1, id2)]]
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim <= dur_unlim + 1e-6

    def test_combined_min_and_max_range(self, t0, t1, space_assets, ephemeris):
        """Using both min and max range should be more restrictive than either alone."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        max_only = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, max_range=2000 * lox.km
        )
        both = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, min_range=500 * lox.km, max_range=2000 * lox.km)
        res_max = max_only.run()
        res_both = both.run()
        for pair in res_max.windows:
            id1, id2 = pair
            dur_max = sum(float(iv.duration()) for iv in [w.interval for w in res_max.windows[(id1, id2)]])
            dur_both = sum(float(iv.duration()) for iv in [w.interval for w in res_both.windows[(id1, id2)]])
            assert dur_both <= dur_max + 1e-6

    def test_range_with_los(self, t0, t1, space_assets, ephemeris):
        """Range filtering combined with LOS occlusion should work together."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        # Central body (Earth) LOS is always applied for inter-satellite pairs.
        analysis = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, max_range=2000 * lox.km)
        results = analysis.run()
        n = len(space_assets)
        assert len(results.windows) == n * (n - 1) // 2

    def test_range_now_also_gates_ground_space(
        self, scenario, ephemeris, ground_assets, space_assets
    ):
        """Range applies to ground-space too, which the old API could not express.

        A slant-range limit below any achievable range must remove everything.
        """
        without_range = lox.VisibilityAnalysis(scenario).run()
        assert any(without_range.passes.values()), "fixture produced no passes"

        with_range = lox.VisibilityAnalysis(scenario, max_range=100 * lox.km).run()
        assert not any(with_range.passes.values()), (
            "a 100 km slant-range cap should exclude every LEO ground pass"
        )


# ---------------------------------------------------------------------------
# Inter-satellite slew rate filtering
# ---------------------------------------------------------------------------


class TestInterSatelliteSlewRateFiltering:
    def test_space_asset_max_slew_rate(self, oneweb_subset):
        """SpaceAsset accepts max_slew_rate and exposes it via accessor."""
        name, sgp4 = next(iter(oneweb_subset.items()))
        sa = lox.Spacecraft(name, sgp4, max_slew_rate=2.5 * lox.deg_per_s)
        rate = sa.max_slew_rate()
        assert rate is not None
        assert float(rate.to_degrees_per_second()) == pytest.approx(2.5)

    def test_space_asset_no_slew_rate(self, oneweb_subset):
        """SpaceAsset without max_slew_rate returns None."""
        name, sgp4 = next(iter(oneweb_subset.items()))
        sa = lox.Spacecraft(name, sgp4)
        assert sa.max_slew_rate() is None

    def test_slew_rate_restricts_intervals(self, oneweb_subset, t0, t1, ephemeris):
        """A tight slew rate limit should produce less total visibility time."""
        assets_unlimited = [
            lox.Spacecraft(name, sgp4) for name, sgp4 in oneweb_subset.items()
        ]
        assets_limited = [
            lox.Spacecraft(name, sgp4, max_slew_rate=0.01 * lox.deg_per_s)
            for name, sgp4 in oneweb_subset.items()
        ]
        scenario_unlimited = lox.Scenario(t0, t1, spacecraft=assets_unlimited)
        scenario_limited = lox.Scenario(t0, t1, spacecraft=assets_limited)
        res_unlimited = lox.InterSatelliteAnalysis(
            scenario_unlimited, ephemeris=ephemeris
        ).run()
        res_limited = lox.InterSatelliteAnalysis(
            scenario_limited, ephemeris=ephemeris
        ).run()
        for pair in res_unlimited.windows:
            id1, id2 = pair
            dur_unlim = sum(
                float(iv.duration()) for iv in [w.interval for w in res_unlimited.windows[(id1, id2)]]
            )
            dur_lim = sum(
                float(iv.duration()) for iv in [w.interval for w in res_limited.windows[(id1, id2)]]
            )
            assert dur_lim <= dur_unlim + 1e-6

    def test_large_slew_rate_matches_unlimited(self, oneweb_subset, t0, t1, ephemeris):
        """A very generous slew rate should not remove any intervals."""
        assets_unlimited = [
            lox.Spacecraft(name, sgp4) for name, sgp4 in oneweb_subset.items()
        ]
        assets_generous = [
            lox.Spacecraft(name, sgp4, max_slew_rate=1000 * lox.deg_per_s)
            for name, sgp4 in oneweb_subset.items()
        ]
        scenario_unlimited = lox.Scenario(t0, t1, spacecraft=assets_unlimited)
        scenario_generous = lox.Scenario(t0, t1, spacecraft=assets_generous)
        res_unlimited = lox.InterSatelliteAnalysis(
            scenario_unlimited, ephemeris=ephemeris
        ).run()
        res_generous = lox.InterSatelliteAnalysis(
            scenario_generous, ephemeris=ephemeris
        ).run()
        assert len(res_generous.windows) == len(res_unlimited.windows)
        for pair in res_unlimited.windows:
            id1, id2 = pair
            dur_unlim = sum(
                float(iv.duration()) for iv in [w.interval for w in res_unlimited.windows[(id1, id2)]]
            )
            dur_gen = sum(
                float(iv.duration()) for iv in [w.interval for w in res_generous.windows[(id1, id2)]]
            )
            assert dur_gen == pytest.approx(dur_unlim, abs=1.0)

    def test_slew_rate_with_range_and_los(self, oneweb_subset, t0, t1, ephemeris):
        """Slew rate combined with range and LOS constraints should work."""
        assets = [
            lox.Spacecraft(name, sgp4, max_slew_rate=1.0 * lox.deg_per_s)
            for name, sgp4 in oneweb_subset.items()
        ]
        scenario = lox.Scenario(t0, t1, spacecraft=assets)
        # Central body LOS is always applied; only additional bodies need
        # to be passed via occulting_bodies.
        analysis = lox.InterSatelliteAnalysis(scenario, ephemeris=ephemeris, max_range=5000 * lox.km)
        results = analysis.run()
        n = len(assets)
        assert len(results.windows) == n * (n - 1) // 2
