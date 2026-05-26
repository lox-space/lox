# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""Tests for ITU-R atmospheric propagation models."""

import os
import pathlib

import pytest

import lox_space as lox


@pytest.fixture(scope="session")
def itur_provider() -> lox.ItuProvider:
    path = os.environ.get(
        "LOX_ITUR_BUNDLE",
        str(pathlib.Path(__file__).parents[3].joinpath("target", "lox-itur-data.npz")),
    )
    return lox.ItuProvider(path)


def test_environmental_losses_constructor(itur_provider):
    """Test EnvironmentalLosses constructor."""
    losses = lox.EnvironmentalLosses(
        itur_provider,
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        probability=0.01,
        diameter=1.2 * lox.m,
    )
    assert float(losses.rain) >= 0.0
    assert 0.0 < float(losses.atmospheric) < 30.0


def test_atmospheric_attenuation_slant_path(itur_provider):
    """End-to-end test for the provider method."""
    losses = itur_provider.atmospheric_attenuation_slant_path(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        probability=0.01,
        diameter=1.2 * lox.m,
    )
    # All components should be non-negative
    assert float(losses.rain) >= 0.0
    assert float(losses.gaseous) >= 0.0
    assert float(losses.cloud) >= 0.0
    assert float(losses.scintillation) >= 0.0
    # Total should be reasonable (< 30 dB for Ku-band at 30 deg)
    assert 0.0 < float(losses.atmospheric) < 30.0


def test_atmospheric_attenuation_with_custom_tilt(itur_provider):
    """Test with explicit polarisation tilt angle."""
    losses = itur_provider.atmospheric_attenuation_slant_path(
        lat=51.5 * lox.deg,
        lon=-0.1 * lox.deg,
        frequency=29.0 * lox.GHz,
        elevation=45.0 * lox.deg,
        probability=0.1,
        diameter=0.6 * lox.m,
        polarisation_tilt=0.0 * lox.deg,
    )
    assert float(losses.atmospheric) > 0.0


def test_rain_attenuation(itur_provider):
    """Test rain attenuation for Madrid at Ku-band."""
    a = itur_provider.rain_attenuation(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        probability=0.01,
    )
    assert 0.0 < float(a) < 20.0


def test_gaseous_attenuation():
    """Test gaseous attenuation at sea level."""
    a_o, a_w = lox.gaseous_attenuation_slant_path(
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        pressure=1013.25 * lox.hPa,
        rho=7.5,
        temperature=288.15 * lox.K,
    )
    # Both should be small positive or near-zero at 14.25 GHz
    assert float(a_o) + float(a_w) > 0.0
    # Total gaseous < 1 dB at this frequency and elevation
    assert float(a_o) + float(a_w) < 1.0


def test_cloud_attenuation(itur_provider):
    """Test cloud attenuation."""
    a = itur_provider.cloud_attenuation(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        elevation=30.0 * lox.deg,
        frequency=14.25 * lox.GHz,
        probability=1.0,
    )
    assert float(a) >= 0.0


def test_scintillation_attenuation(itur_provider):
    """Test scintillation attenuation."""
    a = itur_provider.scintillation_attenuation(
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        probability=0.01,
        diameter=1.2 * lox.m,
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
    )
    assert 0.0 < float(a) < 2.0


def test_rain_specific_attenuation():
    """Test specific rain attenuation."""
    gamma = lox.rain_specific_attenuation(
        rain_rate=25.0,
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
    )
    assert gamma > 0.0


def test_topographic_altitude(itur_provider):
    """Test topographic altitude lookup for known locations."""
    alt = itur_provider.topographic_altitude(
        lat=27.99 * lox.deg,
        lon=86.93 * lox.deg,
    )
    # Everest region should be > 5 km
    assert alt.to_kilometers() > 5.0

    alt_sea = itur_provider.topographic_altitude(
        lat=0.0 * lox.deg,
        lon=0.0 * lox.deg,
    )
    # Gulf of Guinea, near sea level
    assert alt_sea.to_kilometers() < 1.0


def test_surface_mean_temperature(itur_provider):
    """Test surface temperature lookup."""
    t = itur_provider.surface_mean_temperature(
        lat=0.0 * lox.deg,
        lon=0.0 * lox.deg,
    )
    # Equatorial temperature should be warm (> 290 K)
    assert t.to_kelvin() > 290.0


def test_rainfall_rate(itur_provider):
    """Test rainfall rate at 0.01% exceedance."""
    r = itur_provider.rainfall_rate(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        probability=0.01,
    )
    # Madrid: moderate rainfall, expect 20-50 mm/h at 0.01%
    assert 5.0 < r < 100.0


def test_rain_height(itur_provider):
    """Test rain height lookup."""
    h = itur_provider.rain_height(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
    )
    # Rain height at mid-latitudes ~2-4 km
    assert 1.0 < h.to_kilometers() < 6.0


def test_environmental_losses_properties(itur_provider):
    """Test that EnvironmentalLosses fields are accessible."""
    losses = itur_provider.atmospheric_attenuation_slant_path(
        lat=40.4 * lox.deg,
        lon=-3.7 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=30.0 * lox.deg,
        probability=0.01,
        diameter=1.2 * lox.m,
    )
    # All properties should return Decibel
    assert float(losses.rain) >= 0.0
    assert float(losses.gaseous) >= 0.0
    assert float(losses.cloud) >= 0.0
    assert float(losses.scintillation) >= 0.0
    # depolarization stores the XPD (negative = discrimination, not a loss)
    assert isinstance(float(losses.depolarization), float)
    # atmospheric field has the ITU-R combined total
    assert float(losses.atmospheric) > 0.0


def test_high_frequency_ka_band(itur_provider):
    """Test at Ka-band (30 GHz) which exercises different code paths."""
    losses = itur_provider.atmospheric_attenuation_slant_path(
        lat=51.5 * lox.deg,
        lon=-0.1 * lox.deg,
        frequency=30.0 * lox.GHz,
        elevation=20.0 * lox.deg,
        probability=0.01,
        diameter=0.6 * lox.m,
    )
    assert float(losses.atmospheric) > 0.0
    # Ka-band should have higher attenuation than Ku-band
    losses_ku = itur_provider.atmospheric_attenuation_slant_path(
        lat=51.5 * lox.deg,
        lon=-0.1 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=20.0 * lox.deg,
        probability=0.01,
        diameter=0.6 * lox.m,
    )
    assert float(losses.atmospheric) > float(losses_ku.atmospheric)


def test_equatorial_location(itur_provider):
    """Test at equatorial location (different climate profile)."""
    losses = itur_provider.atmospheric_attenuation_slant_path(
        lat=0.0 * lox.deg,
        lon=30.0 * lox.deg,
        frequency=14.25 * lox.GHz,
        elevation=45.0 * lox.deg,
        probability=0.01,
        diameter=1.2 * lox.m,
    )
    assert float(losses.atmospheric) > 0.0


def test_pressure_unit():
    """Test Pressure unit type."""
    p = 1013.25 * lox.hPa
    assert abs(p.to_hpa() - 1013.25) < 1e-6
    assert abs(p.to_pa() - 101325.0) < 0.1
