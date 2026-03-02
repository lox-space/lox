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
def results(scenario, ephemeris):
    analysis = lox.VisibilityAnalysis(scenario)
    return analysis.compute(ephemeris)


@pytest.fixture(scope="module")
def results_with_los(scenario, ephemeris):
    analysis = lox.VisibilityAnalysis(
        scenario, occulting_bodies=[lox.Origin("Earth")]
    )
    return analysis.compute(ephemeris)


@pytest.fixture(scope="module")
def combined_results(scenario, ephemeris):
    """Results with both ground-to-space and inter-satellite pairs."""
    analysis = lox.VisibilityAnalysis(scenario, inter_satellite=True)
    return analysis.compute(ephemeris)


@pytest.fixture(scope="module")
def inter_satellite_results(t0, t1, space_assets, ephemeris):
    """Results with inter-satellite pairs only."""
    scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
    analysis = lox.VisibilityAnalysis(scenario, inter_satellite=True)
    return analysis.compute(ephemeris)


# ---------------------------------------------------------------------------
# VisibilityAnalysis construction & compute
# ---------------------------------------------------------------------------


class TestVisibilityAnalysis:
    def test_basic(self, results, ground_assets, space_assets):
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_occulting_bodies(self, results_with_los, ground_assets, space_assets):
        assert results_with_los.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_custom_step(self, scenario, ephemeris, ground_assets, space_assets):
        analysis = lox.VisibilityAnalysis(scenario, step=lox.TimeDelta(30))
        results = analysis.compute(ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_min_pass_duration(
        self, scenario, ephemeris, ground_assets, space_assets
    ):
        analysis = lox.VisibilityAnalysis(
            scenario, min_pass_duration=lox.TimeDelta(300)
        )
        results = analysis.compute(ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_all_options(self, scenario, ephemeris, ground_assets, space_assets):
        analysis = lox.VisibilityAnalysis(
            scenario,
            occulting_bodies=[lox.Origin("Earth")],
            step=lox.TimeDelta(30),
            min_pass_duration=lox.TimeDelta(300),
        )
        results = analysis.compute(ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_inter_satellite_only(self, t0, t1, space_assets, ephemeris):
        """Inter-satellite with no ground assets."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        analysis = lox.VisibilityAnalysis(scenario, inter_satellite=True)
        results = analysis.compute(ephemeris)
        n = len(space_assets)
        assert results.num_pairs() == n * (n - 1) // 2

    def test_combined_ground_and_inter_satellite(
        self, scenario, ephemeris, ground_assets, space_assets
    ):
        """Ground-to-space and inter-satellite pairs together."""
        analysis = lox.VisibilityAnalysis(scenario, inter_satellite=True)
        results = analysis.compute(ephemeris)
        n_gs = len(ground_assets) * len(space_assets)
        n_is = len(space_assets) * (len(space_assets) - 1) // 2
        assert results.num_pairs() == n_gs + n_is

    def test_los_is_subset_of_basic(
        self, results, results_with_los, ground_assets, space_assets
    ):
        """LOS occlusion can only remove intervals, never add."""
        gs_id = ground_assets[0].id()
        for sc in space_assets:
            sc_id = sc.id()
            basic = results.intervals(gs_id, sc_id)
            with_los = results_with_los.intervals(gs_id, sc_id)
            assert len(with_los) <= len(basic)


# ---------------------------------------------------------------------------
# VisibilityResults API
# ---------------------------------------------------------------------------


class TestVisibilityResults:
    def test_pair_ids(self, results, ground_assets, space_assets):
        pair_ids = results.pair_ids()
        assert len(pair_ids) == len(ground_assets) * len(space_assets)
        gs_id = ground_assets[0].id()
        for sc in space_assets:
            assert (gs_id, sc.id()) in pair_ids

    def test_num_pairs(self, results, ground_assets, space_assets):
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_total_intervals(self, results):
        total = results.total_intervals()
        assert isinstance(total, int)
        assert total >= 0

    def test_intervals_for_known_pair(self, results, ground_assets, space_assets):
        gs_id = ground_assets[0].id()
        sc_id = space_assets[0].id()
        intervals = results.intervals(gs_id, sc_id)
        assert isinstance(intervals, list)

    def test_intervals_for_unknown_pair(self, results):
        intervals = results.intervals("nonexistent_gs", "nonexistent_sc")
        assert intervals == []

    def test_passes_for_known_pair(self, results, ground_assets, space_assets):
        gs_id = ground_assets[0].id()
        sc_id = space_assets[0].id()
        passes = results.passes(gs_id, sc_id)
        assert isinstance(passes, list)

    def test_passes_for_unknown_pair(self, results):
        passes = results.passes("nonexistent_gs", "nonexistent_sc")
        assert passes == []

    def test_all_passes(self, results, ground_assets, space_assets):
        all_passes = results.all_passes()
        assert isinstance(all_passes, dict)
        assert len(all_passes) == results.num_pairs()
        for (gs_id, sc_id), passes in all_passes.items():
            assert isinstance(gs_id, str)
            assert isinstance(sc_id, str)
            assert isinstance(passes, list)


# ---------------------------------------------------------------------------
# Pair type filtering
# ---------------------------------------------------------------------------


class TestPairTypeFiltering:
    def test_ground_space_pair_ids(self, results, ground_assets, space_assets):
        """Ground-only results should have all pairs as ground-space."""
        gs_pair_ids = results.ground_space_pair_ids()
        assert len(gs_pair_ids) == len(ground_assets) * len(space_assets)

    def test_inter_satellite_pair_ids_empty(self, results):
        """Ground-only results should have no inter-satellite pairs."""
        is_pair_ids = results.inter_satellite_pair_ids()
        assert is_pair_ids == []

    def test_combined_ground_space_pair_ids(
        self, combined_results, ground_assets, space_assets
    ):
        gs_pair_ids = combined_results.ground_space_pair_ids()
        assert len(gs_pair_ids) == len(ground_assets) * len(space_assets)

    def test_combined_inter_satellite_pair_ids(self, combined_results, space_assets):
        is_pair_ids = combined_results.inter_satellite_pair_ids()
        n = len(space_assets)
        assert len(is_pair_ids) == n * (n - 1) // 2

    def test_combined_pair_ids_partition(
        self, combined_results, ground_assets, space_assets
    ):
        """ground_space + inter_satellite should equal all pair_ids."""
        all_ids = set(combined_results.pair_ids())
        gs_ids = set(combined_results.ground_space_pair_ids())
        is_ids = set(combined_results.inter_satellite_pair_ids())
        assert gs_ids | is_ids == all_ids
        assert gs_ids & is_ids == set()

    def test_all_intervals(self, combined_results, ground_assets, space_assets):
        all_ivs = combined_results.all_intervals()
        n_gs = len(ground_assets) * len(space_assets)
        n_is = len(space_assets) * (len(space_assets) - 1) // 2
        assert len(all_ivs) == n_gs + n_is

    def test_ground_space_intervals(
        self, combined_results, ground_assets, space_assets
    ):
        gs_ivs = combined_results.ground_space_intervals()
        assert len(gs_ivs) == len(ground_assets) * len(space_assets)

    def test_inter_satellite_intervals(self, combined_results, space_assets):
        is_ivs = combined_results.inter_satellite_intervals()
        n = len(space_assets)
        assert len(is_ivs) == n * (n - 1) // 2

    def test_passes_raises_for_inter_satellite(
        self, inter_satellite_results, space_assets
    ):
        """passes() should raise ValueError for inter-satellite pairs."""
        pair_ids = inter_satellite_results.inter_satellite_pair_ids()
        assert len(pair_ids) > 0
        id1, id2 = pair_ids[0]
        with pytest.raises(ValueError, match="inter-satellite"):
            inter_satellite_results.passes(id1, id2)

    def test_all_passes_skips_inter_satellite(
        self, combined_results, ground_assets, space_assets
    ):
        """all_passes() should only return ground-to-space pairs."""
        all_passes = combined_results.all_passes()
        gs_pair_ids = set(combined_results.ground_space_pair_ids())
        assert set(all_passes.keys()) == gs_pair_ids


# ---------------------------------------------------------------------------
# Interval API
# ---------------------------------------------------------------------------


class TestInterval:
    def _first_interval(self, results, ground_assets, space_assets):
        """Find the first non-empty pair and return its first interval."""
        for gs in ground_assets:
            for sc in space_assets:
                intervals = results.intervals(gs.id(), sc.id())
                if intervals:
                    return intervals[0]
        pytest.skip("no visibility intervals in test interval")

    def test_start_end_duration(self, results, ground_assets, space_assets, t0, t1):
        w = self._first_interval(results, ground_assets, space_assets)
        start = w.start()
        end = w.end()
        duration = w.duration()
        assert isinstance(start, lox.Time)
        assert isinstance(end, lox.Time)
        assert isinstance(duration, lox.TimeDelta)
        assert float(duration) > 0

    def test_repr(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        r = repr(w)
        assert r.startswith("Interval(")
        assert ")" in r

    def test_is_empty(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        assert not w.is_empty()
        # Reversed interval is empty
        empty = lox.Interval(w.end(), w.start())
        assert empty.is_empty()

    def test_contains_time(self, results, ground_assets, space_assets, t0):
        w = self._first_interval(results, ground_assets, space_assets)
        mid = w.start() + lox.TimeDelta(float(w.duration()) / 2.0)
        assert w.contains_time(mid)
        before = w.start() - lox.TimeDelta(86400)
        assert not w.contains_time(before)

    def test_contains(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        assert w.contains(w)

    def test_intersect(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        # Self-intersection equals self
        inter = w.intersect(w)
        assert float(inter.duration()) == pytest.approx(float(w.duration()))
        # Intersection with non-overlapping is empty
        far = lox.Interval(
            w.end() + lox.TimeDelta(86400),
            w.end() + lox.TimeDelta(2 * 86400),
        )
        assert w.intersect(far).is_empty()

    def test_overlaps(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        assert w.overlaps(w)
        far = lox.Interval(
            w.end() + lox.TimeDelta(86400),
            w.end() + lox.TimeDelta(2 * 86400),
        )
        assert not w.overlaps(far)

    def test_step_by(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        step = lox.TimeDelta(60)
        times = w.step_by(step)
        assert all(isinstance(t, lox.Time) for t in times)
        expected_count = int(float(w.duration()) / 60) + 1
        assert abs(len(times) - expected_count) <= 1

    def test_linspace(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        times = w.linspace(5)
        assert len(times) == 5
        assert all(isinstance(t, lox.Time) for t in times)

    def test_step_by_zero_raises(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        with pytest.raises(ValueError):
            w.step_by(lox.TimeDelta(0))

    def test_linspace_one_raises(self, results, ground_assets, space_assets):
        w = self._first_interval(results, ground_assets, space_assets)
        with pytest.raises(ValueError):
            w.linspace(1)


# ---------------------------------------------------------------------------
# Interval set operations
# ---------------------------------------------------------------------------


class TestIntervalOperations:
    def test_intersect_intervals(self, t0):
        t1 = t0 + lox.TimeDelta(100)
        t2 = t0 + lox.TimeDelta(200)
        t3 = t0 + lox.TimeDelta(300)
        # Disjoint → empty
        a = [lox.Interval(t0, t1)]
        b = [lox.Interval(t2, t3)]
        assert lox.intersect_intervals(a, b) == []
        # Overlapping → non-empty
        b2 = [lox.Interval(t0 + lox.TimeDelta(50), t2)]
        result = lox.intersect_intervals(a, b2)
        assert len(result) == 1
        assert not result[0].is_empty()

    def test_union_intervals(self, t0):
        t1 = t0 + lox.TimeDelta(100)
        t2 = t0 + lox.TimeDelta(200)
        t3 = t0 + lox.TimeDelta(300)
        # Adjacent/overlapping → merged
        a = [lox.Interval(t0, t2)]
        b = [lox.Interval(t1, t3)]
        result = lox.union_intervals(a, b)
        assert len(result) == 1
        # Disjoint → both preserved
        a2 = [lox.Interval(t0, t1)]
        b2 = [lox.Interval(t2, t3)]
        result2 = lox.union_intervals(a2, b2)
        assert len(result2) == 2

    def test_complement_intervals(self, t0):
        t1 = t0 + lox.TimeDelta(100)
        t2 = t0 + lox.TimeDelta(200)
        t3 = t0 + lox.TimeDelta(300)
        bound = lox.Interval(t0, t3)
        intervals = [lox.Interval(t1, t2)]
        result = lox.complement_intervals(intervals, bound)
        assert len(result) == 2
        # The gaps should be [t0, t1) and [t2, t3)
        assert float(result[0].duration()) == pytest.approx(100.0)
        assert float(result[1].duration()) == pytest.approx(100.0)


# ---------------------------------------------------------------------------
# Pass API
# ---------------------------------------------------------------------------


class TestPass:
    def _first_pass_with_gs(self, results, ground_assets, space_assets):
        """Find the first non-empty pair and return (pass, ground_asset)."""
        for gs in ground_assets:
            for sc in space_assets:
                passes = results.passes(gs.id(), sc.id())
                if passes:
                    return passes[0], gs
        pytest.skip("no passes in test interval")

    def _first_pass(self, results, ground_assets, space_assets):
        p, _ = self._first_pass_with_gs(results, ground_assets, space_assets)
        return p

    def test_interval(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        w = p.interval()
        assert isinstance(w, lox.Interval)
        assert float(w.duration()) > 0

    def test_times(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        times = p.times()
        assert len(times) >= 2
        assert all(isinstance(t, lox.Time) for t in times)

    def test_observables(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        obs_list = p.observables()
        assert len(obs_list) == len(p.times())
        for obs in obs_list:
            assert isinstance(obs, lox.Observables)
            assert float(obs.range()) > 0
            assert -math.pi <= float(obs.azimuth()) <= math.pi

    def test_observables_above_mask(self, results, ground_assets, space_assets):
        """All observables in a pass should be above the elevation mask."""
        p, gs = self._first_pass_with_gs(results, ground_assets, space_assets)
        mask = gs.mask()
        for obs in p.observables():
            min_elev = mask.min_elevation(obs.azimuth())
            assert float(obs.elevation()) >= float(min_elev)

    def test_interpolate_within_pass(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        mid = p.times()[len(p.times()) // 2]
        obs = p.interpolate(mid)
        assert obs is not None
        assert isinstance(obs, lox.Observables)
        assert float(obs.range()) > 0

    def test_interpolate_outside_pass(self, results, ground_assets, space_assets, t0):
        p = self._first_pass(results, ground_assets, space_assets)
        # Well before the pass
        before = t0 - lox.TimeDelta(86400)
        assert p.interpolate(before) is None

    def test_repr(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        r = repr(p)
        assert "Pass(" in r
        assert "observables" in r


# ---------------------------------------------------------------------------
# Asset accessors
# ---------------------------------------------------------------------------


class TestAssets:
    def test_ground_asset_id(self, ground_assets):
        for ga in ground_assets:
            assert isinstance(ga.id(), str)
            assert len(ga.id()) > 0

    def test_ground_asset_location(self, ground_assets):
        for ga in ground_assets:
            loc = ga.location()
            assert isinstance(loc, lox.GroundLocation)

    def test_ground_asset_mask(self, ground_assets):
        for ga in ground_assets:
            mask = ga.mask()
            assert isinstance(mask, lox.ElevationMask)

    def test_space_asset_id(self, space_assets):
        for sa in space_assets:
            assert isinstance(sa.id(), str)
            assert len(sa.id()) > 0


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
        unlimited = lox.VisibilityAnalysis(scenario, inter_satellite=True)
        limited = lox.VisibilityAnalysis(
            scenario, inter_satellite=True, max_range=500 * lox.km
        )
        res_unlimited = unlimited.compute(ephemeris)
        res_limited = limited.compute(ephemeris)
        # Every pair in the limited result should have at most as many intervals
        # as the unlimited result (usually fewer or shorter).
        for pair in res_unlimited.inter_satellite_pair_ids():
            id1, id2 = pair
            ivs_unlim = res_unlimited.intervals(id1, id2)
            ivs_lim = res_limited.intervals(id1, id2)
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim <= dur_unlim + 1e-6

    def test_large_max_range_matches_unlimited(self, t0, t1, space_assets, ephemeris):
        """A very large max_range should not remove any intervals."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.VisibilityAnalysis(scenario, inter_satellite=True)
        limited = lox.VisibilityAnalysis(
            scenario,
            inter_satellite=True,
            max_range=1_000_000 * lox.km,
        )
        res_unlimited = unlimited.compute(ephemeris)
        res_limited = limited.compute(ephemeris)
        assert res_limited.num_pairs() == res_unlimited.num_pairs()
        for pair in res_unlimited.inter_satellite_pair_ids():
            id1, id2 = pair
            ivs_unlim = res_unlimited.intervals(id1, id2)
            ivs_lim = res_limited.intervals(id1, id2)
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim == pytest.approx(dur_unlim, abs=1.0)

    def test_min_range_restricts_intervals(self, t0, t1, space_assets, ephemeris):
        """A positive min_range should produce fewer/shorter intervals than no limit."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        unlimited = lox.VisibilityAnalysis(scenario, inter_satellite=True)
        limited = lox.VisibilityAnalysis(
            scenario, inter_satellite=True, min_range=1000 * lox.km
        )
        res_unlimited = unlimited.compute(ephemeris)
        res_limited = limited.compute(ephemeris)
        for pair in res_unlimited.inter_satellite_pair_ids():
            id1, id2 = pair
            ivs_unlim = res_unlimited.intervals(id1, id2)
            ivs_lim = res_limited.intervals(id1, id2)
            dur_unlim = sum(float(iv.duration()) for iv in ivs_unlim)
            dur_lim = sum(float(iv.duration()) for iv in ivs_lim)
            assert dur_lim <= dur_unlim + 1e-6

    def test_combined_min_and_max_range(self, t0, t1, space_assets, ephemeris):
        """Using both min and max range should be more restrictive than either alone."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        max_only = lox.VisibilityAnalysis(
            scenario, inter_satellite=True, max_range=2000 * lox.km
        )
        both = lox.VisibilityAnalysis(
            scenario,
            inter_satellite=True,
            min_range=500 * lox.km,
            max_range=2000 * lox.km,
        )
        res_max = max_only.compute(ephemeris)
        res_both = both.compute(ephemeris)
        for pair in res_max.inter_satellite_pair_ids():
            id1, id2 = pair
            dur_max = sum(float(iv.duration()) for iv in res_max.intervals(id1, id2))
            dur_both = sum(float(iv.duration()) for iv in res_both.intervals(id1, id2))
            assert dur_both <= dur_max + 1e-6

    def test_range_with_los(self, t0, t1, space_assets, ephemeris):
        """Range filtering combined with LOS occlusion should work together."""
        scenario = lox.Scenario(t0, t1, spacecraft=space_assets)
        analysis = lox.VisibilityAnalysis(
            scenario,
            occulting_bodies=[lox.Origin("Earth")],
            inter_satellite=True,
            max_range=2000 * lox.km,
        )
        results = analysis.compute(ephemeris)
        n = len(space_assets)
        assert results.num_pairs() == n * (n - 1) // 2

    def test_range_does_not_affect_ground_space(
        self, scenario, ephemeris, ground_assets, space_assets
    ):
        """Range constraints only apply to inter-satellite pairs, not ground-space."""
        without_range = lox.VisibilityAnalysis(scenario)
        with_range = lox.VisibilityAnalysis(scenario, max_range=100 * lox.km)
        res_without = without_range.compute(ephemeris)
        res_with = with_range.compute(ephemeris)
        for pair in res_without.ground_space_pair_ids():
            id1, id2 = pair
            ivs_without = res_without.intervals(id1, id2)
            ivs_with = res_with.intervals(id1, id2)
            assert len(ivs_without) == len(ivs_with)


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
        res_unlimited = lox.VisibilityAnalysis(
            scenario_unlimited, inter_satellite=True
        ).compute(ephemeris)
        res_limited = lox.VisibilityAnalysis(
            scenario_limited, inter_satellite=True
        ).compute(ephemeris)
        for pair in res_unlimited.inter_satellite_pair_ids():
            id1, id2 = pair
            dur_unlim = sum(
                float(iv.duration()) for iv in res_unlimited.intervals(id1, id2)
            )
            dur_lim = sum(
                float(iv.duration()) for iv in res_limited.intervals(id1, id2)
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
        res_unlimited = lox.VisibilityAnalysis(
            scenario_unlimited, inter_satellite=True
        ).compute(ephemeris)
        res_generous = lox.VisibilityAnalysis(
            scenario_generous, inter_satellite=True
        ).compute(ephemeris)
        assert res_generous.num_pairs() == res_unlimited.num_pairs()
        for pair in res_unlimited.inter_satellite_pair_ids():
            id1, id2 = pair
            dur_unlim = sum(
                float(iv.duration()) for iv in res_unlimited.intervals(id1, id2)
            )
            dur_gen = sum(
                float(iv.duration()) for iv in res_generous.intervals(id1, id2)
            )
            assert dur_gen == pytest.approx(dur_unlim, abs=1.0)

    def test_slew_rate_with_range_and_los(self, oneweb_subset, t0, t1, ephemeris):
        """Slew rate combined with range and LOS constraints should work."""
        assets = [
            lox.Spacecraft(name, sgp4, max_slew_rate=1.0 * lox.deg_per_s)
            for name, sgp4 in oneweb_subset.items()
        ]
        scenario = lox.Scenario(t0, t1, spacecraft=assets)
        analysis = lox.VisibilityAnalysis(
            scenario,
            occulting_bodies=[lox.Origin("Earth")],
            inter_satellite=True,
            max_range=5000 * lox.km,
        )
        results = analysis.compute(ephemeris)
        n = len(assets)
        assert results.num_pairs() == n * (n - 1) // 2
