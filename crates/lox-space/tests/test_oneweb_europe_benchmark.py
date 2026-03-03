# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""
OneWeb Europe Coverage Benchmark

This benchmark tests visibility analysis performance with a realistic large-scale scenario:
- 651 OneWeb satellites from TLE data (using existing oneweb fixture)
- Ground points covering Europe with 1-degree resolution (~630 points)
- Total combinations: ~410,000 visibility calculations

This represents a realistic worst-case scenario for satellite visibility analysis.
"""

import time
import pytest
import numpy as np
import lox_space as lox


def create_europe_ground_assets(resolution_deg=1.0, min_elevation_deg=10.0):
    """
    Create a list of ground assets covering Europe with specified resolution.

    Parameters
    ----------
    resolution_deg : float
        Grid resolution in degrees (default: 1.0)
    min_elevation_deg : float
        Minimum elevation angle in degrees (default: 10.0)

    Returns
    -------
    list[lox.GroundStation]
        List of ground assets covering Europe.
    """
    # Europe bounding box (approximate)
    lon_min, lon_max = -10.0, 40.0  # West to East
    lat_min, lat_max = 35.0, 70.0  # South to North

    # Generate grid points
    lons = np.arange(lon_min, lon_max + resolution_deg, resolution_deg)
    lats = np.arange(lat_min, lat_max + resolution_deg, resolution_deg)

    elevation_mask = lox.ElevationMask.fixed(min_elevation_deg * lox.deg)
    origin = lox.Origin("Earth")

    ground_assets = []
    point_count = 0
    for lon in lons:
        for lat in lats:
            # Filter to approximate Europe shape (rough filtering)
            if is_point_in_europe(lon, lat):
                gs_name = f"EU_GS_{point_count:04d}"
                ground_location = lox.GroundLocation(
                    origin=origin,
                    longitude=lon * lox.deg,
                    latitude=lat * lox.deg,
                    altitude=0.0 * lox.km,  # Sea level
                )
                ground_assets.append(
                    lox.GroundStation(gs_name, ground_location, elevation_mask)
                )
                point_count += 1

    return ground_assets


def is_point_in_europe(lon, lat):
    """
    Rough filter to include only points that are approximately in Europe.
    This is a simplified polygon check for demonstration purposes.
    """
    # Very rough Europe boundary - exclude ocean areas
    if lat < 36.0:  # Below southern Europe
        return False
    if lat > 71.0:  # Above northern Europe
        return False
    if lon < -9.0:  # West of Ireland
        return False
    if lon > 40.0:  # East of Urals
        return False

    # Exclude some obvious non-Europe areas
    if lon < -5.0 and lat < 43.0:  # Atlantic Ocean
        return False
    if lon > 30.0 and lat < 45.0:  # Turkey/Middle East area
        return False
    if lon > 35.0 and lat > 60.0:  # Far eastern Russia
        return False

    return True


@pytest.fixture(scope="session")
def europe_ground_assets():
    """Fixture providing Europe ground asset grid."""
    return create_europe_ground_assets(resolution_deg=1.0)


@pytest.fixture(scope="session")
def europe_ground_assets_coarse():
    """Fixture providing coarser Europe ground asset grid for faster testing."""
    return create_europe_ground_assets(resolution_deg=2.0)


@pytest.fixture(scope="session")
def t0(oneweb):
    return next(iter(oneweb.values())).time()


@pytest.fixture(scope="session")
def t1(t0):
    return t0 + lox.TimeDelta(7200)


def make_space_assets(oneweb_dict):
    return [lox.Spacecraft(name, sgp4) for name, sgp4 in oneweb_dict.items()]


def make_scenario_and_ensemble(oneweb_dict, ground_assets, t0, t1):
    space_assets = make_space_assets(oneweb_dict)
    scenario = lox.Scenario(
        t0, t1, spacecraft=space_assets, ground_stations=ground_assets
    )
    ensemble = scenario.propagate()
    return scenario, ensemble


@pytest.fixture(scope="session")
def oneweb_sample_small(oneweb):
    """Small sample of OneWeb constellation for quick tests."""
    return dict(list(oneweb.items())[:10])


@pytest.fixture(scope="session")
def oneweb_sample_medium(oneweb):
    """Medium sample of OneWeb constellation for performance tests."""
    return dict(list(oneweb.items())[:50])


@pytest.fixture(scope="session")
def oneweb_sample_large(oneweb):
    """Large sample of OneWeb constellation for stress tests."""
    return dict(list(oneweb.items())[:200])


@pytest.fixture(scope="session")
def small_scenario_and_ensemble(oneweb_sample_small, europe_ground_assets_coarse, t0, t1):
    """Pre-computed scenario and ensemble for small benchmark."""
    sample_ground_assets = europe_ground_assets_coarse[:10]
    return make_scenario_and_ensemble(oneweb_sample_small, sample_ground_assets, t0, t1)


@pytest.fixture(scope="session")
def medium_scenario_and_ensemble(oneweb_sample_medium, europe_ground_assets_coarse, t0, t1):
    """Pre-computed scenario and ensemble for medium benchmark."""
    sample_ground_assets = europe_ground_assets_coarse[:50]
    return make_scenario_and_ensemble(oneweb_sample_medium, sample_ground_assets, t0, t1)


@pytest.fixture(scope="session")
def large_scenario_and_ensemble(oneweb_sample_large, europe_ground_assets, t0, t1):
    """Pre-computed scenario and ensemble for large benchmark."""
    sample_ground_assets = europe_ground_assets[:100]
    return make_scenario_and_ensemble(oneweb_sample_large, sample_ground_assets, t0, t1)


class TestOneWebEuropeBenchmark:
    """Benchmark tests for OneWeb Europe coverage scenario."""

    def test_benchmark_info(self, europe_ground_assets, oneweb):
        """Display benchmark scenario information."""
        num_spacecraft = len(oneweb)
        num_ground_stations = len(europe_ground_assets)
        total_combinations = num_spacecraft * num_ground_stations

        print(f"\n{'=' * 60}")
        print("OneWeb Europe Coverage Benchmark")
        print(f"{'=' * 60}")
        print(f"Spacecraft (OneWeb): {num_spacecraft}")
        print(f"Ground stations (Europe 1deg): {num_ground_stations}")
        print(f"Total combinations: {total_combinations:,}")
        print(f"Expected visibility calculations: {total_combinations:,}")
        print(f"{'=' * 60}")

        # This is not a real test, just info display
        assert num_spacecraft > 600  # Should have most OneWeb satellites
        assert num_ground_stations > 500  # Should cover Europe well
        assert total_combinations > 300000  # Should be a large-scale test

    def test_benchmark_sequential_sample(
        self,
        small_scenario_and_ensemble,
        oneweb_sample_small,
        europe_ground_assets_coarse,
        ephemeris,
    ):
        """Test visibility analysis on a small sample."""
        scenario, ensemble = small_scenario_and_ensemble
        analysis = lox.VisibilityAnalysis(scenario, ensemble=ensemble)
        results = analysis.compute(ephemeris)

        sample_ground_assets = europe_ground_assets_coarse[:10]
        assert results.num_pairs() == len(oneweb_sample_small) * len(
            sample_ground_assets
        )

    @pytest.mark.benchmark
    def test_benchmark_parallel_medium(
        self,
        medium_scenario_and_ensemble,
        oneweb_sample_medium,
        europe_ground_assets_coarse,
        ephemeris,
    ):
        """Test visibility analysis on a medium sample."""
        scenario, ensemble = medium_scenario_and_ensemble
        analysis = lox.VisibilityAnalysis(scenario, ensemble=ensemble)
        results = analysis.compute(ephemeris)

        sample_ground_assets = europe_ground_assets_coarse[:50]
        assert results.num_pairs() == len(oneweb_sample_medium) * len(
            sample_ground_assets
        )

    @pytest.mark.slow
    def test_benchmark_parallel_large(
        self,
        large_scenario_and_ensemble,
        oneweb_sample_large,
        europe_ground_assets,
        ephemeris,
    ):
        """Test visibility analysis on a large subset (marked as slow)."""
        scenario, ensemble = large_scenario_and_ensemble
        analysis = lox.VisibilityAnalysis(scenario, ensemble=ensemble)
        results = analysis.compute(ephemeris)

        sample_ground_assets = europe_ground_assets[:100]
        assert results.num_pairs() == len(oneweb_sample_large) * len(
            sample_ground_assets
        )

    @pytest.mark.slow
    @pytest.mark.benchmark
    def test_full_scale_estimate(self, europe_ground_assets, oneweb):
        """Estimate performance for full-scale scenario."""
        total_spacecraft = len(oneweb)
        total_ground_stations = len(europe_ground_assets)
        total_combinations = total_spacecraft * total_ground_stations

        print(f"\n{'=' * 60}")
        print("Full-Scale Performance Estimate")
        print(f"{'=' * 60}")
        print(f"Total OneWeb satellites: {total_spacecraft}")
        print(f"Total Europe ground points: {total_ground_stations}")
        print(f"Total combinations: {total_combinations:,}")
        print(
            f"Memory estimate (results only): {total_combinations * 200 / 1024 / 1024:.1f} MB"
        )
        print(f"{'=' * 60}")

        # This is just an informational test
        assert total_combinations > 400000
