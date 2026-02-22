# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math
import pickle

import pytest
import lox_space as lox


# --- Decibel ---


def test_decibel_new():
    db = lox.Decibel(3.0)
    assert float(db) == 3.0


def test_decibel_from_linear():
    db = lox.Decibel.from_linear(100.0)
    assert float(db) == pytest.approx(20.0, abs=1e-10)


def test_decibel_to_linear():
    db = lox.Decibel(20.0)
    assert db.to_linear() == pytest.approx(100.0, abs=1e-10)


def test_decibel_roundtrip():
    db = lox.Decibel(13.5)
    assert float(lox.Decibel.from_linear(db.to_linear())) == pytest.approx(13.5, abs=1e-10)


def test_decibel_add():
    a = lox.Decibel(3.0)
    b = lox.Decibel(3.0)
    assert float(a + b) == pytest.approx(6.0, abs=1e-10)


def test_decibel_sub():
    a = lox.Decibel(6.0)
    b = lox.Decibel(3.0)
    assert float(a - b) == pytest.approx(3.0, abs=1e-10)


def test_decibel_neg():
    db = lox.Decibel(3.0)
    assert float(-db) == pytest.approx(-3.0, abs=1e-10)


def test_decibel_repr():
    db = lox.Decibel(3.0)
    assert repr(db) == "Decibel(3.0)"


def test_decibel_str():
    db = lox.Decibel(3.0)
    assert str(db) == "3 dB"


def test_decibel_eq():
    assert lox.Decibel(3.0) == lox.Decibel(3.0)
    assert not (lox.Decibel(3.0) == lox.Decibel(4.0))


def test_decibel_pickle():
    db = lox.Decibel(13.5)
    assert pickle.loads(pickle.dumps(db)) == db


def test_decibel_repr_roundtrip():
    db = lox.Decibel(13.5)
    assert eval(repr(db), {"Decibel": lox.Decibel}) == db


# --- Modulation ---


def test_modulation_bits_per_symbol():
    assert lox.Modulation("BPSK").bits_per_symbol() == 1
    assert lox.Modulation("QPSK").bits_per_symbol() == 2
    assert lox.Modulation("8PSK").bits_per_symbol() == 3
    assert lox.Modulation("16QAM").bits_per_symbol() == 4
    assert lox.Modulation("256QAM").bits_per_symbol() == 8


def test_modulation_invalid():
    with pytest.raises(ValueError, match="unknown modulation"):
        lox.Modulation("AM")


def test_modulation_repr():
    m = lox.Modulation("QPSK")
    assert repr(m) == "Modulation('QPSK')"


def test_modulation_eq():
    assert lox.Modulation("BPSK") == lox.Modulation("BPSK")
    assert not (lox.Modulation("BPSK") == lox.Modulation("QPSK"))


def test_modulation_pickle():
    for name in ["BPSK", "QPSK", "8PSK", "16QAM", "32QAM", "64QAM", "128QAM", "256QAM"]:
        m = lox.Modulation(name)
        assert pickle.loads(pickle.dumps(m)) == m


def test_modulation_repr_roundtrip():
    m = lox.Modulation("8PSK")
    assert eval(repr(m), {"Modulation": lox.Modulation}) == m


# --- Parabolic Pattern ---


def test_parabolic_peak_gain():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    gain = p.peak_gain(frequency_hz=29e9)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_parabolic_beamwidth():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    bw = p.beamwidth(frequency_hz=29e9)
    assert bw == pytest.approx(0.7371800, rel=1e-4)


def test_parabolic_on_axis_equals_peak():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    f = 29e9
    gain = p.gain(f, angle_deg=0.0)
    peak = p.peak_gain(f)
    assert float(gain) == pytest.approx(float(peak), abs=1e-6)


def test_parabolic_gain_at_180():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    gain = p.gain(29e9, angle_deg=180.0)
    assert float(gain) < -50.0


def test_parabolic_from_beamwidth_roundtrip():
    p = lox.ParabolicPattern.from_beamwidth(
        beamwidth_deg=math.degrees(0.1), frequency_hz=2e9, efficiency=0.65
    )
    bw = p.beamwidth(frequency_hz=2e9)
    assert bw == pytest.approx(math.degrees(0.1), rel=0.01)


def test_parabolic_eq():
    a = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    b = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    c = lox.ParabolicPattern(diameter_m=1.0, efficiency=0.45)
    assert a == b
    assert not (a == c)


def test_parabolic_pickle():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    assert pickle.loads(pickle.dumps(p)) == p


def test_parabolic_repr_roundtrip():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    assert eval(repr(p), {"ParabolicPattern": lox.ParabolicPattern}) == p


# --- Gaussian Pattern ---


def test_gaussian_peak_gain():
    p = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    gain = p.peak_gain(frequency_hz=29e9)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_gaussian_3db_at_half_beamwidth():
    p = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    f = 29e9
    half_bw = p.beamwidth(f) / 2.0
    peak = float(p.peak_gain(f))
    gain = float(p.gain(f, angle_deg=half_bw))
    assert peak - gain == pytest.approx(3.0103, abs=0.01)


def test_gaussian_eq():
    a = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    b = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    c = lox.GaussianPattern(diameter_m=1.0, efficiency=0.45)
    assert a == b
    assert not (a == c)


def test_gaussian_pickle():
    p = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    assert pickle.loads(pickle.dumps(p)) == p


def test_gaussian_repr_roundtrip():
    p = lox.GaussianPattern(diameter_m=0.98, efficiency=0.45)
    assert eval(repr(p), {"GaussianPattern": lox.GaussianPattern}) == p


# --- Dipole Pattern ---


def test_half_wave_dipole_broadside():
    # Half-wave dipole: peak gain at 90 deg approx 2.15 dBi
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length_m=wavelength / 2.0)
    gain = d.gain(29e9, angle_deg=90.0)
    assert float(gain) == pytest.approx(2.15, abs=0.01)


def test_half_wave_dipole_endfire():
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length_m=wavelength / 2.0)
    gain = d.gain(29e9, angle_deg=0.0)
    assert float(gain) < -50.0


def test_short_dipole_peak():
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length_m=wavelength / 100.0)
    peak = d.peak_gain(29e9)
    assert float(peak) == pytest.approx(1.76, abs=0.1)


def test_dipole_eq():
    a = lox.DipolePattern(length_m=0.5)
    b = lox.DipolePattern(length_m=0.5)
    c = lox.DipolePattern(length_m=1.0)
    assert a == b
    assert not (a == c)


def test_dipole_pickle():
    d = lox.DipolePattern(length_m=0.5)
    assert pickle.loads(pickle.dumps(d)) == d


def test_dipole_repr_roundtrip():
    d = lox.DipolePattern(length_m=0.5)
    assert eval(repr(d), {"DipolePattern": lox.DipolePattern}) == d


# --- Antennas ---


def test_simple_antenna():
    a = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    assert repr(a) == "SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)"


def test_simple_antenna_eq():
    a = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    b = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    c = lox.SimpleAntenna(gain_db=20.0, beamwidth_deg=3.0)
    assert a == b
    assert not (a == c)


def test_simple_antenna_pickle():
    a = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    assert pickle.loads(pickle.dumps(a)) == a


def test_simple_antenna_repr_roundtrip():
    a = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    assert eval(repr(a), {"SimpleAntenna": lox.SimpleAntenna}) == a


def test_complex_antenna():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    a = lox.ComplexAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    gain = a.gain(29e9, angle_deg=0.0)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_complex_antenna_invalid_pattern():
    with pytest.raises(ValueError, match="expected a ParabolicPattern"):
        lox.ComplexAntenna(pattern="not a pattern", boresight=[0.0, 0.0, 1.0])


def test_complex_antenna_pickle():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    a = lox.ComplexAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    restored = pickle.loads(pickle.dumps(a))
    # ComplexAntenna doesn't have __eq__, verify via gain
    assert float(restored.gain(29e9, 0.0)) == pytest.approx(float(a.gain(29e9, 0.0)), abs=1e-10)


def test_complex_antenna_repr():
    p = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
    a = lox.ComplexAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    r = repr(a)
    assert r.startswith("ComplexAntenna(pattern=ParabolicPattern(")
    assert "boresight=[0.0, 0.0, 1.0]" in r


# --- Transmitter ---


def test_transmitter_eirp():
    # 10 dBi antenna, 5 W power, 1 dB line loss, 0 dB OBO
    # EIRP = 10 + 10*log10(5) - 1 = 15.99 dBW
    a = lox.SimpleAntenna(gain_db=10.0, beamwidth_deg=10.0)
    tx = lox.Transmitter(frequency_hz=29e9, power_w=5.0, line_loss_db=1.0)
    eirp = tx.eirp(a, angle_deg=0.0)
    assert float(eirp) == pytest.approx(15.99, abs=0.01)


def test_transmitter_default_obo():
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=0.0)
    assert repr(tx).endswith("output_back_off_db=0.0)")


def test_transmitter_eq():
    a = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    b = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    c = lox.Transmitter(frequency_hz=29e9, power_w=5.0, line_loss_db=1.0)
    assert a == b
    assert not (a == c)


def test_transmitter_pickle():
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0, output_back_off_db=0.5)
    assert pickle.loads(pickle.dumps(tx)) == tx


def test_transmitter_repr_roundtrip():
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0, output_back_off_db=0.5)
    assert eval(repr(tx), {"Transmitter": lox.Transmitter}) == tx


# --- Receivers ---


def test_complex_receiver_noise_temperature():
    rx = lox.ComplexReceiver(
        frequency_hz=29e9,
        antenna_noise_temperature_k=265.0,
        lna_gain_db=30.0,
        lna_noise_figure_db=1.0,
        noise_figure_db=5.0,
        loss_db=3.0,
    )
    assert rx.noise_temperature() == pytest.approx(627.06, rel=1e-4)


def test_complex_receiver_system_noise_temperature():
    rx = lox.ComplexReceiver(
        frequency_hz=29e9,
        antenna_noise_temperature_k=265.0,
        lna_gain_db=30.0,
        lna_noise_figure_db=1.0,
        noise_figure_db=5.0,
        loss_db=3.0,
    )
    assert rx.system_noise_temperature() == pytest.approx(904.53, rel=1e-4)


def test_simple_receiver_eq():
    a = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    b = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    c = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=600.0)
    assert a == b
    assert not (a == c)


def test_simple_receiver_pickle():
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_simple_receiver_repr_roundtrip():
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    assert eval(repr(rx), {"SimpleReceiver": lox.SimpleReceiver}) == rx


def test_complex_receiver_eq():
    kwargs = dict(
        frequency_hz=29e9,
        antenna_noise_temperature_k=265.0,
        lna_gain_db=30.0,
        lna_noise_figure_db=1.0,
        noise_figure_db=5.0,
        loss_db=3.0,
    )
    a = lox.ComplexReceiver(**kwargs)
    b = lox.ComplexReceiver(**kwargs)
    c = lox.ComplexReceiver(**{**kwargs, "loss_db": 4.0})
    assert a == b
    assert not (a == c)


def test_complex_receiver_pickle():
    rx = lox.ComplexReceiver(
        frequency_hz=29e9,
        antenna_noise_temperature_k=265.0,
        lna_gain_db=30.0,
        lna_noise_figure_db=1.0,
        noise_figure_db=5.0,
        loss_db=3.0,
        demodulator_loss_db=0.5,
        implementation_loss_db=0.3,
    )
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_complex_receiver_repr_roundtrip():
    rx = lox.ComplexReceiver(
        frequency_hz=29e9,
        antenna_noise_temperature_k=265.0,
        lna_gain_db=30.0,
        lna_noise_figure_db=1.0,
        noise_figure_db=5.0,
        loss_db=3.0,
    )
    assert eval(repr(rx), {"ComplexReceiver": lox.ComplexReceiver}) == rx


# --- Channel ---


def test_channel_bandwidth():
    # BPSK, 1 Mbit/s, roll-off=0.5, FEC=0.5 -> BW = 3 MHz
    ch = lox.Channel(
        link_type="downlink",
        data_rate=1e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("BPSK"),
        roll_off=0.5,
        fec=0.5,
    )
    assert ch.bandwidth() == pytest.approx(3e6, rel=1e-10)


def test_channel_eb_n0():
    # C/N0 = 80 dB*Hz, R = 1 Mbit/s -> Eb/N0 = 80 - 60 = 20 dB
    ch = lox.Channel(
        link_type="downlink",
        data_rate=1e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
    )
    eb_n0 = ch.eb_n0(lox.Decibel(80.0))
    assert float(eb_n0) == pytest.approx(20.0, abs=1e-10)


def test_channel_link_margin():
    ch = lox.Channel(
        link_type="downlink",
        data_rate=1e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
    )
    margin = ch.link_margin(lox.Decibel(15.0))
    assert float(margin) == pytest.approx(2.0, abs=1e-10)


def test_channel_invalid_link_type():
    with pytest.raises(ValueError, match="unknown link type"):
        lox.Channel(
            link_type="crosslink",
            data_rate=1e6,
            required_eb_n0_db=10.0,
            margin_db=3.0,
            modulation=lox.Modulation("BPSK"),
        )


def test_channel_pickle():
    ch = lox.Channel(
        link_type="downlink",
        data_rate=1e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )
    restored = pickle.loads(pickle.dumps(ch))
    # Channel doesn't have __eq__, verify via bandwidth
    assert restored.bandwidth() == pytest.approx(ch.bandwidth(), abs=1e-10)


def test_channel_repr():
    ch = lox.Channel(
        link_type="downlink",
        data_rate=1e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
    )
    r = repr(ch)
    assert "link_type='downlink'" in r
    assert "modulation=Modulation('QPSK')" in r


# --- Environmental Losses ---


def test_environmental_losses_default():
    losses = lox.EnvironmentalLosses()
    assert float(losses.total()) == pytest.approx(0.0, abs=1e-15)


def test_environmental_losses_total():
    losses = lox.EnvironmentalLosses(rain_db=2.0, gaseous_db=0.5, atmospheric_db=1.0)
    assert float(losses.total()) == pytest.approx(3.5, abs=1e-10)


def test_environmental_losses_eq():
    a = lox.EnvironmentalLosses(rain_db=2.0)
    b = lox.EnvironmentalLosses(rain_db=2.0)
    c = lox.EnvironmentalLosses(rain_db=3.0)
    assert a == b
    assert not (a == c)


def test_environmental_losses_pickle():
    losses = lox.EnvironmentalLosses(rain_db=2.0, gaseous_db=0.5, atmospheric_db=1.0)
    assert pickle.loads(pickle.dumps(losses)) == losses


def test_environmental_losses_repr_roundtrip():
    losses = lox.EnvironmentalLosses(rain_db=2.0, gaseous_db=0.5, atmospheric_db=1.0)
    assert eval(repr(losses), {"EnvironmentalLosses": lox.EnvironmentalLosses}) == losses


# --- Free functions ---


def test_fspl():
    # 1000 km, 29 GHz -> ~181.7 dB
    loss = lox.fspl(distance_km=1000.0, frequency_hz=29e9)
    assert float(loss) == pytest.approx(181.696, abs=0.1)


def test_freq_overlap_full():
    assert lox.freq_overlap(10e9, 1e6, 10e9, 1e6) == pytest.approx(1.0, abs=1e-10)


def test_freq_overlap_none():
    assert lox.freq_overlap(10e9, 1e6, 12e9, 1e6) == pytest.approx(0.0, abs=1e-10)


def test_freq_overlap_partial():
    assert lox.freq_overlap(10e9, 1e9, 10.5e9, 1e9) == pytest.approx(0.5, abs=1e-10)


# --- Communication System ---


def test_communication_system_c_n0():
    tx_ant = lox.SimpleAntenna(gain_db=46.0, beamwidth_deg=0.7)
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    c_n0 = tx_sys.carrier_to_noise_density(
        rx_system=rx_sys,
        losses_db=0.0,
        range_km=1000.0,
        tx_angle_deg=0.0,
        rx_angle_deg=0.0,
    )
    assert float(c_n0) == pytest.approx(104.9, abs=0.2)


def test_communication_system_noise_power():
    rx_ant = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    p_noise = rx_sys.noise_power(bandwidth_hz=1e6)
    assert float(p_noise) == pytest.approx(-141.61, abs=0.01)


def test_communication_system_pickle():
    tx_ant = lox.SimpleAntenna(gain_db=46.0, beamwidth_deg=0.7)
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)
    restored = pickle.loads(pickle.dumps(tx_sys))
    # CommunicationSystem doesn't have __eq__, verify repr matches
    assert repr(restored) == repr(tx_sys)


def test_communication_system_pickle_with_receiver():
    rx_ant = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)
    restored = pickle.loads(pickle.dumps(rx_sys))
    # Verify via noise_power computation
    assert float(restored.noise_power(1e6)) == pytest.approx(float(rx_sys.noise_power(1e6)), abs=1e-10)


# --- Link Stats ---


def test_link_stats_end_to_end():
    tx_ant = lox.SimpleAntenna(gain_db=46.0, beamwidth_deg=0.7)
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        data_rate=10e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    stats = lox.LinkStats.calculate(
        tx_system=tx_sys,
        rx_system=rx_sys,
        channel=ch,
        range_km=1000.0,
        tx_angle_deg=0.0,
        rx_angle_deg=0.0,
    )

    # EIRP = 46 + 10 - 1 = 55 dBW
    assert float(stats.eirp) == pytest.approx(55.0, abs=0.01)
    # FSPL at 1000 km, 29 GHz
    assert float(stats.fspl) == pytest.approx(181.7, abs=0.1)
    # C/N0
    assert float(stats.c_n0) == pytest.approx(104.9, abs=0.2)
    # Eb/N0 = C/N0 - 10*log10(10e6)
    assert float(stats.eb_n0) == pytest.approx(34.9, abs=0.2)
    # Margin = Eb/N0 - 10 - 3
    assert float(stats.margin) == pytest.approx(21.9, abs=0.2)
    # Properties
    assert stats.slant_range_km == pytest.approx(1000.0, abs=1e-6)
    assert stats.data_rate == pytest.approx(10e6, abs=1e-6)
    assert stats.frequency_hz == pytest.approx(29e9, abs=1.0)


def test_link_stats_with_losses():
    tx_ant = lox.SimpleAntenna(gain_db=46.0, beamwidth_deg=0.7)
    tx = lox.Transmitter(frequency_hz=29e9, power_w=10.0, line_loss_db=1.0)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.SimpleAntenna(gain_db=30.0, beamwidth_deg=3.0)
    rx = lox.SimpleReceiver(frequency_hz=29e9, system_noise_temperature_k=500.0)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        data_rate=10e6,
        required_eb_n0_db=10.0,
        margin_db=3.0,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    losses = lox.EnvironmentalLosses(rain_db=2.0, atmospheric_db=1.0)

    stats_no_loss = lox.LinkStats.calculate(tx_sys, rx_sys, ch, 1000.0, 0.0, 0.0)
    stats_loss = lox.LinkStats.calculate(tx_sys, rx_sys, ch, 1000.0, 0.0, 0.0, losses)

    # 3 dB of environmental losses should reduce margin by 3 dB
    margin_diff = float(stats_no_loss.margin) - float(stats_loss.margin)
    assert margin_diff == pytest.approx(3.0, abs=0.01)
