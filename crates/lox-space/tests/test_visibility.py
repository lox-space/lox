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
    return [lox.SpaceAsset(name, traj) for name, traj in oneweb_subset.items()]


@pytest.fixture(scope="module")
def ground_assets(estrack):
    return estrack


@pytest.fixture(scope="module")
def t0(oneweb_subset):
    traj = next(iter(oneweb_subset.values()))
    return traj.states()[0].time()


@pytest.fixture(scope="module")
def t1(t0):
    return t0 + lox.TimeDelta(86400)  # 24-hour window


@pytest.fixture(scope="module")
def results(ground_assets, space_assets, t0, t1, ephemeris):
    analysis = lox.VisibilityAnalysis(ground_assets, space_assets)
    return analysis.compute(t0, t1, ephemeris)


@pytest.fixture(scope="module")
def results_with_los(ground_assets, space_assets, t0, t1, ephemeris):
    analysis = lox.VisibilityAnalysis(
        ground_assets,
        space_assets,
        occulting_bodies=[lox.Origin("Earth")],
    )
    return analysis.compute(t0, t1, ephemeris)


# ---------------------------------------------------------------------------
# VisibilityAnalysis construction & compute
# ---------------------------------------------------------------------------


class TestVisibilityAnalysis:
    def test_basic(self, results, ground_assets, space_assets):
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_occulting_bodies(self, results_with_los, ground_assets, space_assets):
        assert results_with_los.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_custom_step(self, ground_assets, space_assets, t0, t1, ephemeris):
        analysis = lox.VisibilityAnalysis(
            ground_assets, space_assets, step=lox.TimeDelta(30)
        )
        results = analysis.compute(t0, t1, ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_min_pass_duration(
        self, ground_assets, space_assets, t0, t1, ephemeris
    ):
        analysis = lox.VisibilityAnalysis(
            ground_assets, space_assets, min_pass_duration=lox.TimeDelta(300)
        )
        results = analysis.compute(t0, t1, ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_with_all_options(self, ground_assets, space_assets, t0, t1, ephemeris):
        analysis = lox.VisibilityAnalysis(
            ground_assets,
            space_assets,
            occulting_bodies=[lox.Origin("Earth")],
            step=lox.TimeDelta(30),
            min_pass_duration=lox.TimeDelta(300),
        )
        results = analysis.compute(t0, t1, ephemeris)
        assert results.num_pairs() == len(ground_assets) * len(space_assets)

    def test_inter_satellite_only(self, space_assets, t0, t1, ephemeris):
        """Inter-satellite with no ground assets."""
        analysis = lox.VisibilityAnalysis([], space_assets, inter_satellite=True)
        results = analysis.compute(t0, t1, ephemeris)
        n = len(space_assets)
        assert results.num_pairs() == n * (n - 1) // 2

    def test_combined_ground_and_inter_satellite(
        self, ground_assets, space_assets, t0, t1, ephemeris
    ):
        """Ground-to-space and inter-satellite pairs together."""
        analysis = lox.VisibilityAnalysis(
            ground_assets, space_assets, inter_satellite=True
        )
        results = analysis.compute(t0, t1, ephemeris)
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
# Window API
# ---------------------------------------------------------------------------


class TestWindow:
    def _first_window(self, results, ground_assets, space_assets):
        """Find the first non-empty pair and return its first window."""
        for gs in ground_assets:
            for sc in space_assets:
                windows = results.intervals(gs.id(), sc.id())
                if windows:
                    return windows[0]
        pytest.skip("no visibility windows in test interval")

    def test_start_end_duration(self, results, ground_assets, space_assets, t0, t1):
        w = self._first_window(results, ground_assets, space_assets)
        start = w.start()
        end = w.end()
        duration = w.duration()
        assert isinstance(start, lox.Time)
        assert isinstance(end, lox.Time)
        assert isinstance(duration, lox.TimeDelta)
        assert float(duration) > 0

    def test_repr(self, results, ground_assets, space_assets):
        w = self._first_window(results, ground_assets, space_assets)
        r = repr(w)
        assert r.startswith("Window(")
        assert ")" in r


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

    def test_window(self, results, ground_assets, space_assets):
        p = self._first_pass(results, ground_assets, space_assets)
        w = p.window()
        assert isinstance(w, lox.Window)
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

    def test_space_asset_trajectory(self, space_assets):
        for sa in space_assets:
            traj = sa.trajectory()
            assert isinstance(traj, lox.Trajectory)


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
