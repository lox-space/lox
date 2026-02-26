# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
# SPDX-FileCopyrightText: 2026 Torkil Rein Gustavsen <torkilrg@ksat.no>
#
# SPDX-License-Identifier: MPL-2.0

"""
Validate lox-comms RF calculations against the spacelink library.

Both lox and spacelink derive from the same RF physics (Friis transmission
equation, uniform circular aperture, Boltzmann noise). These tests confirm
numerical agreement between the two independent implementations.

spacelink function mapping:
    | lox-comms                                          | spacelink                                               |
    |----------------------------------------------------|---------------------------------------------------------|
    | lox.fspl()                                         | spacelink.core.path.free_space_path_loss()              |
    | lox.fspl() (combined)                              | spreading_loss() + aperture_loss()                      |
    | lox.ParabolicPattern.peak_gain()                   | spacelink.core.antenna.dish_gain()                      |
    | lox.GaussianPattern.peak_gain()                    | spacelink.core.antenna.dish_gain()                      |
    | lox.ComplexReceiver.noise_temperature()            | spacelink.core.noise.noise_factor_to_temperature()      |
    | lox.ComplexReceiver.system_noise_temperature()     | (degenerate case only → noise_factor_to_temperature())  |
    | lox.CommunicationSystem.noise_power()              | spacelink.core.noise.noise_power()                      |
    | lox.Channel.eb_n0()                                | spacelink.core.noise.cn0_to_ebn0()                      |
    | lox.DipolePattern.peak_gain()                      | (no spacelink equivalent — see internal unit tests)     |
    | lox.Transmitter.eirp()                             | (no spacelink equivalent)                               |
    | lox.CommunicationSystem.carrier_to_noise_density() | (no spacelink equivalent)                               |
"""

import math

import pytest

import lox_space as lox

spacelink = pytest.importorskip("spacelink")

from spacelink.core.path import free_space_path_loss as sl_fspl
from spacelink.core.path import spreading_loss as sl_spreading_loss
from spacelink.core.path import aperture_loss as sl_aperture_loss
from spacelink.core.antenna import dish_gain as sl_dish_gain
from spacelink.core.noise import noise_factor_to_temperature as sl_nf_to_temp
from spacelink.core.noise import cn0_to_ebn0 as sl_cn0_to_ebn0
from spacelink.core.noise import noise_power as sl_noise_power
import astropy.units as u


# ---------------------------------------------------------------------------
# Test 1: Free-space path loss
# ---------------------------------------------------------------------------

# (distance_km, frequency_ghz)
FSPL_CASES = [
    (1000.0, 2.0),  # S-band, LEO range
    (35786.0, 10.7),  # X-band, GEO
    (500.0, 29.0),  # Ka-band, LEO
]


@pytest.mark.parametrize("distance_km,frequency_ghz", FSPL_CASES)
def test_fspl(distance_km, frequency_ghz):
    """FSPL should agree between lox and spacelink to better than 1e-4 dB."""
    lox_db = float(lox.fspl(distance_km * lox.km, frequency_ghz * lox.GHz))
    sl_db = float(sl_fspl(distance_km * u.km, frequency_ghz * u.GHz).value)
    assert lox_db == pytest.approx(sl_db, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 2: FSPL Friis decomposition
#
# spacelink decomposes FSPL into a distance-only term (spreading loss) and a
# frequency-only term (aperture loss). Their sum must equal lox's combined
# result, validating agreement from a different computational angle.
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("distance_km,frequency_ghz", FSPL_CASES)
def test_fspl_decomposition(distance_km, frequency_ghz):
    """lox FSPL should equal spacelink's spreading_loss + aperture_loss."""
    lox_db = float(lox.fspl(distance_km * lox.km, frequency_ghz * lox.GHz))
    sl_db = (
        sl_spreading_loss(distance_km * u.km).value
        + sl_aperture_loss(frequency_ghz * u.GHz).value
    )
    assert lox_db == pytest.approx(sl_db, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 3: Parabolic dish peak gain
# ---------------------------------------------------------------------------

# (diameter_m, frequency_ghz, efficiency)
DISH_CASES = [
    (0.98, 29.0, 0.45),  # matches lox internal test (expected: 46.011 dBi)
    (0.6, 29.0, 0.65),  # small Ka-band dish
    (2.4, 12.0, 0.60),  # medium Ku-band dish
]


@pytest.mark.parametrize("diameter_m,frequency_ghz,efficiency", DISH_CASES)
def test_parabolic_peak_gain(diameter_m, frequency_ghz, efficiency):
    """Parabolic peak gain should agree between lox and spacelink to better than 1e-4 dB."""
    lox_db = float(
        lox.ParabolicPattern(diameter_m * lox.m, efficiency).peak_gain(
            frequency_ghz * lox.GHz
        )
    )
    sl_db = float(
        sl_dish_gain(
            diameter_m * u.m,
            frequency_ghz * u.GHz,
            efficiency * u.dimensionless_unscaled,
        ).value
    )
    assert lox_db == pytest.approx(sl_db, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 4: Gaussian antenna peak gain
#
# GaussianPattern uses the same η(πD/λ)² peak-gain formula as ParabolicPattern;
# only the off-axis rolloff differs. Peak gain should therefore also match
# spacelink's dish_gain().
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("diameter_m,frequency_ghz,efficiency", DISH_CASES)
def test_gaussian_peak_gain(diameter_m, frequency_ghz, efficiency):
    """Gaussian peak gain should agree with spacelink dish_gain to better than 1e-4 dB."""
    lox_db = float(
        lox.GaussianPattern(diameter_m * lox.m, efficiency).peak_gain(
            frequency_ghz * lox.GHz
        )
    )
    sl_db = float(
        sl_dish_gain(
            diameter_m * u.m,
            frequency_ghz * u.GHz,
            efficiency * u.dimensionless_unscaled,
        ).value
    )
    assert lox_db == pytest.approx(sl_db, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 5: Noise figure → noise temperature
#
# lox.ComplexReceiver.noise_temperature() returns the noise temperature of the
# back-end receive chain, computed as T = (10^(NF/10) − 1) × 290 K.
# lna_gain_db and lna_noise_figure_db have no effect on noise_temperature(),
# which only reads self.noise_figure.
# ---------------------------------------------------------------------------

# (noise_figure_db,)
NF_CASES = [0.1, 1.0, 2.5, 5.0, 8.0, 15.0]


@pytest.mark.parametrize("noise_figure_db", NF_CASES)
def test_noise_temperature(noise_figure_db):
    """Backend noise temperature should agree between lox and spacelink to 0.01 K."""
    rx = lox.ComplexReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=0.0 * lox.K,
        lna_gain=1.0 * lox.dB,  # not used by noise_temperature()
        lna_noise_figure=1.0 * lox.dB,  # not used by noise_temperature()
        noise_figure=noise_figure_db * lox.dB,
        loss=0.0 * lox.dB,
    )
    lox_t = rx.noise_temperature().to_kelvin()
    sl_t = float(sl_nf_to_temp(noise_figure_db * u.dB).value)
    assert lox_t == pytest.approx(sl_t, abs=0.01)


# ---------------------------------------------------------------------------
# Test 6: system_noise_temperature() degenerate case
#
# With loss_db=0 and antenna_noise_temperature_k=0 the formula
# T_sys = T_ant·L + T_room·(1−L) + T_rx reduces to T_rx alone, so
# system_noise_temperature() must equal sl_nf_to_temp(noise_figure_db).
# This exercises a different code path from Test 5 and guards against bugs
# in the loss/antenna-temperature terms.
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("noise_figure_db", NF_CASES)
def test_system_noise_temperature_degenerate(noise_figure_db):
    """With no loss and no antenna temperature, system_noise_temperature()
    must equal the bare receiver noise temperature returned by
    spacelink.core.noise.noise_factor_to_temperature()."""
    rx = lox.ComplexReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=0.0 * lox.K,
        lna_gain=1.0 * lox.dB,
        lna_noise_figure=1.0 * lox.dB,
        noise_figure=noise_figure_db * lox.dB,
        loss=0.0 * lox.dB,
    )
    lox_t = rx.system_noise_temperature().to_kelvin()
    sl_t = float(sl_nf_to_temp(noise_figure_db * u.dB).value)
    assert lox_t == pytest.approx(sl_t, abs=0.01)


# ---------------------------------------------------------------------------
# Test 7: C/N0 → Eb/N0
#
# lox.Channel.eb_n0(cn0) computes Eb/N0 = C/N0 − 10·log10(R).
# spacelink.core.noise.cn0_to_ebn0() implements the same relation.
# ---------------------------------------------------------------------------

# (cn0_dbhz, data_rate_hz)
CN0_CASES = [
    (70.0, 1e6),  # 1 Mbit/s   → Eb/N0 = 70 - 60 = 10 dB
    (80.0, 1e6),  # 1 Mbit/s   → Eb/N0 = 80 - 60 = 20 dB
    (90.0, 10e6),  # 10 Mbit/s  → Eb/N0 = 90 - 70 = 20 dB
    (73.5, 2.5e6),  # 2.5 Mbit/s → Eb/N0 = 73.5 − 10·log₁₀(2.5e6) ≈ 9.52 dB
]


@pytest.mark.parametrize("cn0_dbhz,data_rate_hz", CN0_CASES)
def test_cn0_to_ebn0(cn0_dbhz, data_rate_hz):
    ch = lox.Channel(
        link_type="downlink",
        data_rate=data_rate_hz * lox.bps,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    lox_ebn0 = float(ch.eb_n0(cn0_dbhz * lox.dB))
    sl_ebn0 = float(
        sl_cn0_to_ebn0(
            u.LogQuantity(cn0_dbhz, unit=u.dB(u.Hz)),
            data_rate_hz * u.Hz,
        ).value
    )
    assert lox_ebn0 == pytest.approx(sl_ebn0, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 8: CommunicationSystem.noise_power() vs spacelink
#
# spacelink.core.noise.noise_power(bandwidth, temperature) computes k_B·T·BW
# in watts. CommunicationSystem.noise_power() returns the same quantity in
# dBW as 10·log₁₀(T_sys · k_B · BW).
# ---------------------------------------------------------------------------

# (system_noise_temperature_k, bandwidth_hz)
NOISE_POWER_CASES = [
    (500.0, 1e6),  # ~-141.6 dBW
    (290.0, 10e6),  # ~-134.0 dBW  (room temperature, 10 MHz)
    (1000.0, 100e6),  # ~-118.6 dBW
]


@pytest.mark.parametrize("t_sys_k,bandwidth_hz", NOISE_POWER_CASES)
def test_noise_power(t_sys_k, bandwidth_hz):
    """CommunicationSystem.noise_power() should agree with spacelink to 0.001 dBW."""
    rx_sys = lox.CommunicationSystem(
        lox.SimpleAntenna(gain=30.0 * lox.dB, beamwidth=5.0 * lox.deg),
        receiver=lox.SimpleReceiver(
            frequency=10 * lox.GHz,
            system_noise_temperature=t_sys_k * lox.K,
        ),
    )
    lox_dbw = float(rx_sys.noise_power(bandwidth_hz * lox.Hz))
    sl_w = float(sl_noise_power(bandwidth_hz * u.Hz, t_sys_k * u.K).to(u.W).value)
    sl_dbw = 10.0 * math.log10(sl_w)
    assert lox_dbw == pytest.approx(sl_dbw, abs=1e-3)
