# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""
Validate lox-frames reference frame transformations against Astropy.

Astropy is built on ERFA/SOFA and serves as the de facto standard for
astrodynamic reference frame conversions. These tests compare lox frame
transformations against Astropy for the modern CIO-based chain
(ICRF/CIRF/TIRF/ITRF) and the classical equinox-based chain (MOD/TOD/PEF).

Astropy frame mapping:
    | Lox Frame    | Astropy Frame | Notes                              |
    |--------------|---------------|------------------------------------|
    | ICRF         | GCRS          | Equivalent for geocentric          |
    | CIRF         | CIRS          | CIO-based intermediate             |
    | ITRF         | ITRS          | Earth-fixed                        |
    | TOD(IERS2003)| TETE          | True Equator True Equinox (2000A)  |
"""

import numpy as np
import numpy.testing as npt
import pytest

import lox_space as lox

astropy = pytest.importorskip("astropy")

from astropy.coordinates import (
    GCRS,
    CIRS,
    ITRS,
    TETE,
    CartesianRepresentation,
    CartesianDifferential,
)
from astropy.time import Time as AstropyTime
import astropy.units as u

# Test epochs — all within definitive IERS EOP data range
EPOCHS = [
    "2020-01-01T12:00:00",
    "2022-06-21T06:30:00",
    "2023-03-20T21:24:00",
    "2000-01-01T12:00:00",
]

# LEO test state (not axis-aligned, in km and km/s)
POS_KM = (6068.27927, -1692.84394, -2516.61918)
VEL_KMS = (-0.660415582, 5.495938726, -5.303093233)


def _lox_state(epoch_iso, frame="ICRF"):
    """Create a lox State from an ISO epoch string."""
    time = lox.UTC.from_iso(epoch_iso).to_scale("TAI")
    return lox.State(
        time, POS_KM, VEL_KMS, lox.Origin("Earth"), lox.Frame(frame)
    )


def _astropy_gcrs(epoch_iso):
    """Create an Astropy GCRS state from an ISO epoch string."""
    t = AstropyTime(epoch_iso, scale="utc")
    pos = CartesianRepresentation(
        x=POS_KM[0] * u.km, y=POS_KM[1] * u.km, z=POS_KM[2] * u.km
    )
    vel = CartesianDifferential(
        d_x=VEL_KMS[0] * u.km / u.s,
        d_y=VEL_KMS[1] * u.km / u.s,
        d_z=VEL_KMS[2] * u.km / u.s,
    )
    return GCRS(pos.with_differentials(vel), obstime=t)


def _extract_pos_vel(coord):
    """Extract position (km) and velocity (km/s) from an Astropy coordinate."""
    cart = coord.cartesian
    pos = np.array([cart.x.to(u.km).value, cart.y.to(u.km).value, cart.z.to(u.km).value])
    diff = cart.differentials["s"]
    vel = np.array([
        diff.d_x.to(u.km / u.s).value,
        diff.d_y.to(u.km / u.s).value,
        diff.d_z.to(u.km / u.s).value,
    ])
    return pos, vel


# ---------------------------------------------------------------------------
# Test 1: ICRF -> ITRF (full chain: ICRF -> CIRF -> TIRF -> ITRF)
# ---------------------------------------------------------------------------
@pytest.mark.parametrize("epoch", EPOCHS)
def test_icrf_to_itrf(epoch, provider):
    """Validate full terrestrial transformation chain against Astropy GCRS -> ITRS."""
    # Lox
    state_icrf = _lox_state(epoch)
    state_itrf = state_icrf.to_frame(lox.Frame("ITRF"), provider)
    lox_pos = np.array(state_itrf.position())
    lox_vel = np.array(state_itrf.velocity())

    # Astropy
    gcrs = _astropy_gcrs(epoch)
    t = AstropyTime(epoch, scale="utc")
    itrs = gcrs.transform_to(ITRS(obstime=t))
    ap_pos, ap_vel = _extract_pos_vel(itrs)

    npt.assert_allclose(lox_pos, ap_pos, rtol=1e-8, err_msg=f"ITRF position mismatch at {epoch}")
    npt.assert_allclose(lox_vel, ap_vel, rtol=1e-6, err_msg=f"ITRF velocity mismatch at {epoch}")


# ---------------------------------------------------------------------------
# Test 2: ICRF -> CIRF (CIP-based precession-nutation only)
# ---------------------------------------------------------------------------
@pytest.mark.parametrize("epoch", EPOCHS)
def test_icrf_to_cirf(epoch, provider):
    """Validate CIP-based precession-nutation against Astropy GCRS -> CIRS."""
    # Lox
    state_icrf = _lox_state(epoch)
    state_cirf = state_icrf.to_frame(lox.Frame("CIRF"), provider)
    lox_pos = np.array(state_cirf.position())
    lox_vel = np.array(state_cirf.velocity())

    # Astropy
    gcrs = _astropy_gcrs(epoch)
    t = AstropyTime(epoch, scale="utc")
    cirs = gcrs.transform_to(CIRS(obstime=t))
    ap_pos, ap_vel = _extract_pos_vel(cirs)

    npt.assert_allclose(lox_pos, ap_pos, rtol=1e-8, err_msg=f"CIRF position mismatch at {epoch}")
    npt.assert_allclose(lox_vel, ap_vel, rtol=1e-6, err_msg=f"CIRF velocity mismatch at {epoch}")


# ---------------------------------------------------------------------------
# Test 3: ICRF -> TOD (equinox-based, IAU 2000A nutation = Astropy TETE)
# ---------------------------------------------------------------------------
@pytest.mark.parametrize("epoch", EPOCHS)
def test_icrf_to_tod(epoch, provider):
    """Validate equinox-based TOD(IERS2003) against Astropy GCRS -> TETE."""
    # Lox — TOD(IERS2003) uses IAU 2000A nutation, matching Astropy's TETE
    state_icrf = _lox_state(epoch)
    state_tod = state_icrf.to_frame(lox.Frame("TOD(IERS2003)"), provider)
    lox_pos = np.array(state_tod.position())
    lox_vel = np.array(state_tod.velocity())

    # Astropy
    gcrs = _astropy_gcrs(epoch)
    t = AstropyTime(epoch, scale="utc")
    tete = gcrs.transform_to(TETE(obstime=t))
    ap_pos, ap_vel = _extract_pos_vel(tete)

    npt.assert_allclose(lox_pos, ap_pos, rtol=1e-7, err_msg=f"TOD position mismatch at {epoch}")
    npt.assert_allclose(lox_vel, ap_vel, rtol=1e-5, err_msg=f"TOD velocity mismatch at {epoch}")


# ---------------------------------------------------------------------------
# Test 4: Roundtrip tests (Frame -> ICRF -> Frame)
# ---------------------------------------------------------------------------
@pytest.mark.parametrize(
    "frame",
    ["CIRF", "TIRF", "ITRF", "TEME", "MOD", "TOD", "PEF"],
)
def test_roundtrip(frame, provider):
    """Roundtrip transformation should preserve state to high precision."""
    epoch = "2020-01-01T12:00:00"
    state_icrf = _lox_state(epoch)

    state_frame = state_icrf.to_frame(lox.Frame(frame), provider)
    state_back = state_frame.to_frame(lox.Frame("ICRF"), provider)

    npt.assert_allclose(
        state_icrf.position(), state_back.position(),
        rtol=1e-12, err_msg=f"Roundtrip position failed for {frame}",
    )
    npt.assert_allclose(
        state_icrf.velocity(), state_back.velocity(),
        rtol=1e-10, atol=1e-15, err_msg=f"Roundtrip velocity failed for {frame}",
    )


# ---------------------------------------------------------------------------
# Test 5: Velocity sanity check — Earth rotation contribution
# ---------------------------------------------------------------------------
def test_velocity_includes_earth_rotation(provider):
    """ICRF -> ITRF velocity change should reflect Earth rotation effects."""
    epoch = "2020-01-01T12:00:00"
    state_icrf = _lox_state(epoch)
    state_itrf = state_icrf.to_frame(lox.Frame("ITRF"), provider)

    v_icrf_mag = np.linalg.norm(state_icrf.velocity())
    v_itrf_mag = np.linalg.norm(state_itrf.velocity())

    # Rotation preserves position magnitude
    npt.assert_allclose(
        np.linalg.norm(state_icrf.position()),
        np.linalg.norm(state_itrf.position()),
        rtol=1e-10,
    )

    # ITRF velocity magnitude should differ from ICRF by roughly omega x r
    # (~0.5 km/s for LEO), but the total magnitude change depends on geometry.
    # Just verify both are in the expected LEO velocity range (6-9 km/s).
    assert 6.0 < v_icrf_mag < 9.0, f"ICRF velocity {v_icrf_mag:.3f} km/s outside LEO range"
    assert 6.0 < v_itrf_mag < 9.0, f"ITRF velocity {v_itrf_mag:.3f} km/s outside LEO range"
