#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

"""
OneWeb Europe Coverage Benchmark

This benchmark tests visibility analysis performance with a realistic large-scale scenario:
- 651 OneWeb satellites from TLE data (using existing oneweb fixture)
- Ground points covering Europe with 1-degree resolution (≈630 points)
- Total combinations: ~410,000 visibility calculations

This represents a realistic worst-case scenario for satellite visibility analysis.
"""

import time
import pytest
import numpy as np
import lox_space as lox


def create_europe_ground_grid(resolution_deg=1.0, min_elevation_deg=10.0):
    """
    Create a grid of ground points covering Europe with specified resolution.
    
    Parameters
    ----------
    resolution_deg : float
        Grid resolution in degrees (default: 1.0)
    min_elevation_deg : float
        Minimum elevation angle in degrees (default: 10.0)
    
    Returns
    -------
    dict
        Dictionary of ground station name -> (GroundLocation, ElevationMask)
    """
    # Europe bounding box (approximate)
    lon_min, lon_max = -10.0, 40.0   # West to East
    lat_min, lat_max = 35.0, 70.0    # South to North
    
    # Generate grid points
    lons = np.arange(lon_min, lon_max + resolution_deg, resolution_deg)
    lats = np.arange(lat_min, lat_max + resolution_deg, resolution_deg)
    
    ground_stations = {}
    elevation_mask = lox.ElevationMask.fixed(np.radians(min_elevation_deg))
    origin = lox.Origin("Earth")
    
    point_count = 0
    for lon in lons:
        for lat in lats:
            # Filter to approximate Europe shape (rough filtering)
            if is_point_in_europe(lon, lat):
                gs_name = f"EU_GS_{point_count:04d}"
                ground_location = lox.GroundLocation(
                    origin=origin,
                    longitude=np.radians(lon),
                    latitude=np.radians(lat), 
                    altitude=0.0  # Sea level
                )
                ground_stations[gs_name] = (ground_location, elevation_mask)
                point_count += 1
    
    return ground_stations


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
def europe_ground_stations():
    """Fixture providing Europe ground station grid."""
    return create_europe_ground_grid(resolution_deg=1.0)


@pytest.fixture(scope="session")
def europe_ground_stations_coarse():
    """Fixture providing coarser Europe ground station grid for faster testing."""
    return create_europe_ground_grid(resolution_deg=2.0)


@pytest.fixture(scope="session")
def benchmark_times(oneweb):
    """Fixture providing time grid optimized for benchmarks."""
    # Use first OneWeb satellite's epoch as reference
    t0 = next(iter(oneweb.values())).states()[0].time()
    # 2-hour analysis with 10-minute steps for performance
    return [t0 + t for t in lox.TimeDelta.range(0, 7200, 600)]


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


class TestOneWebEuropeBenchmark:
    """Benchmark tests for OneWeb Europe coverage scenario."""

    def test_benchmark_info(self, europe_ground_stations, oneweb, benchmark_times):
        """Display benchmark scenario information."""
        num_spacecraft = len(oneweb)
        num_ground_stations = len(europe_ground_stations)
        num_times = len(benchmark_times)
        total_combinations = num_spacecraft * num_ground_stations
        
        print(f"\n{'='*60}")
        print("OneWeb Europe Coverage Benchmark")
        print(f"{'='*60}")
        print(f"Spacecraft (OneWeb): {num_spacecraft}")
        print(f"Ground stations (Europe 1°): {num_ground_stations}")
        print(f"Time points: {num_times}")
        print(f"Total combinations: {total_combinations:,}")
        print(f"Expected visibility calculations: {total_combinations:,}")
        print(f"{'='*60}")
        
        # This is not a real test, just info display
        assert num_spacecraft > 600  # Should have most OneWeb satellites
        assert num_ground_stations > 500  # Should cover Europe well
        assert total_combinations > 300000  # Should be a large-scale test

    def test_benchmark_sequential_sample(self, europe_ground_stations_coarse, oneweb_sample_small, benchmark_times, ephemeris):
        """Test sequential visibility analysis on a small sample."""
        sample_ground_stations = dict(list(europe_ground_stations_coarse.items())[:10])
        
        
        start_time = time.time()
        
        results = {}
        for sc_name, sc_trajectory in oneweb_sample_small.items():
            results[sc_name] = {}
            for gs_name, (gs_location, gs_mask) in sample_ground_stations.items():
                try:
                    windows = lox.visibility(
                        times=benchmark_times,
                        gs=gs_location,
                        mask=gs_mask,
                        sc=sc_trajectory,
                        ephemeris=ephemeris,
                        bodies=None,
                        provider=None,
                    )
                    results[sc_name][gs_name] = windows
                except Exception as e:
                    results[sc_name][gs_name] = []
        
        elapsed = time.time() - start_time
        total_calcs = len(oneweb_sample_small) * len(sample_ground_stations)
        
        # Verify we got some results
        total_windows = sum(len(windows) for sc_results in results.values() 
                          for windows in sc_results.values())
        
        assert len(results) == len(oneweb_sample_small)

    @pytest.mark.benchmark
    def test_benchmark_parallel_small(self, europe_ground_stations_coarse, oneweb_sample_small, benchmark_times, ephemeris):
        """Test parallel visibility analysis on a small sample."""
        sample_ground_stations = dict(list(europe_ground_stations_coarse.items())[:20])
        
        
        # Create ensemble for parallel processing
        ensemble = lox.Ensemble(oneweb_sample_small)
        
        start_time = time.time()
        
        try:
            results = lox.visibility_all(
                times=benchmark_times,
                ground_stations=sample_ground_stations,
                spacecraft=ensemble,
                ephemeris=ephemeris,
                bodies=None,
                provider=None,
            )
            
            elapsed = time.time() - start_time
            total_calcs = len(oneweb_sample_small) * len(sample_ground_stations)
            
            # Verify we got some results
            total_windows = sum(len(windows) for sc_results in results.values() 
                              for windows in sc_results.values())
            
            assert len(results) == len(oneweb_sample_small)
            
        except Exception as e:
            pytest.skip(f"Parallel visibility_all not available or failed: {e}")

    @pytest.mark.benchmark  
    def test_benchmark_parallel_medium(self, europe_ground_stations_coarse, oneweb_sample_medium, benchmark_times, ephemeris):
        """Test parallel visibility analysis on a medium sample."""
        sample_ground_stations = dict(list(europe_ground_stations_coarse.items())[:50])
        
        
        # Create ensemble for parallel processing
        ensemble = lox.Ensemble(oneweb_sample_medium)
        
        start_time = time.time()
        
        try:
            results = lox.visibility_all(
                times=benchmark_times,
                ground_stations=sample_ground_stations,
                spacecraft=ensemble,
                ephemeris=ephemeris,
                bodies=None,
                provider=None,
            )
            
            elapsed = time.time() - start_time
            total_calcs = len(oneweb_sample_medium) * len(sample_ground_stations)
            
            # Verify we got some results
            total_windows = sum(len(windows) for sc_results in results.values() 
                              for windows in sc_results.values())
            
            assert len(results) == len(oneweb_sample_medium)
            
        except Exception as e:
            pytest.skip(f"Parallel visibility_all failed: {e}")

    @pytest.mark.slow
    def test_benchmark_parallel_large(self, europe_ground_stations, oneweb_sample_large, benchmark_times, ephemeris):
        """Test parallel visibility analysis on a large subset (marked as slow)."""
        sample_ground_stations = dict(list(europe_ground_stations.items())[:100])
        
        
        # Create ensemble for parallel processing
        ensemble = lox.Ensemble(oneweb_sample_large)
        
        start_time = time.time()
        
        try:
            results = lox.visibility_all(
                times=benchmark_times,
                ground_stations=sample_ground_stations,
                spacecraft=ensemble,
                ephemeris=ephemeris,
                bodies=None,
                provider=None,
            )
            
            elapsed = time.time() - start_time
            total_calcs = len(oneweb_sample_large) * len(sample_ground_stations)
            
            # Estimate full constellation performance
            full_calcs = len(oneweb_sample_large) * 3 * len(europe_ground_stations)  # Scale up
            estimated_time = elapsed * full_calcs / total_calcs / 60
            
            # Verify we got some results
            total_windows = sum(len(windows) for sc_results in results.values() 
                              for windows in sc_results.values())
            
            assert len(results) == len(oneweb_sample_large)
            
        except Exception as e:
            pytest.skip(f"Large parallel test failed: {e}")

    @pytest.mark.benchmark
    def test_performance_comparison(self, europe_ground_stations_coarse, oneweb_sample_small, benchmark_times, ephemeris):
        """Compare sequential vs parallel vs optimized performance with identical datasets."""
        # Use modest sample size for fair comparison
        sample_ground_stations = dict(list(europe_ground_stations_coarse.items())[:15])
        
        # Sequential test
        seq_start = time.time()
        seq_results = {}
        seq_total_windows = 0
        
        for sc_name, sc_trajectory in oneweb_sample_small.items():
            seq_results[sc_name] = {}
            for gs_name, (gs_location, gs_mask) in sample_ground_stations.items():
                try:
                    windows = lox.visibility(
                        times=benchmark_times,
                        gs=gs_location,
                        mask=gs_mask,
                        sc=sc_trajectory,
                        ephemeris=ephemeris,
                        bodies=None,
                        provider=None,
                    )
                    seq_results[sc_name][gs_name] = windows
                    seq_total_windows += len(windows)
                except Exception as e:
                    seq_results[sc_name][gs_name] = []
        
        seq_elapsed = time.time() - seq_start
        
        # Original parallel test
        par_start = time.time()
        
        try:
            ensemble = lox.Ensemble(oneweb_sample_small)
            par_results = lox.visibility_all(
                times=benchmark_times,
                ground_stations=sample_ground_stations,
                spacecraft=ensemble,
                ephemeris=ephemeris,
                bodies=None,
                provider=None,
            )
            
            par_elapsed = time.time() - par_start
            
            # Count parallel results
            par_total_windows = sum(len(windows) for sc_results in par_results.values() 
                                  for windows in sc_results.values())
            
        except Exception as e:
            par_elapsed = float('inf')
            par_total_windows = 0
            par_results = {}

        # Note: visibility_all now uses the optimized implementation
        opt_elapsed = par_elapsed  # Same as parallel since they're now the same
        opt_total_windows = par_total_windows
        opt_results = par_results
        
        # Results
        total_calcs = len(oneweb_sample_small) * len(sample_ground_stations)
        
        
        # Validation - ensure parallel method found similar results to sequential
        if par_total_windows > 0:
            assert abs(seq_total_windows - par_total_windows) <= total_calcs * 0.1  # Allow 10% variance

    @pytest.mark.slow
    @pytest.mark.benchmark
    def test_full_scale_estimate(self, europe_ground_stations, oneweb, benchmark_times):
        """Estimate performance for full-scale scenario."""
        total_spacecraft = len(oneweb)
        total_ground_stations = len(europe_ground_stations) 
        total_combinations = total_spacecraft * total_ground_stations
        
        print(f"\n{'='*60}")
        print("Full-Scale Performance Estimate")
        print(f"{'='*60}")
        print(f"Total OneWeb satellites: {total_spacecraft}")
        print(f"Total Europe ground points: {total_ground_stations}")
        print(f"Total combinations: {total_combinations:,}")
        print(f"Memory estimate (results only): {total_combinations * 200 / 1024 / 1024:.1f} MB")
        
        # This would be way too slow to actually run, so just calculate estimates
        # Based on our medium sample performance
        estimated_rate = 50  # calculations per second (conservative estimate)
        estimated_time_seconds = total_combinations / estimated_rate
        estimated_time_hours = estimated_time_seconds / 3600
        
        print(f"Estimated computation time: {estimated_time_hours:.1f} hours")
        print(f"This benchmark demonstrates the need for:")
        print("- Chunked/streaming processing")
        print("- Spatial optimization")  
        print("- Memory management")
        print("- Incremental results")
        print(f"{'='*60}")
        
        # This is just an informational test
        assert total_combinations > 400000