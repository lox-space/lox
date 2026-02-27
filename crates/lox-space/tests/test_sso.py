# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math

import pytest

import lox_space as lox

# Reference values from lox-orbits Rust tests.
SMA_KM = 7178.1363
EARTH_R_KM = 6378.1366
EXP_INC_RAD = math.radians(98.627)
EXP_RAAN_RAD = math.radians(350.5997)


@pytest.fixture
def epoch():
    return lox.UTC.from_iso("2020-02-18T18:44:37.550").to_scale("TDB")


def test_sso_from_semi_major_axis(epoch, provider):
    """SSO from semi-major axis: derived inclination and RAAN match reference."""
    sso = lox.Keplerian.sso(
        epoch,
        semi_major_axis=SMA_KM * lox.km,
        ltan=(13, 30),
        provider=provider,
    )

    assert float(sso.inclination()) == pytest.approx(EXP_INC_RAD, rel=1e-5)
    assert float(sso.longitude_of_ascending_node()) == pytest.approx(
        EXP_RAAN_RAD, rel=4e-3
    )


def test_sso_from_altitude(epoch, provider):
    """SSO from altitude with LTDN gives the same result."""
    altitude = (SMA_KM - EARTH_R_KM) * lox.km

    sso = lox.Keplerian.sso(
        epoch,
        altitude=altitude,
        ltdn=(1, 30),
        provider=provider,
    )

    assert float(sso.inclination()) == pytest.approx(EXP_INC_RAD, rel=1e-5)
    assert float(sso.longitude_of_ascending_node()) == pytest.approx(
        EXP_RAAN_RAD, rel=4e-3
    )


def test_sso_invalid_parameters(epoch, provider):
    """Passing none or multiple of altitude/semi_major_axis/inclination raises."""
    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian.sso(epoch, provider=provider)

    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian.sso(
            epoch,
            altitude=800 * lox.km,
            semi_major_axis=7178 * lox.km,
            provider=provider,
        )


def test_sso_ltan_ltdn_exclusive(epoch, provider):
    """Passing both ltan and ltdn raises."""
    with pytest.raises(ValueError, match="at most one"):
        lox.Keplerian.sso(
            epoch,
            altitude=800 * lox.km,
            ltan=(13, 30),
            ltdn=(1, 30),
            provider=provider,
        )
