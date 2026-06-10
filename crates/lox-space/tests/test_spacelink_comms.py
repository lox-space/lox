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
    | lox.CascadeReceiver.chain_noise_temperature()      | spacelink.core.noise.noise_factor_to_temperature()      |
    | lox.LinkStats.for_link() noise_power               | spacelink.core.noise.noise_power()                      |
    | lox.Channel.eb_n0()                                | spacelink.core.noise.cn0_to_ebn0()                      |
    | lox noise_figure round-trip                        | spacelink.core.noise.temperature_to_noise_figure()      |
    | lox noise_power_density (via k_B constant)         | spacelink.core.noise.noise_power_density()              |
    | lox G/T decomposition                              | spacelink.core.antenna.gain_from_g_over_t()             |
    | lox.DipolePattern.peak_gain()                      | (no spacelink equivalent — see internal unit tests)     |
    | lox.LinkStats.for_link() C/N0                      | (no spacelink equivalent)                               |
"""

import math

import pytest

import lox_space as lox

spacelink = pytest.importorskip("spacelink")

from spacelink.core.path import free_space_path_loss as sl_fspl
from spacelink.core.path import spreading_loss as sl_spreading_loss
from spacelink.core.path import aperture_loss as sl_aperture_loss
from spacelink.core.antenna import dish_gain as sl_dish_gain
from spacelink.core.antenna import gain_from_g_over_t as sl_gain_from_gt
from spacelink.core.noise import noise_factor_to_temperature as sl_nf_to_temp
from spacelink.core.noise import noise_power_density as sl_npd
from spacelink.core.noise import temperature_to_noise_figure as sl_temp_to_nf
from spacelink.core.noise import cn0_to_ebn0 as sl_cn0_to_ebn0
from spacelink.core.noise import noise_power as sl_noise_power
import astropy.units as u


BOLTZMANN_CONSTANT = 1.380649e-23
WIDE_BAND = lox.FrequencyRange(1.0 * lox.GHz, 31.0 * lox.GHz)


def noise_temp_link_stats(t_sys_k, bandwidth_hz, rx_gain_db=30.0):
    """Evaluates a 10 GHz link against a known-T_sys receiver and returns LinkStats."""
    tx_payload, tx_terminal = lox.CommsPayload.transmitter_only(
        "tx",
        lox.ConstantAntenna(gain=30.0 * lox.dB),
        lox.AmplifierTransmitter(band=WIDE_BAND, power=10.0 * lox.W),
        feed_loss=0.0 * lox.dB,
    )
    rx_payload, rx_terminal = lox.CommsPayload.receiver_only(
        "rx",
        lox.ConstantAntenna(gain=rx_gain_db * lox.dB),
        lox.NoiseTempReceiver(
            band=WIDE_BAND, noise_temperature=t_sys_k * lox.K
        ),
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=0.0 * lox.dB,
    )
    return lox.LinkStats.for_link(
        tx_payload,
        tx_terminal,
        rx_payload,
        rx_terminal,
        carrier=10.0 * lox.GHz,
        bandwidth=bandwidth_hz * lox.Hz,
        range=1000.0 * lox.km,
        direction="downlink",
    )


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
# T = (10^(NF/10) − 1) × 290 K.
# Verified via from_lna_and_noise_figure with zero LNA contribution.
# ---------------------------------------------------------------------------

# (noise_figure_db,)
NF_CASES = [0.1, 1.0, 2.5, 5.0, 8.0, 15.0]


@pytest.mark.parametrize("noise_figure_db", NF_CASES)
def test_noise_temperature(noise_figure_db):
    """Noise figure to temperature should agree between lox and spacelink to 0.01 K."""
    # Build a receiver with only the NF stage (LNA has zero noise, unity gain)
    rx = lox.CascadeReceiver.from_lna_and_noise_figure(
        band=WIDE_BAND,
        lna_gain=0.0 * lox.dB,
        lna_noise_temperature=0.0 * lox.K,
        receiver_noise_figure=noise_figure_db * lox.dB,
    )
    # T_chain = 0 + T_rx/1 = T_rx
    lox_t = rx.chain_noise_temperature().to_kelvin()
    sl_t = float(sl_nf_to_temp(noise_figure_db * u.dB).value)
    assert lox_t == pytest.approx(sl_t, abs=0.01)


# ---------------------------------------------------------------------------
# Test 6: chain_noise_temperature() degenerate case
#
# A single-stage chain's noise temperature should equal the receiver noise
# temperature from the noise figure alone.
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("noise_figure_db", NF_CASES)
def test_chain_noise_temperature_degenerate(noise_figure_db):
    """A single-stage cascade must reproduce the bare receiver noise temperature."""
    rx = lox.CascadeReceiver(
        band=WIDE_BAND,
        stages=[
            lox.NoiseStage(
                gain=0.0 * lox.dB,
                noise_temperature=lox.Temperature(290.0 * (10 ** (noise_figure_db / 10) - 1)),
            ),
        ],
    )
    lox_t = rx.chain_noise_temperature().to_kelvin()
    sl_t = float(sl_nf_to_temp(noise_figure_db * u.dB).value)
    assert lox_t == pytest.approx(sl_t, abs=0.01)


# ---------------------------------------------------------------------------
# Test 7: C/N0 → Eb/N0
#
# lox.Channel.eb_n0(cn0) computes Eb/N0 = C/N0 − 10·log10(R).
# spacelink.core.noise.cn0_to_ebn0() implements the same relation.
# ---------------------------------------------------------------------------

# (cn0_dbhz, data_rate_hz)
# Using BPSK with fec=1.0 so that symbol_rate == data_rate and
# Eb/N0 = C/N0 - 10*log10(data_rate), matching the spacelink formula.
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
        symbol_rate=lox.Frequency(data_rate_hz),
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
        fec=1.0,
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
# Test 8: LinkStats.for_link() noise_power vs spacelink
#
# spacelink.core.noise.noise_power(bandwidth, temperature) computes k_B·T·BW
# in watts. LinkStats.noise_power is the same quantity in dBW as
# 10·log₁₀(T_sys · k_B · BW).
# ---------------------------------------------------------------------------

# (system_noise_temperature_k, bandwidth_hz)
NOISE_POWER_CASES = [
    (500.0, 1e6),  # ~-141.6 dBW
    (290.0, 10e6),  # ~-134.0 dBW  (room temperature, 10 MHz)
    (1000.0, 100e6),  # ~-118.6 dBW
]


def t_sys_from_stats(stats, bandwidth_hz):
    """Recovers T_sys in Kelvin from the noise power reported by LinkStats."""
    return 10.0 ** (float(stats.noise_power) / 10.0) / (
        BOLTZMANN_CONSTANT * bandwidth_hz
    )


@pytest.mark.parametrize("t_sys_k,bandwidth_hz", NOISE_POWER_CASES)
def test_noise_power(t_sys_k, bandwidth_hz):
    """LinkStats.noise_power should agree with spacelink to 0.001 dBW."""
    stats = noise_temp_link_stats(t_sys_k, bandwidth_hz)
    lox_dbw = float(stats.noise_power)
    sl_w = float(sl_noise_power(bandwidth_hz * u.Hz, t_sys_k * u.K).to(u.W).value)
    sl_dbw = 10.0 * math.log10(sl_w)
    assert lox_dbw == pytest.approx(sl_dbw, abs=1e-3)


# ---------------------------------------------------------------------------
# Test 9: Noise figure round-trip
#
# lox: NF → T via CascadeReceiver.from_lna_and_noise_figure
# spacelink: T → NF via temperature_to_noise_figure
# The round-trip should recover the original NF.
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("noise_figure_db", NF_CASES)
def test_noise_figure_round_trip(noise_figure_db):
    """NF → T (lox) → NF (spacelink) should recover the original noise figure."""
    # A zero-gain, zero-temperature LNA leaves the chain temperature equal to
    # the receiver stage's NF-derived temperature.
    rx = lox.CascadeReceiver.from_lna_and_noise_figure(
        band=WIDE_BAND,
        lna_gain=0.0 * lox.dB,
        lna_noise_temperature=0.0 * lox.K,
        receiver_noise_figure=noise_figure_db * lox.dB,
    )
    lox_t = rx.chain_noise_temperature().to_kelvin()
    recovered_nf = float(sl_temp_to_nf(lox_t * u.K).value)
    assert recovered_nf == pytest.approx(noise_figure_db, abs=1e-4)


# ---------------------------------------------------------------------------
# Test 10: Noise power density (Boltzmann constant agreement)
#
# lox noise_power at BW=1Hz in dBW should equal spacelink noise_power_density
# in dBW/Hz, validating that both use the same k_B.
# ---------------------------------------------------------------------------

NPD_CASES = [150.0, 290.0, 500.0, 1000.0]


@pytest.mark.parametrize("t_sys_k", NPD_CASES)
def test_noise_power_density(t_sys_k):
    """k_B · T should agree between lox and spacelink to 0.001 dB."""
    stats = noise_temp_link_stats(t_sys_k, 1.0)
    # noise_power(BW=1Hz) = 10·log₁₀(k_B · T · 1) = noise power density in dBW/Hz
    lox_dbw_hz = float(stats.noise_power)
    sl_w_hz = float(sl_npd(t_sys_k * u.K).to(u.W / u.Hz).value)
    sl_dbw_hz = 10.0 * math.log10(sl_w_hz)
    assert lox_dbw_hz == pytest.approx(sl_dbw_hz, abs=1e-3)


# ---------------------------------------------------------------------------
# Test 11: G/T decomposition
#
# Compute G/T in lox (via LinkStats), then use spacelink gain_from_g_over_t
# to recover the antenna gain from G/T and T_sys. This validates that lox's
# G/T formula is consistent with the standard definition.
# ---------------------------------------------------------------------------

GT_CASES = [
    # (antenna_gain_db, t_sys_k)
    (30.0, 500.0),
    (40.0, 290.0),
    (20.0, 1000.0),
]


@pytest.mark.parametrize("gain_db,t_sys_k", GT_CASES)
def test_gt_decomposition(gain_db, t_sys_k):
    """G/T computed by lox should decompose correctly via spacelink."""
    stats = noise_temp_link_stats(t_sys_k, 1e6, rx_gain_db=gain_db)
    lox_gt = float(stats.gt)
    # Use spacelink to recover gain from G/T and T_sys
    recovered_gain = float(
        sl_gain_from_gt(lox_gt * u.dB(1 / u.K), t_sys_k * u.K).value
    )
    assert recovered_gain == pytest.approx(gain_db, abs=1e-3)


# ---------------------------------------------------------------------------
# Test 12: Friis cascade with feed loss — stage-level validation
#
# Build a cascade receive chain behind a lossy port feed and verify each
# stage's noise temperature individually against spacelink, then verify the
# Friis cascade T_sys (port feed synthesized at link setup) against hand
# calculation.
# ---------------------------------------------------------------------------


def test_friis_cascade_feed_loss():
    """Friis cascade with feed loss: stage temperatures should match spacelink,
    and T_sys should match the hand-computed Friis formula."""
    feed_loss_db = 3.0
    rx_nf_db = 5.0
    t_ant = 265.0
    bandwidth_hz = 1e6

    # Receiver stage temperature from its noise figure
    t_rx = 290.0 * (10.0 ** (rx_nf_db / 10.0) - 1.0)
    sl_t_rx = float(sl_nf_to_temp(rx_nf_db * u.dB).value)
    assert t_rx == pytest.approx(sl_t_rx, abs=0.01)

    # Feed stage: passive attenuator at 290K → T_feed = 290*(L-1)
    l_linear = 10.0 ** (feed_loss_db / 10.0)
    t_feed = 290.0 * (l_linear - 1.0)
    # Verify feed temperature against spacelink NF→T (feed NF = feed loss for passive)
    sl_t_feed = float(sl_nf_to_temp(feed_loss_db * u.dB).value)
    assert t_feed == pytest.approx(sl_t_feed, abs=0.01)

    # The chain holds one 20 dB stage at T_rx; the feed lives on the port.
    chain = lox.CascadeReceiver(
        band=WIDE_BAND,
        stages=[lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=t_rx * lox.K)],
    )
    tx_payload, tx_terminal = lox.CommsPayload.transmitter_only(
        "tx",
        lox.ConstantAntenna(gain=30.0 * lox.dB),
        lox.AmplifierTransmitter(band=WIDE_BAND, power=10.0 * lox.W),
        feed_loss=0.0 * lox.dB,
    )
    rx_payload, rx_terminal = lox.CommsPayload.receiver_only(
        "rx",
        lox.ConstantAntenna(gain=30.0 * lox.dB),
        chain,
        antenna_noise_temperature=t_ant * lox.K,
        feed_loss=feed_loss_db * lox.dB,
    )
    stats = lox.LinkStats.for_link(
        tx_payload,
        tx_terminal,
        rx_payload,
        rx_terminal,
        carrier=10.0 * lox.GHz,
        bandwidth=bandwidth_hz * lox.Hz,
        range=1000.0 * lox.km,
        direction="downlink",
    )
    lox_t_sys = t_sys_from_stats(stats, bandwidth_hz)

    # Friis: T_sys = T_ant + T_feed + T_rx / G_feed
    # G_feed = 1/L (feed is lossy, gain = -loss)
    g_feed_linear = 1.0 / l_linear
    t_sys_expected = t_ant + t_feed + t_rx / g_feed_linear
    assert lox_t_sys == pytest.approx(t_sys_expected, abs=0.01)


# ---------------------------------------------------------------------------
# Test 13: TDRS-style Ka-band return link budget
#
# A realistic LEO-to-GEO relay scenario with parameters derived from public
# TDRS specifications. Each intermediate value is cross-checked against
# spacelink where possible.
# ---------------------------------------------------------------------------


def test_tdrs_ka_band_return_link():
    """End-to-end Ka-band link budget with a cascade receive chain, validated
    step-by-step against spacelink building blocks."""
    freq_ghz = 25.5
    freq = freq_ghz * lox.GHz

    # --- Transmitter (LEO user spacecraft) ---
    tx_diameter = 0.6  # m
    tx_efficiency = 0.55
    tx_power_w = 10.0
    tx_feed_loss_db = 1.0

    tx_pattern = lox.ParabolicPattern(tx_diameter * lox.m, tx_efficiency)
    tx_antenna = lox.PatternedAntenna(pattern=tx_pattern)
    tx_payload, tx_terminal = lox.CommsPayload.transmitter_only(
        "leo user",
        tx_antenna,
        lox.AmplifierTransmitter(band=WIDE_BAND, power=tx_power_w * lox.W),
        feed_loss=tx_feed_loss_db * lox.dB,
    )

    # Cross-check TX antenna gain against spacelink
    lox_tx_gain = float(tx_pattern.peak_gain(freq))
    sl_tx_gain = float(
        sl_dish_gain(
            tx_diameter * u.m,
            freq_ghz * u.GHz,
            tx_efficiency * u.dimensionless_unscaled,
        ).value
    )
    assert lox_tx_gain == pytest.approx(sl_tx_gain, abs=1e-4)

    # --- Receiver (GEO relay satellite) ---
    rx_diameter = 4.6  # m
    rx_efficiency = 0.55
    t_ant = 150.0  # K
    lna_gain_db = 25.0
    lna_noise_temp = 75.0  # K
    rx_nf_db = 3.0

    rx_pattern = lox.ParabolicPattern(rx_diameter * lox.m, rx_efficiency)
    rx_antenna = lox.PatternedAntenna(pattern=rx_pattern)
    rx = lox.CascadeReceiver.from_lna_and_noise_figure(
        band=WIDE_BAND,
        lna_gain=lna_gain_db * lox.dB,
        lna_noise_temperature=lna_noise_temp * lox.K,
        receiver_noise_figure=rx_nf_db * lox.dB,
    )
    rx_payload, rx_terminal = lox.CommsPayload.receiver_only(
        "geo relay",
        rx_antenna,
        rx,
        antenna_noise_temperature=t_ant * lox.K,
        feed_loss=0.0 * lox.dB,
    )

    # Cross-check RX antenna gain against spacelink
    lox_rx_gain = float(rx_pattern.peak_gain(freq))
    sl_rx_gain = float(
        sl_dish_gain(
            rx_diameter * u.m,
            freq_ghz * u.GHz,
            rx_efficiency * u.dimensionless_unscaled,
        ).value
    )
    assert lox_rx_gain == pytest.approx(sl_rx_gain, abs=1e-4)

    # Cross-check receiver noise temperature components
    t_rx = 290.0 * (10.0 ** (rx_nf_db / 10.0) - 1.0)
    sl_t_rx = float(sl_nf_to_temp(rx_nf_db * u.dB).value)
    assert t_rx == pytest.approx(sl_t_rx, abs=0.01)

    # --- Channel ---
    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=25e6 * lox.Hz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        fec=0.5,
        roll_off=0.35,
    )

    # --- Link budget ---
    range_km = 40000.0
    bandwidth_hz = float(channel.bandwidth())
    stats = lox.LinkStats.for_link(
        tx_payload,
        tx_terminal,
        rx_payload,
        rx_terminal,
        carrier=freq,
        bandwidth=channel.bandwidth(),
        range=range_km * lox.km,
        direction="downlink",
    )
    modulated = channel.apply(stats)

    # Verify T_sys via Friis: T_ant + T_LNA + T_rx/G_LNA (zero feed loss)
    g_lna = 10.0 ** (lna_gain_db / 10.0)
    t_sys_expected = t_ant + lna_noise_temp + t_rx / g_lna
    lox_t_sys = t_sys_from_stats(stats, bandwidth_hz)
    assert lox_t_sys == pytest.approx(t_sys_expected, abs=0.01)

    # Cross-check FSPL against spacelink
    sl_fspl_db = float(sl_fspl(range_km * u.km, freq_ghz * u.GHz).value)
    assert float(stats.fspl) == pytest.approx(sl_fspl_db, abs=1e-4)

    # Cross-check Eb/N0 against spacelink
    # For QPSK with FEC=0.5: information_rate = symbol_rate * 2 * 0.5 = symbol_rate
    information_rate_hz = 25e6
    sl_ebn0 = float(
        sl_cn0_to_ebn0(
            u.LogQuantity(float(stats.c_n0), unit=u.dB(u.Hz)),
            information_rate_hz * u.Hz,
        ).value
    )
    assert float(modulated.eb_n0) == pytest.approx(sl_ebn0, abs=1e-4)

    # Verify G/T via spacelink gain recovery
    recovered_gain = float(
        sl_gain_from_gt(
            float(stats.gt) * u.dB(1 / u.K), lox_t_sys * u.K
        ).value
    )
    # G/T uses antenna gain only (no chain gain), so recovered gain
    # should equal the RX antenna peak gain
    assert recovered_gain == pytest.approx(lox_rx_gain, abs=0.1)

    # Sanity checks: the link should close with these parameters
    assert float(modulated.margin) > 0.0, "TDRS Ka-band return link should close"
    assert float(stats.c_n0) > 60.0, "C/N0 should be reasonable for Ka-band relay"
