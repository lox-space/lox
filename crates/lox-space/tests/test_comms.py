# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math
import pickle

import pytest
import lox_space as lox


KA_BAND = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)


def make_tx(gain=46.0, power=10.0, feed_loss=1.0, antenna=None):
    """Standard transmit chain: constant-gain antenna + Ka-band amplifier."""
    if antenna is None:
        antenna = lox.ConstantAntenna(gain=gain * lox.dB)
    return lox.TxChain(
        antenna,
        lox.AmplifierTransmitter(power=power * lox.W),
        band=KA_BAND,
        feed_loss=feed_loss * lox.dB,
    )


def make_rx(gain=30.0, noise_temperature=500.0, feed_loss=0.0):
    """Standard receive chain: constant-gain antenna + 500 K receiver."""
    return lox.RxChain(
        lox.ConstantAntenna(gain=gain * lox.dB),
        lox.NoiseTempReceiver(noise_temperature=noise_temperature * lox.K),
        band=KA_BAND,
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=feed_loss * lox.dB,
    )


def link_budget(tx, rx, **kwargs):
    """Computes the standard 29 GHz link budget between two terminals."""
    kwargs.setdefault("carrier", 29.0 * lox.GHz)
    kwargs.setdefault("range", 1000.0 * lox.km)
    return lox.LinkBudget(tx, rx, **kwargs)


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
    gain = p.gain(f, theta=0.0 * lox.deg)
    peak = p.peak_gain(f)
    assert float(gain) == pytest.approx(float(peak), abs=1e-6)


def test_parabolic_gain_at_180():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    gain = p.gain(29e9 * lox.Hz, theta=180.0 * lox.deg)
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
    gain = float(p.gain(f, theta=half_bw))
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
    gain = d.gain(29e9 * lox.Hz, theta=90.0 * lox.deg)
    assert float(gain) == pytest.approx(2.15, abs=0.01)


def test_half_wave_dipole_endfire():
    c = 299792458.0
    wavelength = c / 29e9
    d = lox.DipolePattern(length=(wavelength / 2.0) * lox.m)
    gain = d.gain(29e9 * lox.Hz, theta=0.0 * lox.deg)
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


# --- Antennas ---


def test_simple_antenna():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB)
    r = repr(a)
    assert r == "ConstantAntenna(gain=Decibel(30.0))"


def test_simple_antenna_eq():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB)
    b = lox.ConstantAntenna(gain=30.0 * lox.dB)
    c = lox.ConstantAntenna(gain=20.0 * lox.dB)
    assert a == b
    assert not (a == c)


def test_simple_antenna_pickle():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB)
    assert pickle.loads(pickle.dumps(a)) == a


def test_simple_antenna_repr_roundtrip():
    a = lox.ConstantAntenna(gain=30.0 * lox.dB)
    assert (
        eval(
            repr(a),
            {
                "ConstantAntenna": lox.ConstantAntenna,
                "Decibel": lox.Decibel,
            },
        )
        == a
    )


def test_complex_antenna():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    gain = a.gain(29e9 * lox.Hz, theta=0.0 * lox.deg)
    assert float(gain) == pytest.approx(46.01119, rel=1e-4)


def test_complex_antenna_invalid_pattern():
    with pytest.raises(ValueError, match="expected a ParabolicPattern"):
        lox.PatternedAntenna(pattern="not a pattern")


def test_complex_antenna_pickle():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    restored = pickle.loads(pickle.dumps(a))
    # PatternedAntenna doesn't have __eq__, verify via gain
    assert float(restored.gain(29e9 * lox.Hz, 0.0 * lox.deg)) == pytest.approx(
        float(a.gain(29e9 * lox.Hz, 0.0 * lox.deg)), abs=1e-10
    )


def test_complex_antenna_repr():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    r = repr(a)
    assert r.startswith("PatternedAntenna(pattern=ParabolicPattern(")
    assert "frame=AntennaFrame(boresight=[0.0, 0.0, 1.0]" in r


def test_antenna_frame_identity_axes_and_pickle():
    frame = lox.AntennaFrame.identity()
    assert frame.x() == [1.0, 0.0, 0.0]
    assert frame.y() == [0.0, 1.0, 0.0]
    assert frame.z() == [0.0, 0.0, 1.0]
    assert pickle.loads(pickle.dumps(frame)) == frame


def test_antenna_frame_angles_for():
    frame = lox.AntennaFrame.from_boresight_and_reference(
        [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]
    )
    theta, phi = frame.angles_for([1.0, 0.0, 0.0])
    assert float(theta) == pytest.approx(0.0, abs=1e-12)
    assert float(phi) == pytest.approx(0.0, abs=1e-12)

    theta, phi = frame.angles_for([0.0, 0.0, 1.0])
    assert float(theta) == pytest.approx(math.pi / 2.0, abs=1e-12)
    assert float(phi) == pytest.approx(0.0, abs=1e-12)


def test_antenna_frame_rejects_invalid_inputs():
    with pytest.raises(ValueError, match="invalid boresight"):
        lox.AntennaFrame.from_boresight_and_reference(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]
        )
    with pytest.raises(ValueError, match="invalid reference"):
        lox.AntennaFrame.from_boresight_and_reference(
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]
        )
    with pytest.raises(ValueError, match="invalid direction"):
        lox.AntennaFrame.identity().angles_for([0.0, 0.0, 0.0])


def test_complex_antenna_gain_toward_uses_frame():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    frame = lox.AntennaFrame.from_boresight_and_reference(
        [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]
    )
    a = lox.PatternedAntenna(pattern=p, frame=frame)
    on_axis = a.gain_toward(29e9 * lox.Hz, [1.0, 0.0, 0.0])
    off_axis = a.gain_toward(29e9 * lox.Hz, [0.0, 1.0, 0.0])
    assert float(on_axis) == pytest.approx(float(a.peak_gain(29e9 * lox.Hz)), abs=1e-10)
    assert float(off_axis) < float(on_axis)


def test_complex_antenna_gain_toward_rejects_invalid_direction():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    with pytest.raises(ValueError, match="invalid direction"):
        a.gain_toward(29e9 * lox.Hz, [0.0, 0.0, 0.0])


# --- Transmitter ---


def test_transmitter_eirp():
    # 10 dBi antenna, 5 W power, 1 dB feed loss, 0 dB OBO
    # EIRP = 10 + 10*log10(5) - 1 = 15.99 dBW
    tx = make_tx(gain=10.0, power=5.0, feed_loss=1.0)
    stats = link_budget(tx, make_rx())
    assert float(stats.eirp) == pytest.approx(15.99, abs=0.01)


def test_transmitter_getters():
    tx = lox.AmplifierTransmitter(power=10.0 * lox.W, output_back_off=0.5 * lox.dB)
    assert tx.power.to_watts() == pytest.approx(10.0, abs=1e-12)
    assert float(tx.output_back_off) == pytest.approx(0.5, abs=1e-12)


def test_transmitter_default_obo():
    tx = lox.AmplifierTransmitter(power=10.0 * lox.W)
    assert repr(tx).endswith("output_back_off=Decibel(0.0))")


def test_transmitter_eq():
    a = lox.AmplifierTransmitter(power=10.0 * lox.W)
    b = lox.AmplifierTransmitter(power=10.0 * lox.W)
    c = lox.AmplifierTransmitter(power=5.0 * lox.W)
    assert a == b
    assert not (a == c)


def test_transmitter_pickle():
    tx = lox.AmplifierTransmitter(
        power=10.0 * lox.W,
        output_back_off=0.5 * lox.dB,
    )
    assert pickle.loads(pickle.dumps(tx)) == tx


def test_transmitter_repr_roundtrip():
    tx = lox.AmplifierTransmitter(
        power=10.0 * lox.W,
        output_back_off=0.5 * lox.dB,
    )
    assert (
        eval(
            repr(tx),
            {
                "AmplifierTransmitter": lox.AmplifierTransmitter,
                "FrequencyRange": lox.FrequencyRange,
                "Frequency": lox.Frequency,
                "Power": lox.Power,
                "Decibel": lox.Decibel,
            },
        )
        == tx
    )


# --- Receivers ---


def test_complex_receiver_from_lna_and_noise_figure():
    # LNA(G=20dB, T=175K), Rx(NF=2dB)
    # T_chain = 175 + 169.619/100 = 176.696 K
    rx = lox.CascadeReceiver.from_lna_and_noise_figure(
        lna_gain=20.0 * lox.dB,
        lna_noise_temperature=175.0 * lox.K,
        receiver_noise_figure=2.0 * lox.dB,
    )
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(176.696, abs=0.01)


def test_simple_receiver_eq():
    a = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    b = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    c = lox.NoiseTempReceiver(noise_temperature=600.0 * lox.K)
    assert a == b
    assert not (a == c)


def test_simple_receiver_getters():
    rx = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    assert rx.noise_temperature.to_kelvin() == pytest.approx(500.0, abs=1e-12)


def test_simple_receiver_pickle():
    rx = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_simple_receiver_repr_roundtrip():
    rx = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    assert (
        eval(
            repr(rx),
            {
                "NoiseTempReceiver": lox.NoiseTempReceiver,
                "FrequencyRange": lox.FrequencyRange,
                "Frequency": lox.Frequency,
                "Temperature": lox.Temperature,
            },
        )
        == rx
    )


def test_complex_receiver_chain_noise_temperature():
    # Direct stage construction
    rx = lox.CascadeReceiver(
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
            lox.NoiseStage(gain=10.0 * lox.dB, noise_temperature=500.0 * lox.K),
        ],
    )
    # T_chain = 50 + 500/100 = 55 K
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(55.0, abs=0.01)


def test_complex_receiver_eq():
    kwargs = dict(
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    a = lox.CascadeReceiver(**kwargs)
    b = lox.CascadeReceiver(**kwargs)
    c = lox.CascadeReceiver(
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=100.0 * lox.K),
        ],
    )
    assert a == b
    assert not (a == c)


def test_complex_receiver_pickle():
    rx = lox.CascadeReceiver(
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
                "FrequencyRange": lox.FrequencyRange,
                "Frequency": lox.Frequency,
                "Temperature": lox.Temperature,
                "Decibel": lox.Decibel,
            },
        )
        == rx
    )


# --- Channel ---


def test_channel_bandwidth():
    # 1 Msps, roll-off=0.5 -> BW = 1.5 MHz
    ch = lox.Channel(symbol_rate=1 * lox.MHz, roll_off=0.5)
    assert ch.bandwidth().to_hertz() == pytest.approx(1.5e6, rel=1e-10)


def test_channel_es_n0():
    # Es/N0 = 80 - 10*log10(500e3) = 23.01
    ch = lox.Channel(symbol_rate=500 * lox.kHz)
    es_n0 = ch.es_n0(80.0 * lox.dB)
    expected = 80.0 - 10.0 * math.log10(500e3)
    assert float(es_n0) == pytest.approx(expected, abs=1e-6)


def test_channel_rejects_invalid():
    with pytest.raises(ValueError):
        lox.Channel(symbol_rate=0 * lox.Hz)
    with pytest.raises(ValueError):
        lox.Channel(symbol_rate=1 * lox.MHz, roll_off=-0.1)
    with pytest.raises(ValueError):
        lox.Channel(symbol_rate=5 * lox.MHz, chip_rate=1 * lox.MHz)


def test_channel_dsss():
    ch = lox.Channel(symbol_rate=10 * lox.kHz, chip_rate=4 * lox.MHz)
    assert ch.spreading_factor() == pytest.approx(400.0, rel=1e-10)
    assert float(ch.processing_gain()) == pytest.approx(26.02, abs=0.01)


def test_channel_pickle():
    ch = lox.Channel(symbol_rate=1 * lox.MHz, roll_off=0.25)
    restored = pickle.loads(pickle.dumps(ch))
    assert restored == ch


def test_channel_repr():
    ch = lox.Channel(symbol_rate=1 * lox.MHz)
    r = repr(ch)
    assert "symbol_rate=" in r
    assert "roll_off=0.35" in r


# --- ModCod ---


def test_modcod_from_required_eb_n0():
    mc = lox.ModCod(
        "my downlink", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB
    )
    assert mc.name == "my downlink"
    assert mc.info_bits_per_symbol == pytest.approx(1.0, abs=1e-12)
    assert float(mc.required_eb_n0) == pytest.approx(10.0, abs=1e-12)
    assert float(mc.required_es_n0) == pytest.approx(10.0, abs=1e-9)
    assert mc.metric == "BER"
    assert mc.error_rate == pytest.approx(1e-6)


def test_modcod_rejects_invalid():
    with pytest.raises(ValueError):
        lox.ModCod("bad", lox.Modulation("QPSK"), 1.5, 10.0 * lox.dB)
    with pytest.raises(ValueError, match="unknown error metric"):
        lox.ModCod("bad", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB, metric="XYZ")


def test_modcod_dvb_s2_table():
    table = lox.ModCod.dvb_s2()
    assert len(table) == 28
    qpsk12 = next(mc for mc in table if mc.name == "QPSK 1/2")
    # ETSI EN 302 307-1 Table 13: efficiency 0.988858, Es/N0 = 1.0 dB.
    assert qpsk12.info_bits_per_symbol == pytest.approx(0.988858, abs=5e-7)
    assert float(qpsk12.required_es_n0) == pytest.approx(1.0, abs=1e-9)
    assert qpsk12.reference.startswith("ETSI")
    # The coding chain is exposed.
    names = [name for name, _ in qpsk12.codes]
    assert any("LDPC" in n for n in names)


def test_modcod_select():
    table = lox.ModCod.dvb_s2()
    best = lox.ModCod.select(20.0 * lox.dB, 0.0 * lox.dB, table)
    assert best.name == "32APSK 9/10"
    assert lox.ModCod.select(-3.0 * lox.dB, 0.0 * lox.dB, table) is None


def test_modcod_evaluate():
    losses = lox.PropagationLosses(rain=2.0 * lox.dB)
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    stats = link_budget(make_tx(), make_rx(), losses=losses)
    m = stats.modulate(ch, mc, design_margin=3.0 * lox.dB)
    # Es/N0 = C/N0 - 10*log10(5e6); Eb/N0 = Es/N0 (1 info bit/symbol)
    assert float(m.es_n0) == pytest.approx(float(stats.c_n0) - 10 * math.log10(5e6), abs=1e-9)
    assert float(m.eb_n0) == pytest.approx(float(m.es_n0), abs=1e-9)
    assert float(m.margin) == pytest.approx(float(m.eb_n0) - 10.0 - 3.0, abs=1e-9)
    assert m.information_rate().to_hertz() == pytest.approx(5e6, rel=1e-10)
    assert m.modcod == mc
    assert m.channel == ch
    assert float(m.design_margin) == pytest.approx(3.0)


# --- Propagation Losses ---


def test_propagation_losses_none():
    losses = lox.PropagationLosses.none()
    assert float(losses.total()) == pytest.approx(0.0, abs=1e-15)
    assert losses.lines == []


def test_propagation_losses_total_and_absorptive():
    losses = lox.PropagationLosses(
        rain=2.0 * lox.dB, gaseous=0.5 * lox.dB, scintillation=0.3 * lox.dB
    )
    assert float(losses.total()) == pytest.approx(2.8, abs=1e-10)
    # Scintillation is non-absorptive.
    assert float(losses.absorptive()) == pytest.approx(2.5, abs=1e-10)


def test_propagation_losses_other_lines():
    losses = lox.PropagationLosses(other=[("Radome wetting", 0.5 * lox.dB, True)])
    assert float(losses.total()) == pytest.approx(0.5, abs=1e-10)
    assert float(losses.absorptive()) == pytest.approx(0.5, abs=1e-10)
    label, value, absorptive = losses.lines[0]
    assert label == "Radome wetting"
    assert float(value) == pytest.approx(0.5, abs=1e-10)
    assert absorptive


def test_propagation_losses_rejects_negative():
    with pytest.raises(ValueError):
        lox.PropagationLosses(rain=-1.0 * lox.dB)


def test_propagation_losses_eq():
    a = lox.PropagationLosses(rain=2.0 * lox.dB)
    b = lox.PropagationLosses(rain=2.0 * lox.dB)
    c = lox.PropagationLosses(rain=3.0 * lox.dB)
    assert a == b
    assert not (a == c)


def test_propagation_losses_repr():
    losses = lox.PropagationLosses(rain=2.0 * lox.dB, gaseous=0.5 * lox.dB)
    r = repr(losses)
    assert "Rain attenuation" in r
    assert "Gaseous absorption" in r


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


# --- Link Stats ---


def test_link_budget_noise_power():
    budget = link_budget(make_tx(), make_rx())
    assert float(budget.noise_power(1.0 * lox.MHz)) == pytest.approx(-141.61, abs=0.01)


def test_link_stats_end_to_end():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    mc = lox.ModCod("QPSK 1/2 test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)

    stats = link_budget(
        make_tx(),
        make_rx(),
        tx_angle=0.0 * lox.deg,
        rx_angle=0.0 * lox.deg,
    )
    modulated = stats.modulate(ch, mc, design_margin=3.0 * lox.dB)

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
    assert modulated.information_rate().to_hertz() == pytest.approx(5e6, abs=1e-6)
    assert stats.frequency.to_hertz() == pytest.approx(29e9, abs=1.0)


def test_link_stats_downlink_degrades_gt():
    losses = lox.PropagationLosses(
        rain=2.0 * lox.dB, gaseous=0.5 * lox.dB, cloud=0.2 * lox.dB,
        scintillation=0.3 * lox.dB,
    )
    clear = link_budget(make_tx(), make_rx(), losses=losses)
    faded = link_budget(make_tx(), make_rx(), losses=losses, link_type="downlink")

    assert clear.gt_degraded is None
    assert clear.link_type is None
    assert faded.link_type == "downlink"
    assert float(faded.gt) == pytest.approx(float(clear.gt), abs=1e-12)
    assert float(faded.gt_degraded) < float(faded.gt)
    # The budget uses the degraded G/T.
    assert float(clear.c_n0) - float(faded.c_n0) == pytest.approx(
        float(faded.gt) - float(faded.gt_degraded), abs=1e-10
    )


def test_link_stats_uplink_stays_clear_sky():
    losses = lox.PropagationLosses(rain=2.0 * lox.dB)
    clear = link_budget(make_tx(), make_rx(), losses=losses)
    uplink = link_budget(make_tx(), make_rx(), losses=losses, link_type="uplink")
    assert uplink.gt_degraded is None
    assert float(uplink.c_n0) == pytest.approx(float(clear.c_n0), abs=1e-12)


def test_link_stats_rejects_unknown_link_type():
    with pytest.raises(ValueError):
        link_budget(make_tx(), make_rx(), link_type="sideways")


def test_link_stats_with_losses():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    mc = lox.ModCod("QPSK 1/2 test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)

    losses = lox.PropagationLosses(rain=2.0 * lox.dB, other=[("Atmospheric", 1.0 * lox.dB, True)])

    stats_no_loss = link_budget(make_tx(), make_rx())
    stats_loss = link_budget(make_tx(), make_rx(), losses=losses)
    modulated_no_loss = stats_no_loss.modulate(ch, mc, design_margin=3.0 * lox.dB)
    modulated_loss = stats_loss.modulate(ch, mc, design_margin=3.0 * lox.dB)

    # 3 dB of environmental losses should reduce margin by 3 dB
    margin_diff = float(modulated_no_loss.margin) - float(modulated_loss.margin)
    assert margin_diff == pytest.approx(3.0, abs=0.01)


# --- Channel additional methods ---


def test_channel_c_n():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    c_n = ch.c_n(80.0 * lox.dB)
    bw = 5e6 * 1.35
    expected = 80.0 - 10.0 * math.log10(bw)
    assert float(c_n) == pytest.approx(expected, abs=1e-3)


def test_channel_spreading_factor_narrowband():
    ch = lox.Channel(symbol_rate=1 * lox.MHz)
    assert ch.spreading_factor() is None
    assert ch.processing_gain() is None


def test_modcod_getters_and_repr():
    mc = lox.ModCod("my link", lox.Modulation("8PSK"), 0.75, 6.0 * lox.dB)
    assert mc.modulation == lox.Modulation("8PSK")
    assert mc.code_rate == pytest.approx(0.75)
    assert mc.codes == [("FEC", 0.75)]
    r = repr(mc)
    assert "my link" in r
    assert "required_eb_n0=6.00 dB" in r


@pytest.mark.parametrize("metric", ["WER", "FER", "PER", "wer"])
def test_modcod_metric_variants(metric):
    mc = lox.ModCod(
        "m", lox.Modulation("QPSK"), 0.5, 4.0 * lox.dB, metric=metric, error_rate=1e-7
    )
    assert mc.metric == metric.upper()
    assert mc.error_rate == pytest.approx(1e-7)


def test_channel_chip_rate_getter_and_repr():
    ch = lox.Channel(symbol_rate=10 * lox.kHz, chip_rate=4 * lox.MHz)
    assert ch.chip_rate.to_hertz() == pytest.approx(4e6, rel=1e-12)
    assert "chip_rate=" in repr(ch)
    assert lox.Channel(symbol_rate=10 * lox.kHz).chip_rate is None


def test_propagation_losses_repr_non_absorptive():
    losses = lox.PropagationLosses(other=[("Pointing margin", 0.4 * lox.dB, False)])
    r = repr(losses)
    assert "Pointing margin" in r
    assert "False" in r


def test_modcod_information_rate():
    # QPSK rate 1/2 at 5 Msps: 5 Mbit/s information rate.
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    assert mc.information_rate(5 * lox.MHz).to_hertz() == pytest.approx(5e6, rel=1e-10)


# --- LinkBudget additional outputs ---


def test_link_stats_carrier_power():
    stats = link_budget(make_tx(), make_rx())
    # carrier_rx_power should be finite for component-tier links
    assert math.isfinite(float(stats.carrier_rx_power))


def test_tx_chain_invalid_antenna():
    tx = lox.AmplifierTransmitter(power=10.0 * lox.W)
    with pytest.raises(ValueError, match="expected a ConstantAntenna or PatternedAntenna"):
        lox.TxChain("not an antenna", tx, KA_BAND)


def test_rx_chain_invalid_receiver():
    with pytest.raises(ValueError, match="expected NoiseTempReceiver or CascadeReceiver"):
        lox.RxChain(
            lox.ConstantAntenna(gain=30.0 * lox.dB),
            "not a receiver",
            band=KA_BAND,
            antenna_noise_temperature=0.0 * lox.K,
        )


# --- PatternedAntenna additional patterns ---


def test_complex_antenna_gaussian_repr():
    p = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    frame = lox.AntennaFrame.from_boresight_and_reference(
        [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]
    )
    a = lox.PatternedAntenna(pattern=p, frame=frame)
    r = repr(a)
    assert "GaussianPattern" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(a))
    assert float(restored.gain(29e9 * lox.Hz, 0.0 * lox.deg)) == pytest.approx(
        float(a.gain(29e9 * lox.Hz, 0.0 * lox.deg)), abs=1e-10
    )


def test_complex_antenna_dipole_repr():
    d = lox.DipolePattern(length=0.005 * lox.m)
    frame = lox.AntennaFrame.from_boresight_and_reference(
        [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]
    )
    a = lox.PatternedAntenna(pattern=d, frame=frame)
    r = repr(a)
    assert "DipolePattern" in r
    # Pickle roundtrip
    restored = pickle.loads(pickle.dumps(a))
    assert float(restored.peak_gain(29e9 * lox.Hz)) == pytest.approx(
        float(a.peak_gain(29e9 * lox.Hz)), abs=1e-10
    )


def test_complex_antenna_peak_gain():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    pg = a.peak_gain(frequency=29e9 * lox.Hz)
    assert float(pg) == pytest.approx(46.01119, rel=1e-4)


# --- CascadeReceiver additional methods ---


def test_complex_receiver_chain_gain():
    rx = lox.CascadeReceiver(
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


# --- LinkBudget additional getters ---


def test_link_stats_all_getters():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)

    stats = link_budget(make_tx(), make_rx())

    # Derived views take the bandwidth explicitly.
    assert math.isfinite(float(stats.c_n(ch.bandwidth())))
    assert stats.carrier_rx_power is not None
    assert math.isfinite(float(stats.carrier_rx_power))
    assert stats.noise_power(ch.bandwidth()) is not None
    assert math.isfinite(float(stats.noise_power(ch.bandwidth())))
    assert math.isfinite(float(stats.gt))
    # C/N0 consistency through the views.
    bw = ch.bandwidth()
    c_n0 = float(stats.carrier_rx_power) - float(stats.noise_power(bw)) + 10 * math.log10(float(bw))
    assert float(stats.c_n0) == pytest.approx(c_n0, abs=1e-9)


def test_link_stats_repr():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)

    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    stats = link_budget(make_tx(), make_rx())
    modulated = stats.modulate(ch, mc, design_margin=3.0 * lox.dB)
    r = repr(stats)
    assert "LinkBudget" in r
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


def test_pfd_mask_art_21_16():
    mask = lox.PfdMask.art_21_16(-154.0 * lox.dB)
    assert float(mask.value_at(0.0 * lox.deg)) == pytest.approx(-154.0, abs=1e-10)
    assert float(mask.value_at(15.0 * lox.deg)) == pytest.approx(-149.0, abs=1e-10)
    assert float(mask.value_at(90.0 * lox.deg)) == pytest.approx(-144.0, abs=1e-10)


def test_pfd_mask_custom_nodes():
    mask = lox.PfdMask(
        [
            (5.0 * lox.deg, -150.0 * lox.dB),
            (15.0 * lox.deg, -145.0 * lox.dB),
            (25.0 * lox.deg, -142.5 * lox.dB),
        ]
    )
    assert float(mask.value_at(0.0 * lox.deg)) == pytest.approx(-150.0, abs=1e-10)
    assert float(mask.value_at(10.0 * lox.deg)) == pytest.approx(-147.5, abs=1e-10)
    assert float(mask.value_at(20.0 * lox.deg)) == pytest.approx(-143.75, abs=1e-10)
    assert float(mask.value_at(60.0 * lox.deg)) == pytest.approx(-142.5, abs=1e-10)
    assert len(mask.nodes()) == 3


def test_pfd_mask_rejects_invalid_nodes():
    with pytest.raises(ValueError):
        lox.PfdMask([(5.0 * lox.deg, -150.0 * lox.dB)])
    with pytest.raises(ValueError):
        lox.PfdMask(
            [
                (25.0 * lox.deg, -144.0 * lox.dB),
                (5.0 * lox.deg, -154.0 * lox.dB),
            ]
        )


def test_pfd_mask_eq_and_pickle():
    mask = lox.PfdMask.art_21_16(-154.0 * lox.dB)
    assert mask == lox.PfdMask.art_21_16(-154.0 * lox.dB)
    assert mask != lox.PfdMask.art_21_16(-150.0 * lox.dB)
    assert pickle.loads(pickle.dumps(mask)) == mask


# --- AmplifierTransmitter with PatternedAntenna ---


def test_transmitter_eirp_complex_antenna():
    p = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    a = lox.PatternedAntenna(pattern=p)
    stats = link_budget(make_tx(antenna=a), make_rx())
    assert math.isfinite(float(stats.eirp))


def test_tx_chain_rejects_negative_feed_loss():
    tx = lox.AmplifierTransmitter(power=10.0 * lox.W)
    with pytest.raises(ValueError, match="feed loss"):
        lox.TxChain(
            lox.ConstantAntenna(gain=46.0 * lox.dB),
            tx,
            band=KA_BAND,
            feed_loss=-1.0 * lox.dB,
        )


# --- Lumped-tier smoke test ---


def test_lumped_link_interference_requires_absolute_power():
    tx = lox.EirpModel(KA_BAND, 55.0 * lox.dB)
    rx = lox.GtModel(KA_BAND, 3.01 * lox.dB)
    channel = lox.Channel(symbol_rate=5.0 * lox.MHz)
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    link = link_budget(tx, rx)
    modulated = link.modulate(channel, mc, design_margin=3.0 * lox.dB)

    with pytest.raises(ValueError, match="absolute carrier and noise powers"):
        modulated.with_interference(1e-12 * lox.W)


# --- ModulatedLinkBudget.with_interference happy path ---


def test_modulated_with_interference_component_tier():
    channel = lox.Channel(symbol_rate=5.0 * lox.MHz)

    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    link = link_budget(make_tx(), make_rx())
    modulated = link.modulate(channel, mc, design_margin=3.0 * lox.dB)
    interference = modulated.with_interference(1e-12 * lox.W)

    assert float(interference.margin_with_interference) < float(modulated.margin)
    assert float(interference.eb_n0i0) < float(modulated.eb_n0)
    assert float(interference.interference_power) == 1e-12


# --- NoiseTempReceiver repr ---


def test_simple_receiver_repr():
    rx = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    r = repr(rx)
    assert "NoiseTempReceiver" in r
    assert "500" in r


# --- Channel DSSS pickle ---


def test_channel_dsss_pickle():
    ch = lox.Channel(symbol_rate=10 * lox.kHz, chip_rate=4 * lox.MHz)
    restored = pickle.loads(pickle.dumps(ch))
    assert restored.spreading_factor() == pytest.approx(ch.spreading_factor(), rel=1e-10)
    assert float(restored.processing_gain()) == pytest.approx(float(ch.processing_gain()), abs=1e-10)


# --- LinkBudget pattern-angle getters and pointing arguments ---


def test_link_stats_off_boresight_angle_reduces_eirp():
    pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    tx_ant = lox.PatternedAntenna(pattern=pattern)
    boresight = link_budget(make_tx(antenna=tx_ant), make_rx())
    off = link_budget(make_tx(antenna=tx_ant), make_rx(), tx_angle=2.0 * lox.deg)
    assert float(off.eirp) < float(boresight.eirp)


def test_link_stats_defaults_to_boresight():
    stats = link_budget(make_tx(), make_rx())
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.1)


def test_link_stats_direction_pointing():
    # Dish boresight along +X; a line of sight along +Z is 90° off boresight.
    frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
    pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    tx_ant = lox.PatternedAntenna(pattern=pattern, frame=frame)

    on_axis = link_budget(
        make_tx(antenna=tx_ant),
        make_rx(),
        tx_direction=[1.0, 0.0, 0.0],
    )
    off_axis = link_budget(
        make_tx(antenna=tx_ant),
        make_rx(),
        tx_direction=[0.0, 0.0, 1.0],
    )

    assert float(off_axis.eirp) < float(on_axis.eirp)


def test_link_stats_rejects_angle_and_direction():
    with pytest.raises(ValueError, match="tx_angle or tx_direction"):
        link_budget(
            make_tx(),
            make_rx(),
            tx_angle=1.0 * lox.deg,
            tx_direction=[1.0, 0.0, 0.0],
        )


# --- ModulatedLinkBudget budget and channel getters ---


def test_modulated_link_stats_link_and_channel_getters():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    stats = link_budget(make_tx(), make_rx())
    modulated = stats.modulate(ch, mc)

    # budget getter returns the underlying LinkBudget
    assert modulated.budget.c_n0 == stats.c_n0
    assert math.isfinite(float(modulated.c_n))
    assert modulated.closes()
    # channel and modcod getters round-trip
    assert modulated.channel == ch
    assert modulated.modcod == mc
    assert float(modulated.design_margin) == pytest.approx(0.0)


# --- InterferenceStats c_n0i0 and repr ---


def test_interference_stats_c_n0i0_and_repr():
    channel = lox.Channel(symbol_rate=5.0 * lox.MHz)
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    link = link_budget(make_tx(), make_rx())
    modulated = link.modulate(channel, mc, design_margin=3.0 * lox.dB)
    interference = modulated.with_interference(1e-12 * lox.W)

    assert math.isfinite(float(interference.c_n0i0))
    r = repr(interference)
    assert "InterferenceStats" in r
    assert "c_n0i0" in r


# --- ModulatedLinkBudget repr ---


def test_modulated_link_stats_repr():
    ch = lox.Channel(symbol_rate=5 * lox.MHz)
    mc = lox.ModCod("test", lox.Modulation("QPSK"), 0.5, 10.0 * lox.dB)
    stats = link_budget(make_tx(), make_rx())
    modulated = stats.modulate(ch, mc)
    r = repr(modulated)
    assert "ModulatedLinkBudget" in r
    assert "eb_n0=" in r
    assert "margin=" in r


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
        "OQPSK": "Modulation('OQPSK')",
        "8APSK": "Modulation('8APSK')",
        "16APSK": "Modulation('16APSK')",
        "32APSK": "Modulation('32APSK')",
        "64APSK": "Modulation('64APSK')",
        "GMSK": "Modulation('GMSK')",
        "2FSK": "Modulation('2FSK')",
        "4FSK": "Modulation('4FSK')",
    }
    for name, expected_repr in expected.items():
        assert repr(lox.Modulation(name)) == expected_repr


# --- Link terminals ---


def test_frequency_range():
    assert KA_BAND.contains(29.0 * lox.GHz)
    assert not KA_BAND.contains(45.0 * lox.GHz)
    assert KA_BAND.min().to_gigahertz() == pytest.approx(27.0)
    assert str(KA_BAND) == "27.000\u201331.000 GHz"
    optical = lox.FrequencyRange.from_wavelengths(1530e-9 * lox.m, 1565e-9 * lox.m)
    assert optical.contains(193.4 * lox.THz)
    with pytest.raises(ValueError):
        lox.FrequencyRange(31.0 * lox.GHz, 27.0 * lox.GHz)
    assert pickle.loads(pickle.dumps(KA_BAND)) == KA_BAND


def test_for_link_reference_values():
    # 46 dBi + 10 W \u2212 1 dB feed loss \u2192 EIRP 55 dBW;
    # 30 dBi / 500 K \u2192 G/T 3.0103 dB/K; C/N0 104.913 dB\u00b7Hz at 1000 km.
    stats = link_budget(make_tx(), make_rx())

    assert float(stats.eirp) == pytest.approx(55.0, abs=0.01)
    assert float(stats.gt) == pytest.approx(3.0103, abs=0.01)
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.01)
    assert math.isfinite(float(stats.carrier_rx_power))


def test_lumped_link():
    stats = lox.LinkBudget(
        lox.EirpModel(KA_BAND, 55.0 * lox.dB),
        lox.GtModel(KA_BAND, 3.01 * lox.dB),
        carrier=29.0 * lox.GHz,
        range=1000.0 * lox.km,
    )
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.1)
    assert stats.carrier_rx_power is None
    assert stats.noise_power(5.0 * lox.MHz) is None


def test_for_link_carrier_out_of_band():
    rx_band = lox.FrequencyRange(17.0 * lox.GHz, 21.0 * lox.GHz)
    with pytest.raises(ValueError, match="outside the supported range"):
        lox.LinkBudget(
            lox.EirpModel(KA_BAND, 55.0 * lox.dB),
            lox.GtModel(rx_band, 3.01 * lox.dB),
            carrier=29.0 * lox.GHz,
            range=1000.0 * lox.km,
        )


def test_tx_chain_accessors_and_eirp():
    tx = make_tx()
    assert tx.band == KA_BAND
    assert float(tx.feed_loss) == pytest.approx(1.0, abs=1e-12)
    assert tx.antenna == lox.ConstantAntenna(gain=46.0 * lox.dB)
    assert tx.transmitter == lox.AmplifierTransmitter(power=10.0 * lox.W)
    # EIRP = 46 + 10\u00b7log10(10) \u2212 1 = 55 dBW
    assert float(tx.eirp_at(29.0 * lox.GHz)) == pytest.approx(55.0, abs=1e-9)
    with pytest.raises(ValueError, match="angle or direction"):
        tx.eirp_at(29.0 * lox.GHz, angle=1.0 * lox.deg, direction=[1.0, 0.0, 0.0])


def test_rx_chain_accessors_gt_and_noise():
    rx = lox.RxChain(
        lox.ConstantAntenna(gain=46.0 * lox.dB),
        lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K),
        band=KA_BAND,
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=0.5 * lox.dB,
    )
    assert rx.band == KA_BAND
    assert float(rx.feed_loss) == pytest.approx(0.5, abs=1e-12)
    assert rx.antenna_noise_temperature.to_kelvin() == pytest.approx(0.0, abs=1e-12)
    # T_sys = 290\u00b7(L \u2212 1) + 500\u00b7L with L = 10^0.05
    expected_t_sys = 290.0 * (10 ** (0.5 / 10.0) - 1.0) + 500.0 * 10 ** (0.5 / 10.0)
    assert rx.system_noise_temperature().to_kelvin() == pytest.approx(
        expected_t_sys, rel=1e-12
    )
    gt = rx.gt_at(29.0 * lox.GHz)
    assert float(gt) == pytest.approx(46.0 - 10.0 * math.log10(expected_t_sys), abs=1e-9)


def test_shared_antenna_transceiver():
    # A diplexer-style transceiver is a TX and an RX chain sharing one dish.
    dish = lox.ConstantAntenna(gain=46.0 * lox.dB)
    tx = lox.TxChain(
        dish,
        lox.AmplifierTransmitter(power=10.0 * lox.W),
        band=KA_BAND,
        feed_loss=1.0 * lox.dB,
    )
    rx = lox.RxChain(
        dish,
        lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K),
        band=KA_BAND,
        antenna_noise_temperature=150.0 * lox.K,
        feed_loss=0.5 * lox.dB,
    )
    assert tx.antenna == rx.antenna
    link = lox.LinkBudget(
        tx,
        lox.GtModel(KA_BAND, 30.0 * lox.dB),
        carrier=29.0 * lox.GHz,
        range=1000.0 * lox.km,
        rx_angle=0.0 * lox.deg,
    )
    assert float(link.eirp) == pytest.approx(46.0 + 10.0 - 1.0, abs=1e-9)
    assert math.isfinite(float(rx.gt_at(29.0 * lox.GHz)))


def test_eirp_model_accessors():
    model = lox.EirpModel(KA_BAND, 55.0 * lox.dB)
    assert model.band == KA_BAND
    assert float(model.eirp) == pytest.approx(55.0, abs=1e-12)
    # The pointing is ignored for lumped figures.
    assert float(model.eirp_at(29.0 * lox.GHz, angle=10.0 * lox.deg)) == pytest.approx(
        55.0, abs=1e-12
    )
    assert "EirpModel" in repr(model)


def test_band_accepts_letter_band_names():
    # An IEEE letter-band name is shorthand for the full band range.
    gt = lox.GtModel("Ka", 3.0 * lox.dB)
    assert gt.band.contains(29.0 * lox.GHz)
    assert not gt.band.contains(12.0 * lox.GHz)
    assert gt.band == lox.GtModel("ka", 3.0 * lox.dB).band  # case-insensitive
    eirp = lox.EirpModel("X", 55.0 * lox.dB)
    assert eirp.band.contains(8.2 * lox.GHz)
    tx = lox.TxChain(
        lox.ConstantAntenna(gain=46.0 * lox.dB),
        lox.AmplifierTransmitter(power=10.0 * lox.W),
        band="Ka",
    )
    assert tx.band.contains(29.0 * lox.GHz)
    rx = lox.RxChain(
        lox.ConstantAntenna(gain=30.0 * lox.dB),
        lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K),
        band="Ka",
        antenna_noise_temperature=0.0 * lox.K,
    )
    assert rx.band.contains(29.0 * lox.GHz)
    with pytest.raises(ValueError, match="unknown frequency band"):
        lox.GtModel("Q", 3.0 * lox.dB)
    with pytest.raises(ValueError, match="FrequencyRange or a band name"):
        lox.GtModel(42, 3.0 * lox.dB)


def test_gt_model_accessors():
    model = lox.GtModel(KA_BAND, 3.01 * lox.dB)
    assert model.band == KA_BAND
    assert float(model.gt) == pytest.approx(3.01, abs=1e-12)
    assert float(model.gt_at(29.0 * lox.GHz, angle=10.0 * lox.deg)) == pytest.approx(
        3.01, abs=1e-12
    )
    assert "GtModel" in repr(model)


def test_ground_station_terminals():
    rx = lox.GtModel(KA_BAND, 30.0 * lox.dB)
    location = lox.GroundLocation(
        lox.Origin("Earth"), 13.4 * lox.deg, 52.5 * lox.deg, 0.1 * lox.km
    )
    mask = lox.ElevationMask.fixed(5.0 * lox.deg)
    gs = lox.GroundStation("berlin", location, mask, rx_terminals={"ka": rx})
    assert gs.rx_terminals()["ka"] == rx
    assert gs.tx_terminals() == {}

    bare = lox.GroundStation("bare", location, mask)
    assert bare.rx_terminals() == {}


def test_asset_terminals_round_trip_both_tiers():
    # Component chains and lumped models survive the asset dict round trip.
    location = lox.GroundLocation(
        lox.Origin("Earth"), 13.4 * lox.deg, 52.5 * lox.deg, 0.1 * lox.km
    )
    mask = lox.ElevationMask.fixed(5.0 * lox.deg)
    tx_terminals = {"hga": make_tx(), "beacon": lox.EirpModel(KA_BAND, 40.0 * lox.dB)}
    rx_terminals = {"main": make_rx(), "lumped": lox.GtModel(KA_BAND, 30.0 * lox.dB)}
    gs = lox.GroundStation(
        "berlin", location, mask, tx_terminals=tx_terminals, rx_terminals=rx_terminals
    )
    assert gs.tx_terminals() == tx_terminals
    assert gs.rx_terminals() == rx_terminals

    tle = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""
    sc = lox.Spacecraft(
        "iss", lox.SGP4(tle), tx_terminals=tx_terminals, rx_terminals=rx_terminals
    )
    assert sc.tx_terminals() == tx_terminals
    assert sc.rx_terminals() == rx_terminals
    with pytest.raises(ValueError, match="expected a TxChain or EirpModel"):
        lox.Spacecraft("bad", lox.SGP4(tle), tx_terminals={"x": "not a terminal"})


def test_antenna_patterns_reject_non_physical_parameters():
    with pytest.raises(ValueError, match="diameter"):
        lox.ParabolicPattern(diameter=-0.5 * lox.m, efficiency=0.45)
    with pytest.raises(ValueError, match="efficiency"):
        lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=1.5)
    with pytest.raises(ValueError, match="efficiency"):
        lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.0)
    with pytest.raises(ValueError, match="length"):
        lox.DipolePattern(length=0.0 * lox.m)
    with pytest.raises(ValueError, match="beamwidth"):
        lox.ParabolicPattern.from_beamwidth(0.0 * lox.deg, 29.0 * lox.GHz, 0.45)
    with pytest.raises(ValueError, match="gain"):
        lox.ConstantAntenna(gain=float("nan") * lox.dB)


# --- Coverage: binding error paths, reprs, and pickles ---


def test_amplifier_transmitter_accessors_repr_pickle():
    tx = lox.AmplifierTransmitter( power=10.0 * lox.W, output_back_off=0.5 * lox.dB
    )
    assert float(tx.power) == pytest.approx(10.0)
    assert float(tx.output_back_off) == pytest.approx(0.5)
    assert "AmplifierTransmitter" in repr(tx)
    restored = pickle.loads(pickle.dumps(tx))
    assert restored == tx
    assert tx != lox.AmplifierTransmitter(power=5.0 * lox.W)
    with pytest.raises(ValueError, match="transmit power"):
        lox.AmplifierTransmitter(power=-1.0 * lox.W)


def test_cascade_receiver_accessors_and_validation():
    rx = lox.CascadeReceiver(
        stages=[lox.NoiseStage(35.0 * lox.dB, 50.0 * lox.K)],
        demodulator_loss=0.5 * lox.dB,
    )
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(50.0)
    assert float(rx.chain_gain()) == pytest.approx(35.0)
    assert "CascadeReceiver" in repr(rx)
    restored = pickle.loads(pickle.dumps(rx))
    assert restored == rx
    with pytest.raises(ValueError, match="demodulator loss"):
        lox.CascadeReceiver( stages=[], demodulator_loss=-0.5 * lox.dB
        )
    with pytest.raises(ValueError, match="stage count"):
        lox.CascadeReceiver(stages=[])
    with pytest.raises(ValueError, match="stage noise temperature"):
        lox.NoiseStage(35.0 * lox.dB, -50.0 * lox.K)
    lna = lox.CascadeReceiver.from_lna_and_noise_figure(
        lna_gain=20.0 * lox.dB,
        lna_noise_temperature=175.0 * lox.K,
        receiver_noise_figure=2.0 * lox.dB,
    )
    assert lna.chain_noise_temperature().to_kelvin() == pytest.approx(176.696, abs=0.01)


def test_terminal_error_paths():
    # Negative feed loss and antenna noise temperature are non-physical.
    with pytest.raises(ValueError, match="antenna noise temperature"):
        lox.RxChain(
            lox.ConstantAntenna(gain=30.0 * lox.dB),
            lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K),
            band=KA_BAND,
            antenna_noise_temperature=-1.0 * lox.K,
        )
    # Non-finite lumped figures are rejected.
    with pytest.raises(ValueError, match="EIRP"):
        lox.EirpModel(KA_BAND, float("nan") * lox.dB)
    with pytest.raises(ValueError, match="G/T"):
        lox.GtModel(KA_BAND, float("inf") * lox.dB)


def test_for_link_error_paths():
    tx = lox.EirpModel(KA_BAND, 55.0 * lox.dB)
    rx = lox.GtModel(KA_BAND, 3.01 * lox.dB)
    # Wrong terminal types
    with pytest.raises(ValueError, match="expected a TxChain or EirpModel"):
        lox.LinkBudget(
            rx, rx,
            carrier=29.0 * lox.GHz, range=1000.0 * lox.km,
        )
    with pytest.raises(ValueError, match="expected an RxChain or GtModel"):
        lox.LinkBudget(
            tx, tx,
            carrier=29.0 * lox.GHz, range=1000.0 * lox.km,
        )
    # Non-physical link inputs
    with pytest.raises(ValueError, match="non-physical"):
        lox.LinkBudget(
            tx, rx,
            carrier=29.0 * lox.GHz, range=0.0 * lox.km,
        )


def test_out_of_band_terminal_figures_are_rejected():
    # eirp_at/gt_at enforce the same carrier-in-band check as for_link
    # instead of extrapolating the hardware figures out of band.
    with pytest.raises(ValueError, match="outside the supported range"):
        lox.EirpModel(KA_BAND, 55.0 * lox.dB).eirp_at(2.4 * lox.GHz)
    with pytest.raises(ValueError, match="outside the supported range"):
        lox.GtModel(KA_BAND, 3.0 * lox.dB).gt_at(2.4 * lox.GHz)


def test_terminal_pickle_round_trips():
    tx = make_tx()
    assert pickle.loads(pickle.dumps(tx)) == tx
    rx = make_rx()
    assert pickle.loads(pickle.dumps(rx)) == rx
    eirp = lox.EirpModel(KA_BAND, 55.0 * lox.dB)
    assert pickle.loads(pickle.dumps(eirp)) == eirp
    gt = lox.GtModel(KA_BAND, 3.01 * lox.dB)
    assert pickle.loads(pickle.dumps(gt)) == gt


def test_chains_with_patterned_antenna_and_cascade_receiver():
    # Exercises the Patterned/Cascade arms of the Python bridge: getters,
    # reprs, and pickling.
    antenna = lox.PatternedAntenna(
        pattern=lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    )
    cascade = lox.CascadeReceiver(
        stages=[lox.NoiseStage(20.0 * lox.dB, 50.0 * lox.K)],
        demodulator_loss=0.5 * lox.dB,
    )
    tx = lox.TxChain(
        antenna,
        lox.AmplifierTransmitter(power=10.0 * lox.W),
        band=KA_BAND,
        feed_loss=1.0 * lox.dB,
    )
    rx = lox.RxChain(
        antenna,
        cascade,
        band=KA_BAND,
        antenna_noise_temperature=100.0 * lox.K,
    )

    assert isinstance(tx.antenna, lox.PatternedAntenna)
    assert isinstance(rx.antenna, lox.PatternedAntenna)
    assert isinstance(rx.receiver, lox.CascadeReceiver)
    assert "PatternedAntenna" in repr(tx)
    assert "CascadeReceiver" in repr(rx)
    assert pickle.loads(pickle.dumps(tx)) == tx
    assert pickle.loads(pickle.dumps(rx)) == rx
    # T_sys = 100 + 50 = 150 K
    assert rx.system_noise_temperature().to_kelvin() == pytest.approx(150.0)


def test_chain_direction_pointing_methods():
    # eirp_at/gt_at accept line-of-sight direction vectors directly.
    frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
    antenna = lox.PatternedAntenna(
        pattern=lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45),
        frame=frame,
    )
    tx = lox.TxChain(
        antenna, lox.AmplifierTransmitter(power=10.0 * lox.W), band=KA_BAND
    )
    on = tx.eirp_at(29.0 * lox.GHz, direction=[1.0, 0.0, 0.0])
    off = tx.eirp_at(29.0 * lox.GHz, direction=[0.0, 0.0, 1.0])
    assert float(off) < float(on)

    rx = lox.RxChain(
        antenna,
        lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K),
        band=KA_BAND,
        antenna_noise_temperature=0.0 * lox.K,
    )
    on = rx.gt_at(29.0 * lox.GHz, direction=[1.0, 0.0, 0.0])
    off = rx.gt_at(29.0 * lox.GHz, direction=[0.0, 0.0, 1.0])
    assert float(off) < float(on)
    with pytest.raises(ValueError, match="angle or direction"):
        rx.gt_at(29.0 * lox.GHz, angle=1.0 * lox.deg, direction=[1.0, 0.0, 0.0])


def test_model_direction_pointing_is_ignored():
    eirp = lox.EirpModel(KA_BAND, 55.0 * lox.dB)
    assert float(
        eirp.eirp_at(29.0 * lox.GHz, direction=[0.0, 0.0, 1.0])
    ) == pytest.approx(55.0)
    gt = lox.GtModel(KA_BAND, 3.0 * lox.dB)
    assert float(
        gt.gt_at(29.0 * lox.GHz, direction=[0.0, 0.0, 1.0])
    ) == pytest.approx(3.0)
    with pytest.raises(ValueError, match="angle or direction"):
        eirp.eirp_at(29.0 * lox.GHz, angle=1.0 * lox.deg, direction=[1.0, 0.0, 0.0])


def test_terminal_inequality():
    assert not (make_tx() == make_tx(gain=30.0))
    assert not (make_rx() == make_rx(gain=20.0))
    assert not (
        lox.EirpModel(KA_BAND, 55.0 * lox.dB) == lox.EirpModel(KA_BAND, 50.0 * lox.dB)
    )
    assert not (
        lox.GtModel(KA_BAND, 3.0 * lox.dB) == lox.GtModel(KA_BAND, 5.0 * lox.dB)
    )


def test_terminal_reprs():
    assert repr(make_tx()).startswith("TxChain(antenna=ConstantAntenna(")
    assert repr(make_rx()).startswith("RxChain(antenna=ConstantAntenna(")
    assert "receiver=NoiseTempReceiver(" in repr(make_rx())


def test_spacecraft_terminals():
    tx = lox.EirpModel(KA_BAND, 30.0 * lox.dB)
    tle = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""
    sc = lox.Spacecraft("iss", lox.SGP4(tle), tx_terminals={"beacon": tx})
    assert sc.tx_terminals()["beacon"] == tx
    assert sc.rx_terminals() == {}


def test_antenna_and_pattern_pickle_round_trips():
    parabolic = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    gaussian = lox.GaussianPattern(diameter=0.98 * lox.m, efficiency=0.45)
    dipole = lox.DipolePattern(length=0.0185 * lox.m)
    for pattern in [parabolic, gaussian, dipole]:
        restored = pickle.loads(pickle.dumps(pattern))
        assert restored == pattern

    frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
    antenna = lox.PatternedAntenna(pattern=parabolic, frame=frame)
    restored = pickle.loads(pickle.dumps(antenna))
    assert restored == antenna

    constant = lox.ConstantAntenna(gain=46.0 * lox.dB)
    restored = pickle.loads(pickle.dumps(constant))
    assert restored == constant
    assert constant != lox.ConstantAntenna(gain=30.0 * lox.dB)

    receiver = lox.NoiseTempReceiver(noise_temperature=500.0 * lox.K)
    restored = pickle.loads(pickle.dumps(receiver))
    assert restored == receiver
