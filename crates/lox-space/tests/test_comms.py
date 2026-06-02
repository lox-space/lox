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
    assert float(lox.Decibel.from_linear(db.to_linear())) == pytest.approx(
        13.5, abs=1e-10
    )


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


def test_decibel_mul():
    db = 14.0 * lox.dB
    assert float(db) == pytest.approx(14.0, abs=1e-10)


def test_decibel_rmul():
    db = lox.dB * 14.0
    assert float(db) == pytest.approx(14.0, abs=1e-10)


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
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    gain = p.peak_gain(frequency=29e9 * lox.Hz)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_parabolic_beamwidth():
    # D=0.98m, f=29GHz: full HPBW = 2·arcsin(1.6163308·λ/(π·D)) ≈ 0.6219°
    # (the previous value 0.7372° was the first-null angle, not HPBW)
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    bw = p.beamwidth(frequency=29e9 * lox.Hz)
    lam = 299_792_458 / 29e9
    expected = math.degrees(2.0 * math.asin(1.6163308 * lam / (math.pi * 0.98)))
    assert bw.to_degrees() == pytest.approx(expected, rel=1e-4)


def test_parabolic_beamwidth_none_for_sub_wavelength_diameter():
    # At 1 GHz (λ ≈ 0.300 m) the HPBW threshold diameter is ≈ 0.154 m.
    # A 0.1 m dish is below this limit, so beamwidth is undefined.
    p = lox.ParabolicPattern(diameter=0.1 * lox.m, efficiency=0.65)
    assert p.beamwidth(frequency=1e9 * lox.Hz) is None


def test_parabolic_on_axis_equals_peak():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    f = 29e9 * lox.Hz
    gain = p.gain(f, angle=0.0 * lox.deg)
    peak = p.peak_gain(f)
    assert float(gain) == pytest.approx(float(peak), abs=1e-6)


def test_parabolic_gain_at_180():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    gain = p.gain(29e9 * lox.Hz, angle=180.0 * lox.deg)
    assert float(gain) < -50.0


def test_parabolic_from_beamwidth_roundtrip():
    p = lox.ParabolicPattern.from_beamwidth(
        beamwidth=math.degrees(0.1) * lox.deg, frequency=2e9 * lox.Hz, efficiency=0.65
    )
    bw = p.beamwidth(frequency=2e9 * lox.Hz)
    assert bw.to_degrees() == pytest.approx(math.degrees(0.1), rel=0.01)


def test_parabolic_eq():
    a = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    b = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    c = lox.ParabolicPattern(diameter=1.0 * lox.m, efficiency=0.45)
    assert a == b
    assert not (a == c)


def test_parabolic_pickle():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    assert pickle.loads(pickle.dumps(p)) == p


def test_parabolic_repr_roundtrip():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    assert (
        eval(
            repr(p),
            {"ParabolicPattern": lox.ParabolicPattern, "Distance": lox.Distance},
        )
        == p
    )


# --- Gaussian Pattern ---


def test_gaussian_peak_gain():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    gain = p.peak_gain(frequency=29e9 * lox.Hz)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_gaussian_3db_at_half_beamwidth():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    f = 29e9 * lox.Hz
    bw = p.beamwidth(f)
    half_bw = bw * 0.5
    peak = float(p.peak_gain(f))
    gain = float(p.gain(f, angle=half_bw))
    assert peak - gain == pytest.approx(3.0103, abs=0.01)


def test_gaussian_eq():
    a = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    b = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    c = lox.GaussianPattern(diameter=1.0 * lox.m, efficiency=0.45)
    assert a == b
    assert not (a == c)


def test_gaussian_pickle():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    assert pickle.loads(pickle.dumps(p)) == p


def test_gaussian_repr_roundtrip():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    assert (
        eval(
            repr(p), {"GaussianPattern": lox.GaussianPattern, "Distance": lox.Distance}
        )
        == p
    )


# --- Dipole Pattern ---


def test_half_wave_dipole_broadside():
    # Half-wave dipole: peak gain at 90 deg approx 2.15 dBi
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length=(wavelength / 2.0) * lox.m)
    gain = d.gain(29e9 * lox.Hz, angle=90.0 * lox.deg)
    assert float(gain) == pytest.approx(2.15, abs=0.01)


def test_half_wave_dipole_endfire():
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length=(wavelength / 2.0) * lox.m)
    gain = d.gain(29e9 * lox.Hz, angle=0.0 * lox.deg)
    assert float(gain) < -50.0


def test_short_dipole_peak():
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length=(wavelength / 100.0) * lox.m)
    peak = d.peak_gain(29e9 * lox.Hz)
    assert float(peak) == pytest.approx(1.76, abs=0.1)


def test_dipole_eq():
    a = lox.DipolePattern(length=0.5 * lox.m)
    b = lox.DipolePattern(length=0.5 * lox.m)
    c = lox.DipolePattern(length=1.0 * lox.m)
    assert a == b
    assert not (a == c)


def test_dipole_pickle():
    d = lox.DipolePattern(length=0.5 * lox.m)
    assert pickle.loads(pickle.dumps(d)) == d


def test_dipole_repr_roundtrip():
    d = lox.DipolePattern(length=0.5 * lox.m)
    assert (
        eval(repr(d), {"DipolePattern": lox.DipolePattern, "Distance": lox.Distance})
        == d
    )


def test_complex_antenna_dipole_beamwidth_is_none():
    # DipolePattern returns None because the API has no plane-of-measurement
    # parameter; the E-plane HPBW is physically defined but not expressible as
    # a single value here. PatternedAntenna delegates to the pattern, so it also
    # returns None.
    d = lox.DipolePattern(length=0.005 * lox.m)
    a = lox.PatternedAntenna(pattern=d, boresight=[0.0, 0.0, 1.0])
    assert a.beamwidth(frequency=29e9 * lox.Hz) is None


# --- Antennas ---


def test_simple_antenna():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    r = repr(a)
    assert r.startswith("ConstantAntenna(gain=Decibel(30.0), beamwidth=Angle(")


def test_simple_antenna_eq():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    b = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    c = lox.ConstantAntenna(gain=20.0 * lox.dB, beamwidth=3.0 * lox.deg)
    assert a == b
    assert not (a == c)


def test_simple_antenna_pickle():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    assert pickle.loads(pickle.dumps(a)) == a


def test_simple_antenna_repr_roundtrip():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    assert (
        eval(
            repr(a),
            {
                "ConstantAntenna": lox.ConstantAntenna,
                "Decibel": lox.Decibel,
                "Angle": lox.Angle,
            },
        )
        == a
    )


def test_complex_antenna():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    gain = a.gain(29e9 * lox.Hz, angle=0.0 * lox.deg)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_complex_antenna_invalid_pattern():
    with pytest.raises(ValueError, match="expected a ParabolicPattern"):
        lox.PatternedAntenna(pattern="not a pattern", boresight=[0.0, 0.0, 1.0])


def test_complex_antenna_pickle():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    restored = pickle.loads(pickle.dumps(a))
    # PatternedAntenna doesn't have __eq__, verify via gain
    assert float(restored.gain(29e9 * lox.Hz, 0.0 * lox.deg)) == pytest.approx(
        float(a.gain(29e9 * lox.Hz, 0.0 * lox.deg)), abs=1e-10
    )


def test_complex_antenna_repr():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    r = repr(a)
    assert r.startswith("PatternedAntenna(pattern=ParabolicPattern(")
    assert "boresight=[0.0, 0.0, 1.0]" in r


# --- Transmitter ---


def test_transmitter_eirp():
    # 10 dBi antenna, 5 W power, 1 dB line loss, 0 dB OBO
    # EIRP = 10 + 10*log10(5) - 1 = 15.99 dBW
    a = lox.ConstantAntenna(gain=10.0 * lox.dB, beamwidth=10.0 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=5.0 * lox.W, line_loss=1.0 * lox.dB
    )
    eirp = tx.eirp(a, angle=0.0 * lox.deg)
    assert float(eirp) == pytest.approx(15.99, abs=0.01)


def test_transmitter_default_obo():
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=0.0 * lox.dB
    )
    assert repr(tx).endswith("output_back_off=Decibel(0.0))")


def test_transmitter_eq():
    a = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    b = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    c = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=5.0 * lox.W, line_loss=1.0 * lox.dB
    )
    assert a == b
    assert not (a == c)


def test_transmitter_pickle():
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz,
        power=10.0 * lox.W,
        line_loss=1.0 * lox.dB,
        output_back_off=0.5 * lox.dB,
    )
    assert pickle.loads(pickle.dumps(tx)) == tx


def test_transmitter_repr_roundtrip():
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz,
        power=10.0 * lox.W,
        line_loss=1.0 * lox.dB,
        output_back_off=0.5 * lox.dB,
    )
    assert (
        eval(
            repr(tx),
            {
                "AmplifierTransmitter": lox.AmplifierTransmitter,
                "Frequency": lox.Frequency,
                "Power": lox.Power,
                "Decibel": lox.Decibel,
            },
        )
        == tx
    )


# --- Receivers ---


def test_complex_receiver_from_lna_and_noise_figure():
    # Gateway link: T_ant=290K, LNA(G=20dB, T=175K), Rx(NF=2dB)
    # T_sys = 290 + 175 + 169.619/100 = 466.696 K
    rx = lox.CascadeReceiver.from_lna_and_noise_figure(
        frequency=26.5 * lox.GHz,
        antenna_noise_temperature=290.0 * lox.K,
        lna_gain=20.0 * lox.dB,
        lna_noise_temperature=175.0 * lox.K,
        receiver_noise_figure=2.0 * lox.dB,
    )
    assert rx.system_noise_temperature().to_kelvin() == pytest.approx(466.696, abs=0.01)


def test_complex_receiver_from_feed_loss_and_noise_figure():
    # Feed loss=3dB, NF=5dB, T_ant=265K, receiver_gain=20dB
    rx = lox.CascadeReceiver.from_feed_loss_and_noise_figure(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=265.0 * lox.K,
        feed_loss=3.0 * lox.dB,
        receiver_noise_figure=5.0 * lox.dB,
        receiver_gain=20.0 * lox.dB,
    )
    # Friis input-referred T_sys = old output-referred / L
    old_t_sys_output = 904.53
    loss_linear = 10 ** (-3.0 / 10)
    assert rx.system_noise_temperature().to_kelvin() == pytest.approx(
        old_t_sys_output / loss_linear, rel=1e-3
    )


def test_simple_receiver_eq():
    a = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    b = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    c = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=600.0 * lox.K
    )
    assert a == b
    assert not (a == c)


def test_simple_receiver_pickle():
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_simple_receiver_repr_roundtrip():
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    assert (
        eval(
            repr(rx),
            {
                "NoiseTempReceiver": lox.NoiseTempReceiver,
                "Frequency": lox.Frequency,
                "Temperature": lox.Temperature,
            },
        )
        == rx
    )


def test_complex_receiver_system_noise_temperature():
    # Direct stage construction
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
            lox.NoiseStage(gain=10.0 * lox.dB, noise_temperature=500.0 * lox.K),
        ],
    )
    # T_sys = 100 + 50 + 500/100 = 155 K
    assert rx.system_noise_temperature().to_kelvin() == pytest.approx(155.0, abs=0.01)


def test_complex_receiver_eq():
    kwargs = dict(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    a = lox.CascadeReceiver(**kwargs)
    b = lox.CascadeReceiver(**kwargs)
    c = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=200.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    assert a == b
    assert not (a == c)


def test_complex_receiver_pickle():
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
            lox.NoiseStage(gain=10.0 * lox.dB, noise_temperature=500.0 * lox.K),
        ],
        demodulator_loss=0.5 * lox.dB,
        implementation_loss=0.3 * lox.dB,
    )
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_complex_receiver_repr_roundtrip():
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    assert (
        eval(
            repr(rx),
            {
                "CascadeReceiver": lox.CascadeReceiver,
                "NoiseStage": lox.NoiseStage,
                "Frequency": lox.Frequency,
                "Temperature": lox.Temperature,
                "Decibel": lox.Decibel,
            },
        )
        == rx
    )


# --- Channel ---


def test_channel_bandwidth():
    # BPSK, 1 Msps, roll-off=0.5 -> BW = 1.5 MHz
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=1 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
        roll_off=0.5,
        fec=0.5,
    )
    assert ch.bandwidth().to_hertz() == pytest.approx(1.5e6, rel=1e-10)


def test_channel_eb_n0():
    # QPSK, 500 ksps, fec=0.5 -> data_rate=1 Mbps (=symbol_rate * bps * fec for Eb/N0)
    # Es/N0 = 80 - 10*log10(500e3) = 80 - 56.99 = 23.01
    # Eb/N0 = 23.01 - 10*log10(2*0.5) = 23.01
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=500 * lox.kHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    eb_n0 = ch.eb_n0(80.0 * lox.dB)
    expected = 80.0 - 10.0 * math.log10(500e3) - 10.0 * math.log10(2 * 0.5)
    assert float(eb_n0) == pytest.approx(expected, abs=1e-6)


def test_channel_link_margin():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=500 * lox.kHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    margin = ch.link_margin(15.0 * lox.dB)
    assert float(margin) == pytest.approx(2.0, abs=1e-10)


def test_channel_invalid_link_type():
    with pytest.raises(ValueError, match="unknown link direction"):
        lox.Channel(
            link_type="invalid",
            symbol_rate=1 * lox.MHz,
            required_eb_n0=10.0 * lox.dB,
            margin=3.0 * lox.dB,
            modulation=lox.Modulation("BPSK"),
        )


def test_channel_pickle():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=1 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )
    restored = pickle.loads(pickle.dumps(ch))
    # Channel doesn't have __eq__, verify via bandwidth
    assert restored.bandwidth().to_hertz() == pytest.approx(
        ch.bandwidth().to_hertz(), abs=1e-10
    )


def test_channel_repr():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=1 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    r = repr(ch)
    assert "link_type='downlink'" in r
    assert "modulation=Modulation('QPSK')" in r


# --- Environmental Losses ---


def test_environmental_losses_none():
    losses = lox.EnvironmentalLosses.none()
    assert float(losses.total()) == pytest.approx(0.0, abs=1e-15)


def test_environmental_losses_from_values():
    losses = lox.EnvironmentalLosses.from_values(
        rain=2.0 * lox.dB, gaseous=0.5 * lox.dB, atmospheric=1.0 * lox.dB
    )
    assert float(losses.total()) == pytest.approx(3.5, abs=1e-10)


def test_environmental_losses_eq():
    a = lox.EnvironmentalLosses.from_values(rain=2.0 * lox.dB)
    b = lox.EnvironmentalLosses.from_values(rain=2.0 * lox.dB)
    c = lox.EnvironmentalLosses.from_values(rain=3.0 * lox.dB)
    assert a == b
    assert not (a == c)


def test_environmental_losses_repr():
    losses = lox.EnvironmentalLosses.from_values(
        rain=2.0 * lox.dB, gaseous=0.5 * lox.dB, atmospheric=1.0 * lox.dB
    )
    r = repr(losses)
    assert "rain" in r
    assert "gaseous" in r


# --- Free functions ---


def test_fspl():
    # 1000 km, 29 GHz -> ~181.7 dB
    loss = lox.fspl(distance=1000.0 * lox.km, frequency=29e9 * lox.Hz)
    assert float(loss) == pytest.approx(181.696, abs=0.1)


def test_freq_overlap_full():
    assert lox.freq_overlap(
        10e9 * lox.Hz, 1e6 * lox.Hz, 10e9 * lox.Hz, 1e6 * lox.Hz
    ) == pytest.approx(1.0, abs=1e-10)


def test_freq_overlap_none():
    assert lox.freq_overlap(
        10e9 * lox.Hz, 1e6 * lox.Hz, 12e9 * lox.Hz, 1e6 * lox.Hz
    ) == pytest.approx(0.0, abs=1e-10)


def test_freq_overlap_partial():
    assert lox.freq_overlap(
        10e9 * lox.Hz, 1e9 * lox.Hz, 10.5e9 * lox.Hz, 1e9 * lox.Hz
    ) == pytest.approx(0.5, abs=1e-10)


# --- Communication System ---


def test_communication_system_c_n0():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    c_n0 = tx_sys.carrier_to_noise_density(
        rx_system=rx_sys,
        losses=0.0 * lox.dB,
        range=1000.0 * lox.km,
        tx_angle=0.0 * lox.deg,
        rx_angle=0.0 * lox.deg,
    )
    assert float(c_n0) == pytest.approx(104.9, abs=0.2)


def test_communication_system_noise_power():
    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    p_noise = rx_sys.noise_power(bandwidth=1e6 * lox.Hz)
    assert float(p_noise) == pytest.approx(-141.61, abs=0.01)


def test_communication_system_pickle():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)
    restored = pickle.loads(pickle.dumps(tx_sys))
    # CommunicationSystem doesn't have __eq__, verify repr matches
    assert repr(restored) == repr(tx_sys)


def test_communication_system_pickle_with_receiver():
    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)
    restored = pickle.loads(pickle.dumps(rx_sys))
    # Verify via noise_power computation
    assert float(restored.noise_power(1e6 * lox.Hz)) == pytest.approx(
        float(rx_sys.noise_power(1e6 * lox.Hz)), abs=1e-10
    )


# --- Link Stats ---


def test_link_stats_end_to_end():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    stats = lox.LinkStats.calculate(
        tx_system=tx_sys,
        rx_system=rx_sys,
        range=1000.0 * lox.km,
        bandwidth=ch.bandwidth(),
        tx_angle=0.0 * lox.deg,
        rx_angle=0.0 * lox.deg,
    )
    modulated = ch.apply(stats)

    # EIRP = 46 + 10 - 1 = 55 dBW
    assert float(stats.eirp) == pytest.approx(55.0, abs=0.01)
    # FSPL at 1000 km, 29 GHz
    assert float(stats.fspl) == pytest.approx(181.7, abs=0.1)
    # C/N0
    assert float(stats.c_n0) == pytest.approx(104.9, abs=0.2)
    # Es/N0 = C/N0 - 10*log10(5e6)
    assert float(modulated.es_n0) == pytest.approx(37.91, abs=0.2)
    # Eb/N0 = Es/N0 - 10*log10(2*0.5) = Es/N0
    assert float(modulated.eb_n0) == pytest.approx(37.91, abs=0.2)
    # Margin = Eb/N0 - 10 - 3
    assert float(modulated.margin) == pytest.approx(24.91, abs=0.2)
    # Properties
    assert stats.slant_range.to_kilometers() == pytest.approx(1000.0, abs=1e-6)
    assert modulated.symbol_rate.to_hertz() == pytest.approx(5e6, abs=1e-6)
    assert stats.frequency.to_hertz() == pytest.approx(29e9, abs=1.0)


def test_link_stats_with_losses():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    losses = lox.EnvironmentalLosses.from_values(rain=2.0 * lox.dB, atmospheric=1.0 * lox.dB)

    stats_no_loss = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000 * lox.km, ch.bandwidth(), 0 * lox.deg, 0 * lox.deg
    )
    stats_loss = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000 * lox.km, ch.bandwidth(), 0 * lox.deg, 0 * lox.deg, losses
    )
    modulated_no_loss = ch.apply(stats_no_loss)
    modulated_loss = ch.apply(stats_loss)

    # 3 dB of environmental losses should reduce margin by 3 dB
    margin_diff = float(modulated_no_loss.margin) - float(modulated_loss.margin)
    assert margin_diff == pytest.approx(3.0, abs=0.01)


# --- Channel additional methods ---


def test_channel_data_rate():
    # QPSK (2 bps), 5 Msps -> data_rate = 10 Mbps
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    assert ch.data_rate().to_hertz() == pytest.approx(10e6, rel=1e-10)


def test_channel_information_rate():
    # data_rate=10 Mbps, fec=0.5 -> info_rate=5 Mbps
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        fec=0.5,
    )
    assert ch.information_rate().to_hertz() == pytest.approx(5e6, rel=1e-10)


def test_channel_es_n0():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    es_n0 = ch.es_n0(80.0 * lox.dB)
    expected = 80.0 - 10.0 * math.log10(5e6)
    assert float(es_n0) == pytest.approx(expected, abs=1e-3)


def test_channel_c_n():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
    )
    c_n = ch.c_n(80.0 * lox.dB)
    bw = 5e6 * 1.35
    expected = 80.0 - 10.0 * math.log10(bw)
    assert float(c_n) == pytest.approx(expected, abs=1e-3)


def test_channel_spreading_factor_narrowband():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=1 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
    )
    assert ch.spreading_factor() is None
    assert ch.processing_gain() is None


def test_channel_spreading_factor_dsss():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=10 * lox.kHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
        chip_rate=4 * lox.MHz,
    )
    assert ch.spreading_factor() == pytest.approx(400.0, rel=1e-10)
    pg = ch.processing_gain()
    assert float(pg) == pytest.approx(10.0 * math.log10(400.0), abs=1e-3)


def test_channel_uplink_and_crosslink():
    for lt in ("uplink", "crosslink"):
        ch = lox.Channel(
            link_type=lt,
            symbol_rate=1 * lox.MHz,
            required_eb_n0=10.0 * lox.dB,
            margin=3.0 * lox.dB,
            modulation=lox.Modulation("BPSK"),
        )
        assert lt in repr(ch)


def test_channel_repr_with_chip_rate():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=10 * lox.kHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
        chip_rate=4 * lox.MHz,
    )
    assert "chip_rate=" in repr(ch)


# --- CommunicationSystem additional methods ---


def test_communication_system_carrier_power():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    p_rx = tx_sys.carrier_power(
        rx_system=rx_sys,
        losses=0.0 * lox.dB,
        range=1000.0 * lox.km,
        tx_angle=0.0 * lox.deg,
        rx_angle=0.0 * lox.deg,
    )
    # carrier_power should be finite
    assert math.isfinite(float(p_rx))


def test_communication_system_with_complex_antenna():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    ant = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    sys = lox.CommunicationSystem(antenna=ant, transmitter=tx)
    r = repr(sys)
    assert "PatternedAntenna" in r
    assert "ParabolicPattern" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(sys))
    assert repr(restored) == repr(sys)


def test_communication_system_with_complex_receiver():
    ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    sys = lox.CommunicationSystem(antenna=ant, receiver=rx)
    r = repr(sys)
    assert "CascadeReceiver" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(sys))
    assert repr(restored) == repr(sys)


def test_communication_system_invalid_antenna():
    with pytest.raises(ValueError, match="expected a ConstantAntenna or PatternedAntenna"):
        lox.CommunicationSystem(antenna="not an antenna")


def test_communication_system_invalid_receiver():
    ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    with pytest.raises(ValueError, match="expected NoiseTempReceiver, CascadeReceiver, or GtReceiver"):
        lox.CommunicationSystem(antenna=ant, receiver="not a receiver")


# --- PatternedAntenna additional patterns ---


def test_complex_antenna_gaussian_repr():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[1.0, 0.0, 0.0])
    r = repr(a)
    assert "GaussianPattern" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(a))
    assert float(restored.gain(29e9 * lox.Hz, 0.0 * lox.deg)) == pytest.approx(
        float(a.gain(29e9 * lox.Hz, 0.0 * lox.deg)), abs=1e-10
    )


def test_complex_antenna_dipole_repr():
    d = lox.DipolePattern(length=0.005 * lox.m)
    a = lox.PatternedAntenna(pattern=d, boresight=[0.0, 1.0, 0.0])
    r = repr(a)
    assert "DipolePattern" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(a))
    assert float(restored.peak_gain(29e9 * lox.Hz)) == pytest.approx(
        float(a.peak_gain(29e9 * lox.Hz)), abs=1e-10
    )


def test_complex_antenna_beamwidth():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    bw = a.beamwidth(frequency=29e9 * lox.Hz)
    assert bw is not None
    assert bw.to_degrees() > 0


def test_complex_antenna_peak_gain():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    pg = a.peak_gain(frequency=29e9 * lox.Hz)
    assert float(pg) == pytest.approx(46.01119, rel=1e-4)


# --- CascadeReceiver additional methods ---


def test_complex_receiver_chain_gain():
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
            lox.NoiseStage(gain=10.0 * lox.dB, noise_temperature=500.0 * lox.K),
        ],
    )
    cg = rx.chain_gain()
    assert float(cg) == pytest.approx(30.0, abs=1e-10)


# --- NoiseStage ---


def test_noise_stage_repr():
    ns = lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K)
    r = repr(ns)
    assert "NoiseStage" in r
    assert "gain=" in r
    assert "noise_temperature=" in r


def test_noise_stage_pickle():
    ns = lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K)
    restored = pickle.loads(pickle.dumps(ns))
    assert repr(restored) == repr(ns)


# --- LinkStats additional getters ---


def test_link_stats_all_getters():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    stats = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000 * lox.km, ch.bandwidth(), 0 * lox.deg, 0 * lox.deg
    )

    # Test getters not covered in test_link_stats_end_to_end
    assert math.isfinite(float(stats.c_n))
    assert stats.carrier_rx_power is not None
    assert math.isfinite(float(stats.carrier_rx_power))
    assert stats.noise_power is not None
    assert math.isfinite(float(stats.noise_power))
    assert stats.bandwidth.to_hertz() == pytest.approx(5e6 * 1.35, rel=1e-6)
    assert math.isfinite(float(stats.gt))


def test_link_stats_repr():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(
        frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K
    )
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )

    stats = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000 * lox.km, ch.bandwidth(), 0 * lox.deg, 0 * lox.deg
    )
    modulated = ch.apply(stats)
    r = repr(stats)
    assert "LinkStats" in r
    assert "c_n0=" in r
    assert "margin=" in repr(modulated)


# --- Free functions ---


def test_slant_range():
    sr = lox.slant_range(
        elevation=10.0 * lox.deg,
        earth_radius=6371.0 * lox.km,
        altitude=600.0 * lox.km,
    )
    assert sr.to_kilometers() > 600.0


def test_power_flux_density():
    pfd = lox.power_flux_density(
        eirp=0.0 * lox.dB,
        distance=1000.0 * lox.km,
        occupied_bw=1 * lox.MHz,
        reference_bw=1 * lox.MHz,
    )
    expected = 10.0 * math.log10(1.0 / (4.0 * math.pi * (1e6) ** 2))
    assert float(pfd) == pytest.approx(expected, abs=0.01)


def test_pfd_mask():
    mask = lox.pfd_mask(
        elevation=0.0 * lox.deg,
        start_val=-154.0 * lox.dB,
        end_val=-144.0 * lox.dB,
    )
    assert float(mask) == pytest.approx(-154.0, abs=1e-10)

    mask = lox.pfd_mask(
        elevation=15.0 * lox.deg,
        start_val=-154.0 * lox.dB,
        end_val=-144.0 * lox.dB,
    )
    assert float(mask) == pytest.approx(-149.0, abs=1e-10)

    mask = lox.pfd_mask(
        elevation=90.0 * lox.deg,
        start_val=-154.0 * lox.dB,
        end_val=-144.0 * lox.dB,
    )
    assert float(mask) == pytest.approx(-144.0, abs=1e-10)


# --- AmplifierTransmitter with PatternedAntenna ---


def test_transmitter_eirp_complex_antenna():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p, boresight=[0.0, 0.0, 1.0])
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    eirp = tx.eirp(a, angle=0.0 * lox.deg)
    assert math.isfinite(float(eirp))


def test_transmitter_eirp_invalid_antenna():
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    with pytest.raises(ValueError, match="expected a ConstantAntenna or PatternedAntenna"):
        tx.eirp("not an antenna", angle=0.0 * lox.deg)


# --- CommunicationSystem repr with no receiver/transmitter ---


def test_communication_system_repr_minimal():
    ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    sys = lox.CommunicationSystem(antenna=ant)
    r = repr(sys)
    assert "CommunicationSystem" in r
    assert "ConstantAntenna" in r
    assert "receiver=" not in r
    assert "transmitter=" not in r


def test_communication_system_repr_with_transmitter():
    ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    tx = lox.AmplifierTransmitter(
        frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB
    )
    sys = lox.CommunicationSystem(antenna=ant, transmitter=tx)
    r = repr(sys)
    assert "transmitter=AmplifierTransmitter" in r


# --- Lumped-tier smoke test ---


def test_eirp_gt_lumped_link():
    tx = lox.CommunicationSystem.eirp_only(
        lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    )
    rx = lox.CommunicationSystem.gt_only(
        lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    )
    link = lox.LinkStats.calculate(
        tx,
        rx,
        1000.0 * lox.km,
        5.0 * lox.MHz,
        0.0 * lox.rad,
        0.0 * lox.rad,
    )
    assert link.carrier_rx_power is None
    assert link.noise_power is None
    assert abs(float(link.c_n0) - 104.913) < 0.2


def test_lumped_transmitter_receiver_pickle():
    tx = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    rx = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)

    assert pickle.loads(pickle.dumps(tx)) == tx
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_lumped_link_interference_requires_absolute_power():
    tx = lox.CommunicationSystem.eirp_only(
        lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    )
    rx = lox.CommunicationSystem.gt_only(
        lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    )
    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    link = lox.LinkStats.calculate(
        tx,
        rx,
        1000.0 * lox.km,
        channel.bandwidth(),
        0.0 * lox.rad,
        0.0 * lox.rad,
    )
    modulated = channel.apply(link)

    with pytest.raises(ValueError, match="absolute carrier and noise powers"):
        modulated.with_interference(1e-12)


# --- Error mapping tests ---


def test_missing_transmitter_raises_value_error():
    antenna = lox.ConstantAntenna(0.0 * lox.dB, 1.0 * lox.deg)
    rx_antenna = lox.ConstantAntenna(0.0 * lox.dB, 1.0 * lox.deg)
    rx_receiver = lox.NoiseTempReceiver(29.0 * lox.GHz, 500.0 * lox.K)
    tx = lox.CommunicationSystem(antenna=antenna)
    rx = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx_receiver)
    with pytest.raises(ValueError, match="transmitter"):
        tx.carrier_to_noise_density(
            rx, 0.0 * lox.dB, 1000.0 * lox.km,
            0.0 * lox.rad, 0.0 * lox.rad,
        )


def test_missing_receiver_raises_value_error():
    tx_antenna = lox.ConstantAntenna(0.0 * lox.dB, 1.0 * lox.deg)
    tx_transmitter = lox.AmplifierTransmitter(
        frequency=29.0 * lox.GHz, power=10.0 * lox.W, line_loss=0.0 * lox.dB,
    )
    tx = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx_transmitter)
    rx_antenna = lox.ConstantAntenna(0.0 * lox.dB, 1.0 * lox.deg)
    rx = lox.CommunicationSystem(antenna=rx_antenna)
    with pytest.raises(ValueError, match="receiver"):
        tx.carrier_to_noise_density(
            rx, 0.0 * lox.dB, 1000.0 * lox.km,
            0.0 * lox.rad, 0.0 * lox.rad,
        )


def test_frequency_mismatch_raises_value_error():
    tx = lox.CommunicationSystem.eirp_only(
        lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    )
    rx = lox.CommunicationSystem.gt_only(
        lox.GtReceiver(30.0 * lox.GHz, 3.01 * lox.dB)
    )
    with pytest.raises(ValueError, match="frequency"):
        tx.carrier_to_noise_density(
            rx, 0.0 * lox.dB, 1000.0 * lox.km,
            0.0 * lox.rad, 0.0 * lox.rad,
        )


def test_unexpected_antenna_raises_value_error():
    antenna = lox.ConstantAntenna(46.0 * lox.dB, 0.7 * lox.deg)
    tx_transmitter = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    with pytest.raises(ValueError, match="EirpTransmitter must not be paired"):
        lox.CommunicationSystem(antenna=antenna, transmitter=tx_transmitter)


# --- Pickle round-trip tests for lumped CommunicationSystem ---


def test_lumped_communication_system_pickle():
    tx_system = lox.CommunicationSystem.eirp_only(
        lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    )
    rx_system = lox.CommunicationSystem.gt_only(
        lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    )
    assert pickle.loads(pickle.dumps(tx_system)) == tx_system
    assert pickle.loads(pickle.dumps(rx_system)) == rx_system


def test_lumped_communication_system_via_constructor_pickle():
    tx_system = lox.CommunicationSystem(
        transmitter=lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    )
    rx_system = lox.CommunicationSystem(
        receiver=lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    )
    assert pickle.loads(pickle.dumps(tx_system)) == tx_system
    assert pickle.loads(pickle.dumps(rx_system)) == rx_system


# --- ModulatedLinkStats.with_interference happy path ---


def test_modulated_with_interference_component_tier():
    tx_antenna = lox.ConstantAntenna(46.0 * lox.dB, 0.7 * lox.deg)
    tx_transmitter = lox.AmplifierTransmitter(
        frequency=29.0 * lox.GHz, power=10.0 * lox.W, line_loss=1.0 * lox.dB,
    )
    tx = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx_transmitter)

    rx_antenna = lox.ConstantAntenna(30.0 * lox.dB, 3.0 * lox.deg)
    rx_receiver = lox.NoiseTempReceiver(29.0 * lox.GHz, 500.0 * lox.K)
    rx = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx_receiver)

    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )

    link = lox.LinkStats.calculate(
        tx, rx, 1000.0 * lox.km, channel.bandwidth(),
        0.0 * lox.rad, 0.0 * lox.rad,
    )
    modulated = channel.apply(link)
    interference = modulated.with_interference(1e-12)

    assert float(interference.margin_with_interference) < float(modulated.margin)
    assert float(interference.eb_n0i0) < float(modulated.eb_n0)
    assert interference.interference_power_w == 1e-12


# --- CommunicationSystem __new__ validation error paths ---


def test_build_transmitter_rejects_non_transmitter():
    antenna = lox.ConstantAntenna(0.0 * lox.dB, 1.0 * lox.deg)
    with pytest.raises(ValueError, match="EirpTransmitter or AmplifierTransmitter"):
        lox.CommunicationSystem(antenna=antenna, transmitter="not a transmitter")


def test_amplifier_without_antenna_rejected():
    tx = lox.AmplifierTransmitter(29.0 * lox.GHz, 10.0 * lox.W, 1.0 * lox.dB)
    with pytest.raises(ValueError, match="AmplifierTransmitter requires an antenna"):
        lox.CommunicationSystem(transmitter=tx)


def test_gt_receiver_with_antenna_rejected():
    antenna = lox.ConstantAntenna(30.0 * lox.dB, 3.0 * lox.deg)
    rx = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    with pytest.raises(ValueError, match="GtReceiver must not be paired"):
        lox.CommunicationSystem(antenna=antenna, receiver=rx)


def test_component_receiver_without_antenna_rejected():
    rx = lox.NoiseTempReceiver(29.0 * lox.GHz, 500.0 * lox.K)
    with pytest.raises(ValueError, match="component-tier receiver requires"):
        lox.CommunicationSystem(receiver=rx)


def test_cascade_receiver_without_antenna_rejected():
    rx = lox.CascadeReceiver(
        frequency=29 * lox.GHz,
        antenna_noise_temperature=100.0 * lox.K,
        stages=[lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K)],
    )
    with pytest.raises(ValueError, match="component-tier receiver requires"):
        lox.CommunicationSystem(receiver=rx)


# --- EirpTransmitter getters and repr ---


def test_eirp_transmitter_getters():
    tx = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    assert tx.frequency.to_hertz() == pytest.approx(29e9, abs=1.0)
    assert float(tx.eirp) == pytest.approx(55.0, abs=1e-10)


def test_eirp_transmitter_repr():
    tx = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    r = repr(tx)
    assert "EirpTransmitter" in r
    assert "55.0" in r


def test_eirp_transmitter_eq():
    a = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    b = lox.EirpTransmitter(29.0 * lox.GHz, 55.0 * lox.dB)
    c = lox.EirpTransmitter(29.0 * lox.GHz, 50.0 * lox.dB)
    assert a == b
    assert not (a == c)


# --- GtReceiver getters and repr ---


def test_gt_receiver_getters():
    rx = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    assert rx.frequency.to_hertz() == pytest.approx(29e9, abs=1.0)
    assert float(rx.gt) == pytest.approx(3.01, abs=1e-10)


def test_gt_receiver_repr():
    rx = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    r = repr(rx)
    assert "GtReceiver" in r
    assert "3.01" in r


def test_gt_receiver_eq():
    a = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    b = lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB)
    c = lox.GtReceiver(29.0 * lox.GHz, 5.0 * lox.dB)
    assert a == b
    assert not (a == c)


# --- NoiseTempReceiver repr ---


def test_simple_receiver_repr():
    rx = lox.NoiseTempReceiver(frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K)
    r = repr(rx)
    assert "NoiseTempReceiver" in r
    assert "500" in r


# --- CommunicationSystem __eq__ ---


def test_communication_system_eq():
    ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    tx = lox.AmplifierTransmitter(frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB)
    a = lox.CommunicationSystem(antenna=ant, transmitter=tx)
    b = lox.CommunicationSystem(antenna=ant, transmitter=tx)
    c = lox.CommunicationSystem(antenna=ant)
    assert a == b
    assert not (a == c)


# --- Channel DSSS pickle ---


def test_channel_dsss_pickle():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=10 * lox.kHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("BPSK"),
        chip_rate=4 * lox.MHz,
    )
    restored = pickle.loads(pickle.dumps(ch))
    assert restored.spreading_factor() == pytest.approx(ch.spreading_factor(), rel=1e-10)
    assert float(restored.processing_gain()) == pytest.approx(float(ch.processing_gain()), abs=1e-10)


# --- LinkStats tx_angle and rx_angle getters ---


def test_link_stats_angle_getters():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    stats = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000.0 * lox.km, 5.0 * lox.MHz, 2.0 * lox.deg, 1.0 * lox.deg
    )
    assert stats.tx_angle.to_degrees() == pytest.approx(2.0, abs=1e-10)
    assert stats.rx_angle.to_degrees() == pytest.approx(1.0, abs=1e-10)


# --- ModulatedLinkStats link, channel, interference getters ---


def test_modulated_link_stats_link_and_channel_getters():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    stats = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000.0 * lox.km, ch.bandwidth(), 0.0 * lox.deg, 0.0 * lox.deg
    )
    modulated = ch.apply(stats)

    # link getter returns the underlying PyLinkStats
    assert modulated.link.c_n0 == stats.c_n0
    # channel getter returns the PyChannel
    assert "QPSK" in repr(modulated.channel)
    # interference getter is None when no interference was applied
    assert modulated.interference is None


# --- InterferenceStats c_n0i0 and repr ---


def test_interference_stats_c_n0i0_and_repr():
    tx_antenna = lox.ConstantAntenna(46.0 * lox.dB, 0.7 * lox.deg)
    tx_transmitter = lox.AmplifierTransmitter(
        frequency=29.0 * lox.GHz, power=10.0 * lox.W, line_loss=1.0 * lox.dB,
    )
    tx = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx_transmitter)

    rx_antenna = lox.ConstantAntenna(30.0 * lox.dB, 3.0 * lox.deg)
    rx_receiver = lox.NoiseTempReceiver(29.0 * lox.GHz, 500.0 * lox.K)
    rx = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx_receiver)

    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    link = lox.LinkStats.calculate(
        tx, rx, 1000.0 * lox.km, channel.bandwidth(), 0.0 * lox.rad, 0.0 * lox.rad,
    )
    modulated = channel.apply(link)
    interference = modulated.with_interference(1e-12)

    assert math.isfinite(float(interference.c_n0i0))
    r = repr(interference)
    assert "InterferenceStats" in r
    assert "c_n0i0" in r


# --- ModulatedLinkStats repr ---


def test_modulated_link_stats_repr():
    tx_ant = lox.ConstantAntenna(gain=46.0 * lox.dB, beamwidth=0.7 * lox.deg)
    tx = lox.AmplifierTransmitter(frequency=29e9 * lox.Hz, power=10.0 * lox.W, line_loss=1.0 * lox.dB)
    tx_sys = lox.CommunicationSystem(antenna=tx_ant, transmitter=tx)

    rx_ant = lox.ConstantAntenna(gain=30.0 * lox.dB, beamwidth=3.0 * lox.deg)
    rx = lox.NoiseTempReceiver(frequency=29e9 * lox.Hz, system_noise_temperature=500.0 * lox.K)
    rx_sys = lox.CommunicationSystem(antenna=rx_ant, receiver=rx)

    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    stats = lox.LinkStats.calculate(
        tx_sys, rx_sys, 1000.0 * lox.km, ch.bandwidth(), 0.0 * lox.deg, 0.0 * lox.deg
    )
    modulated = ch.apply(stats)
    r = repr(modulated)
    assert "ModulatedLinkStats" in r
    assert "eb_n0=" in r
    assert "margin=" in r


# --- CommunicationSystem repr with GtReceiver (lumped rx side) ---


def test_communication_system_repr_gt_receiver():
    rx_sys = lox.CommunicationSystem.gt_only(lox.GtReceiver(29.0 * lox.GHz, 3.01 * lox.dB))
    r = repr(rx_sys)
    assert "GtReceiver" in r
    assert "3.01" in r


# --- Decibel arithmetic: __mul__ with scalar on left side ---


def test_decibel_mul_left():
    db = lox.Decibel(5.0)
    result = db * 3.0
    assert float(result) == pytest.approx(15.0, abs=1e-10)


# --- Modulation: all variants pickle to correct name ---


def test_modulation_repr_all_variants():
    expected = {
        "BPSK": "Modulation('BPSK')",
        "QPSK": "Modulation('QPSK')",
        "8PSK": "Modulation('8PSK')",
        "16QAM": "Modulation('16QAM')",
        "32QAM": "Modulation('32QAM')",
        "64QAM": "Modulation('64QAM')",
        "128QAM": "Modulation('128QAM')",
        "256QAM": "Modulation('256QAM')",
    }
    for name, expected_repr in expected.items():
        assert repr(lox.Modulation(name)) == expected_repr
