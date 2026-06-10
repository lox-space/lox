# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math
import pickle

import pytest
import lox_space as lox


KA_BAND = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)


def make_tx_payload(gain=46.0, power=10.0, feed_loss=1.0, antenna=None):
    """Standard transmit payload: constant-gain antenna + Ka-band amplifier."""
    if antenna is None:
        antenna = lox.ConstantAntenna(gain=gain * lox.dB)
    return lox.CommsPayload.transmitter_only(
        "tx",
        antenna,
        lox.AmplifierTransmitter(band=KA_BAND, power=power * lox.W),
        feed_loss=feed_loss * lox.dB,
    )


def make_rx_payload(gain=30.0, noise_temperature=500.0, feed_loss=0.0):
    """Standard receive payload: constant-gain antenna + 500 K receiver."""
    return lox.CommsPayload.receiver_only(
        "rx",
        lox.ConstantAntenna(gain=gain * lox.dB),
        lox.NoiseTempReceiver(
            band=KA_BAND, noise_temperature=noise_temperature * lox.K
        ),
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=feed_loss * lox.dB,
    )


def link_stats(tx, rx, **kwargs):
    """Evaluates the standard 29 GHz downlink between two payloads."""
    tx_payload, tx_terminal = tx
    rx_payload, rx_terminal = rx
    kwargs.setdefault("carrier", 29.0 * lox.GHz)
    kwargs.setdefault("bandwidth", 5.0 * lox.MHz)
    kwargs.setdefault("range", 1000.0 * lox.km)
    kwargs.setdefault("direction", "downlink")
    return lox.LinkStats.for_link(
        tx_payload, tx_terminal, rx_payload, rx_terminal, **kwargs
    )


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
    tx = make_tx_payload(gain=10.0, power=5.0, feed_loss=1.0)
    stats = link_stats(tx, make_rx_payload())
    assert float(stats.eirp) == pytest.approx(15.99, abs=0.01)


def test_transmitter_getters():
    tx = lox.AmplifierTransmitter(
        band=KA_BAND, power=10.0 * lox.W, output_back_off=0.5 * lox.dB
    )
    assert tx.band == KA_BAND
    assert tx.power.to_watts() == pytest.approx(10.0, abs=1e-12)
    assert float(tx.output_back_off) == pytest.approx(0.5, abs=1e-12)


def test_transmitter_default_obo():
    tx = lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    assert repr(tx).endswith("output_back_off=Decibel(0.0))")


def test_transmitter_eq():
    a = lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    b = lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    c = lox.AmplifierTransmitter(band=KA_BAND, power=5.0 * lox.W)
    assert a == b
    assert not (a == c)


def test_transmitter_pickle():
    tx = lox.AmplifierTransmitter(
        band=KA_BAND,
        power=10.0 * lox.W,
        output_back_off=0.5 * lox.dB,
    )
    assert pickle.loads(pickle.dumps(tx)) == tx


def test_transmitter_repr_roundtrip():
    tx = lox.AmplifierTransmitter(
        band=KA_BAND,
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
        band=KA_BAND,
        lna_gain=20.0 * lox.dB,
        lna_noise_temperature=175.0 * lox.K,
        receiver_noise_figure=2.0 * lox.dB,
    )
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(176.696, abs=0.01)


def test_simple_receiver_eq():
    a = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    b = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    c = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=600.0 * lox.K)
    assert a == b
    assert not (a == c)


def test_simple_receiver_getters():
    rx = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    assert rx.band == KA_BAND
    assert rx.noise_temperature.to_kelvin() == pytest.approx(500.0, abs=1e-12)


def test_simple_receiver_pickle():
    rx = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    assert pickle.loads(pickle.dumps(rx)) == rx


def test_simple_receiver_repr_roundtrip():
    rx = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
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
        band=KA_BAND,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
            lox.NoiseStage(gain=10.0 * lox.dB, noise_temperature=500.0 * lox.K),
        ],
    )
    # T_chain = 50 + 500/100 = 55 K
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(55.0, abs=0.01)


def test_complex_receiver_eq():
    kwargs = dict(
        band=KA_BAND,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=50.0 * lox.K),
        ],
    )
    a = lox.CascadeReceiver(**kwargs)
    b = lox.CascadeReceiver(**kwargs)
    c = lox.CascadeReceiver(
        band=KA_BAND,
        stages=[
            lox.NoiseStage(gain=20.0 * lox.dB, noise_temperature=100.0 * lox.K),
        ],
    )
    assert a == b
    assert not (a == c)


def test_complex_receiver_pickle():
    rx = lox.CascadeReceiver(
        band=KA_BAND,
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
        band=KA_BAND,
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


# --- Link Stats ---


def test_link_stats_noise_power():
    stats = link_stats(make_tx_payload(), make_rx_payload(), bandwidth=1.0 * lox.MHz)
    assert float(stats.noise_power) == pytest.approx(-141.61, abs=0.01)


def test_link_stats_end_to_end():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    stats = link_stats(
        make_tx_payload(),
        make_rx_payload(),
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

    stats_no_loss = link_stats(
        make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth()
    )
    stats_loss = link_stats(
        make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth(), losses=losses
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


# --- LinkStats additional outputs ---


def test_link_stats_carrier_power():
    stats = link_stats(make_tx_payload(), make_rx_payload())
    # carrier_rx_power should be finite for component-tier links
    assert math.isfinite(float(stats.carrier_rx_power))


def test_comms_payload_invalid_antenna():
    payload = lox.CommsPayload()
    with pytest.raises(ValueError, match="expected a ConstantAntenna or PatternedAntenna"):
        payload.add_antenna("bad", "not an antenna")


def test_comms_payload_invalid_receiver():
    payload = lox.CommsPayload()
    with pytest.raises(ValueError, match="expected NoiseTempReceiver or CascadeReceiver"):
        payload.add_receiver("bad", "not a receiver")


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
        band=KA_BAND,
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
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
        roll_off=0.35,
        fec=0.5,
    )

    stats = link_stats(make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth())

    # Test getters not covered in test_link_stats_end_to_end
    assert math.isfinite(float(stats.c_n))
    assert stats.carrier_rx_power is not None
    assert math.isfinite(float(stats.carrier_rx_power))
    assert stats.noise_power is not None
    assert math.isfinite(float(stats.noise_power))
    assert stats.bandwidth.to_hertz() == pytest.approx(5e6 * 1.35, rel=1e-6)
    assert math.isfinite(float(stats.gt))
    assert stats.direction == "downlink"


def test_link_stats_repr():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )

    stats = link_stats(make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth())
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
    stats = link_stats(make_tx_payload(antenna=a), make_rx_payload())
    assert math.isfinite(float(stats.eirp))


def test_transmitter_only_invalid_antenna():
    tx = lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    with pytest.raises(ValueError, match="expected a ConstantAntenna or PatternedAntenna"):
        lox.CommsPayload.transmitter_only(
            "tx", "not an antenna", tx, feed_loss=1.0 * lox.dB
        )


# --- Lumped-tier smoke test ---


def test_lumped_link_interference_requires_absolute_power():
    tx = lox.CommsPayload.eirp_only("eirp", KA_BAND, 55.0 * lox.dB)
    rx = lox.CommsPayload.gt_only("gt", KA_BAND, 3.01 * lox.dB)
    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    link = link_stats(tx, rx, bandwidth=channel.bandwidth())
    modulated = channel.apply(link)

    with pytest.raises(ValueError, match="absolute carrier and noise powers"):
        modulated.with_interference(1e-12 * lox.W)


# --- ModulatedLinkStats.with_interference happy path ---


def test_modulated_with_interference_component_tier():
    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )

    link = link_stats(
        make_tx_payload(), make_rx_payload(), bandwidth=channel.bandwidth()
    )
    modulated = channel.apply(link)
    interference = modulated.with_interference(1e-12 * lox.W)

    assert float(interference.margin_with_interference) < float(modulated.margin)
    assert float(interference.eb_n0i0) < float(modulated.eb_n0)
    assert float(interference.interference_power) == 1e-12


# --- NoiseTempReceiver repr ---


def test_simple_receiver_repr():
    rx = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    r = repr(rx)
    assert "NoiseTempReceiver" in r
    assert "500" in r


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


# --- LinkStats pattern-angle getters and pointing arguments ---


def test_link_stats_pattern_angle_getters():
    stats = link_stats(
        make_tx_payload(),
        make_rx_payload(),
        tx_angle=2.0 * lox.deg,
        rx_angle=1.0 * lox.deg,
    )
    assert stats.tx_theta.to_degrees() == pytest.approx(2.0, abs=1e-10)
    assert stats.tx_phi.to_degrees() == pytest.approx(0.0, abs=1e-10)
    assert stats.rx_theta.to_degrees() == pytest.approx(1.0, abs=1e-10)
    assert stats.rx_phi.to_degrees() == pytest.approx(0.0, abs=1e-10)


def test_link_stats_defaults_to_boresight():
    stats = link_stats(make_tx_payload(), make_rx_payload())
    assert stats.tx_theta.to_degrees() == pytest.approx(0.0, abs=1e-10)
    assert stats.rx_theta.to_degrees() == pytest.approx(0.0, abs=1e-10)
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.1)


def test_link_stats_direction_pointing():
    # Dish boresight along +X; a line of sight along +Z is 90° off boresight.
    frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
    pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
    tx_ant = lox.PatternedAntenna(pattern=pattern, frame=frame)

    on_axis = link_stats(
        make_tx_payload(antenna=tx_ant),
        make_rx_payload(),
        tx_direction=[1.0, 0.0, 0.0],
    )
    off_axis = link_stats(
        make_tx_payload(antenna=tx_ant),
        make_rx_payload(),
        tx_direction=[0.0, 0.0, 1.0],
    )

    assert on_axis.tx_theta.to_degrees() == pytest.approx(0.0, abs=1e-10)
    assert off_axis.tx_theta.to_degrees() == pytest.approx(90.0, abs=1e-10)
    assert float(off_axis.eirp) < float(on_axis.eirp)


def test_link_stats_rejects_angle_and_direction():
    with pytest.raises(ValueError, match="tx_angle or tx_direction"):
        link_stats(
            make_tx_payload(),
            make_rx_payload(),
            tx_angle=1.0 * lox.deg,
            tx_direction=[1.0, 0.0, 0.0],
        )


# --- ModulatedLinkStats link and channel getters ---


def test_modulated_link_stats_link_and_channel_getters():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    stats = link_stats(make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth())
    modulated = ch.apply(stats)

    # link getter returns the underlying PyLinkStats
    assert modulated.link.c_n0 == stats.c_n0
    # channel getter returns the PyChannel
    assert "QPSK" in repr(modulated.channel)


# --- InterferenceStats c_n0i0 and repr ---


def test_interference_stats_c_n0i0_and_repr():
    channel = lox.Channel(
        link_type="downlink",
        symbol_rate=5.0 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    link = link_stats(
        make_tx_payload(), make_rx_payload(), bandwidth=channel.bandwidth()
    )
    modulated = channel.apply(link)
    interference = modulated.with_interference(1e-12 * lox.W)

    assert math.isfinite(float(interference.c_n0i0))
    r = repr(interference)
    assert "InterferenceStats" in r
    assert "c_n0i0" in r


# --- ModulatedLinkStats repr ---


def test_modulated_link_stats_repr():
    ch = lox.Channel(
        link_type="downlink",
        symbol_rate=5 * lox.MHz,
        required_eb_n0=10.0 * lox.dB,
        margin=3.0 * lox.dB,
        modulation=lox.Modulation("QPSK"),
    )
    stats = link_stats(make_tx_payload(), make_rx_payload(), bandwidth=ch.bandwidth())
    modulated = ch.apply(stats)
    r = repr(modulated)
    assert "ModulatedLinkStats" in r
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
    }
    for name, expected_repr in expected.items():
        assert repr(lox.Modulation(name)) == expected_repr


# --- CommsPayload (inventory + wiring) and endpoint-based link budgets ---


def test_frequency_range():
    assert KA_BAND.contains(29.0 * lox.GHz)
    assert not KA_BAND.contains(45.0 * lox.GHz)
    assert KA_BAND.min().to_gigahertz() == pytest.approx(27.0)
    assert str(KA_BAND) == "27.000–31.000 GHz"
    optical = lox.FrequencyRange.from_wavelengths(1530e-9 * lox.m, 1565e-9 * lox.m)
    assert optical.contains(193.4 * lox.THz)
    with pytest.raises(ValueError):
        lox.FrequencyRange(31.0 * lox.GHz, 27.0 * lox.GHz)
    assert pickle.loads(pickle.dumps(KA_BAND)) == KA_BAND


def test_comms_payload_manual_wiring_diplexer():
    payload = lox.CommsPayload()
    dish = payload.add_antenna("dish", lox.ConstantAntenna(gain=46.0 * lox.dB))
    pa = payload.add_transmitter(
        "pa",
        lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W),
    )
    lnb = payload.add_receiver(
        "lnb",
        lox.NoiseTempReceiver(
            band=lox.FrequencyRange(17.0 * lox.GHz, 21.0 * lox.GHz),
            noise_temperature=500.0 * lox.K,
        ),
    )
    tx_port = payload.add_tx_port("tx leg", dish, pa, 1.0 * lox.dB, band=KA_BAND)
    rx_port = payload.add_rx_port(
        "rx leg",
        dish,
        lnb,
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=0.5 * lox.dB,
    )
    terminal = payload.add_transceiver_terminal(
        "ka transceiver", tx_port=tx_port, rx_port=rx_port
    )
    assert payload.find_terminal("ka transceiver") == terminal
    assert payload.find_terminal("nonexistent") is None


def test_comms_payload_for_link_reference_values():
    # 46 dBi + 10 W − 1 dB feed loss → EIRP 55 dBW;
    # 30 dBi / 500 K → G/T 3.0103 dB/K; C/N0 104.913 dB·Hz at 1000 km.
    stats = link_stats(make_tx_payload(), make_rx_payload())

    assert float(stats.eirp) == pytest.approx(55.0, abs=0.01)
    assert float(stats.gt) == pytest.approx(3.0103, abs=0.01)
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.01)
    assert math.isfinite(float(stats.carrier_rx_power))
    assert stats.direction == "downlink"


def test_comms_payload_lumped_link():
    tx_payload, tx_terminal = lox.CommsPayload.eirp_only("eirp", KA_BAND, 55.0 * lox.dB)
    rx_payload, rx_terminal = lox.CommsPayload.gt_only("gt", KA_BAND, 3.01 * lox.dB)
    stats = lox.LinkStats.for_link(
        tx_payload,
        tx_terminal,
        rx_payload,
        rx_terminal,
        carrier=29.0 * lox.GHz,
        bandwidth=5.0 * lox.MHz,
        range=1000.0 * lox.km,
        direction="uplink",
    )
    assert float(stats.c_n0) == pytest.approx(104.913, abs=0.1)
    assert stats.carrier_rx_power is None
    assert stats.noise_power is None


def test_comms_payload_carrier_out_of_band():
    tx_payload, tx_terminal = lox.CommsPayload.eirp_only("eirp", KA_BAND, 55.0 * lox.dB)
    rx_band = lox.FrequencyRange(17.0 * lox.GHz, 21.0 * lox.GHz)
    rx_payload, rx_terminal = lox.CommsPayload.gt_only("gt", rx_band, 3.01 * lox.dB)
    with pytest.raises(ValueError, match="outside the supported range"):
        lox.LinkStats.for_link(
            tx_payload,
            tx_terminal,
            rx_payload,
            rx_terminal,
            carrier=29.0 * lox.GHz,
            bandwidth=5.0 * lox.MHz,
            range=1000.0 * lox.km,
            direction="downlink",
        )


def test_comms_payload_rejects_mixed_chain_args():
    payload = lox.CommsPayload()
    with pytest.raises(ValueError, match="exactly one of"):
        payload.add_tx_terminal("bad")


def test_ground_station_comms_payload():
    payload, _terminal = lox.CommsPayload.gt_only("gs gt", KA_BAND, 30.0 * lox.dB)
    location = lox.GroundLocation(
        lox.Origin("Earth"), 13.4 * lox.deg, 52.5 * lox.deg, 0.1 * lox.km
    )
    mask = lox.ElevationMask.fixed(5.0 * lox.deg)
    gs = lox.GroundStation("berlin", location, mask, comms_payload=payload)
    restored = gs.comms_payload()
    assert restored is not None
    assert restored.find_terminal("gs gt") is not None

    bare = lox.GroundStation("bare", location, mask)
    assert bare.comms_payload() is None


def test_comms_payload_rejects_non_physical_inputs():
    payload = lox.CommsPayload()
    with pytest.raises(ValueError, match="non-physical"):
        payload.add_transmitter(
            "pa", lox.AmplifierTransmitter(band=KA_BAND, power=0.0 * lox.W)
        )
    with pytest.raises(ValueError, match="non-physical"):
        payload.add_receiver(
            "rx",
            lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=-10.0 * lox.K),
        )


def test_comms_payload_introspection():
    payload = lox.CommsPayload()
    dish = payload.add_antenna("dish", lox.ConstantAntenna(gain=46.0 * lox.dB))
    pa = payload.add_transmitter(
        "pa", lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    )
    lnb = payload.add_receiver(
        "lnb", lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    )
    tx_port = payload.add_tx_port("tx leg", dish, pa, 1.0 * lox.dB)
    rx_port = payload.add_rx_port(
        "rx leg",
        dish,
        lnb,
        antenna_noise_temperature=0.0 * lox.K,
        feed_loss=0.5 * lox.dB,
    )
    terminal = payload.add_transceiver_terminal(
        "ka transceiver", tx_port=tx_port, rx_port=rx_port
    )

    terminals = payload.terminals()
    assert len(terminals) == 1
    terminal_id, name, kind = terminals[0]
    assert terminal_id == terminal
    assert name == "ka transceiver"
    assert kind == "transceiver"

    assert payload.tx_band(terminal).contains(29.0 * lox.GHz)
    assert payload.rx_band(terminal).contains(29.0 * lox.GHz)

    # EIRP = 46 + 10 − 1 = 55 dBW; G/T = 30 − 26.99... but RX antenna is the
    # 46 dBi dish here: G/T = 46 − 10·log10(500 · 10^(0.05)) with 0.5 dB feed.
    eirp = payload.eirp_at(terminal, 29.0 * lox.GHz)
    assert float(eirp) == pytest.approx(55.0, abs=1e-9)
    gt = payload.gt_at(terminal, 29.0 * lox.GHz)
    expected_t_sys = 290.0 * (10 ** (0.5 / 10.0) - 1.0) + 500.0 * 10 ** (0.5 / 10.0)
    assert float(gt) == pytest.approx(46.0 - 10.0 * math.log10(expected_t_sys), abs=1e-9)

    description = payload.describe()
    assert "ka transceiver" in description
    assert "dish" in description
    assert "transceiver" in description
    assert str(payload) == description

    with pytest.raises(ValueError, match="angle or direction"):
        payload.eirp_at(
            terminal, 29.0 * lox.GHz, angle=1.0 * lox.deg, direction=[1.0, 0.0, 0.0]
        )


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
    tx = lox.AmplifierTransmitter(
        band=KA_BAND, power=10.0 * lox.W, output_back_off=0.5 * lox.dB
    )
    assert tx.band == KA_BAND
    assert float(tx.power) == pytest.approx(10.0)
    assert float(tx.output_back_off) == pytest.approx(0.5)
    assert "AmplifierTransmitter" in repr(tx)
    restored = pickle.loads(pickle.dumps(tx))
    assert restored == tx
    assert tx != lox.AmplifierTransmitter(band=KA_BAND, power=5.0 * lox.W)
    with pytest.raises(ValueError, match="transmit power"):
        lox.AmplifierTransmitter(band=KA_BAND, power=-1.0 * lox.W)


def test_cascade_receiver_accessors_and_validation():
    rx = lox.CascadeReceiver(
        band=KA_BAND,
        stages=[lox.NoiseStage(35.0 * lox.dB, 50.0 * lox.K)],
        demodulator_loss=0.5 * lox.dB,
    )
    assert rx.band == KA_BAND
    assert rx.chain_noise_temperature().to_kelvin() == pytest.approx(50.0)
    assert float(rx.chain_gain()) == pytest.approx(35.0)
    assert "CascadeReceiver" in repr(rx)
    restored = pickle.loads(pickle.dumps(rx))
    assert restored == rx
    with pytest.raises(ValueError, match="demodulator loss"):
        lox.CascadeReceiver(
            band=KA_BAND, stages=[], demodulator_loss=-0.5 * lox.dB
        )
    with pytest.raises(ValueError, match="stage count"):
        lox.CascadeReceiver(band=KA_BAND, stages=[])
    with pytest.raises(ValueError, match="stage noise temperature"):
        lox.NoiseStage(35.0 * lox.dB, -50.0 * lox.K)
    lna = lox.CascadeReceiver.from_lna_and_noise_figure(
        band=KA_BAND,
        lna_gain=20.0 * lox.dB,
        lna_noise_temperature=175.0 * lox.K,
        receiver_noise_figure=2.0 * lox.dB,
    )
    assert lna.chain_noise_temperature().to_kelvin() == pytest.approx(176.696, abs=0.01)


def test_payload_wiring_error_paths():
    payload = lox.CommsPayload()
    other = lox.CommsPayload()
    foreign_antenna = other.add_antenna(
        "foreign", lox.ConstantAntenna(gain=46.0 * lox.dB)
    )
    pa = payload.add_transmitter(
        "pa", lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    )
    # Foreign ID → unknown antenna
    with pytest.raises(ValueError, match="unknown antenna"):
        payload.add_tx_port("leg", foreign_antenna, pa, 0.0 * lox.dB)
    # Negative feed loss → non-physical
    dish = payload.add_antenna("dish", lox.ConstantAntenna(gain=46.0 * lox.dB))
    with pytest.raises(ValueError, match="feed loss"):
        payload.add_tx_port("leg", dish, pa, -1.0 * lox.dB)
    # Wrong input types
    with pytest.raises(ValueError, match="expected a ConstantAntenna"):
        payload.add_antenna("bad", 42)
    with pytest.raises(ValueError, match="expected"):
        payload.add_receiver("bad", "not a receiver")
    # Terminal chain arguments are mutually exclusive
    eirp = payload.add_eirp_model("eirp", KA_BAND, 55.0 * lox.dB)
    port = payload.add_tx_port("leg", dish, pa, 0.0 * lox.dB)
    with pytest.raises(ValueError, match="exactly one of"):
        payload.add_tx_terminal("bad", port=port, eirp_model=eirp)
    with pytest.raises(ValueError, match="exactly one of"):
        payload.add_rx_terminal("bad")
    # Non-finite lumped figures
    with pytest.raises(ValueError, match="EIRP"):
        payload.add_eirp_model("bad", KA_BAND, float("nan") * lox.dB)
    with pytest.raises(ValueError, match="G/T"):
        payload.add_gt_model("bad", KA_BAND, float("inf") * lox.dB)


def test_for_link_error_paths():
    tx_payload, tx_terminal = lox.CommsPayload.eirp_only("tx", KA_BAND, 55.0 * lox.dB)
    rx_payload, rx_terminal = lox.CommsPayload.gt_only("rx", KA_BAND, 3.01 * lox.dB)
    # Invalid direction string
    with pytest.raises(ValueError, match="unknown link direction"):
        lox.LinkStats.for_link(
            tx_payload, tx_terminal, rx_payload, rx_terminal,
            carrier=29.0 * lox.GHz, bandwidth=5.0 * lox.MHz,
            range=1000.0 * lox.km, direction="sideways",
        )
    # Wrong-direction terminal
    with pytest.raises(ValueError, match="no transmit chain"):
        lox.LinkStats.for_link(
            rx_payload, rx_terminal, rx_payload, rx_terminal,
            carrier=29.0 * lox.GHz, bandwidth=5.0 * lox.MHz,
            range=1000.0 * lox.km, direction="downlink",
        )
    # Non-physical link inputs
    with pytest.raises(ValueError, match="non-physical"):
        lox.LinkStats.for_link(
            tx_payload, tx_terminal, rx_payload, rx_terminal,
            carrier=29.0 * lox.GHz, bandwidth=0.0 * lox.MHz,
            range=1000.0 * lox.km, direction="downlink",
        )
    # Introspection on the wrong direction
    with pytest.raises(ValueError, match="no receive chain"):
        tx_payload.gt_at(tx_terminal, 29.0 * lox.GHz)
    with pytest.raises(ValueError, match="no transmit chain"):
        rx_payload.eirp_at(rx_terminal, 29.0 * lox.GHz)
    with pytest.raises(ValueError, match="no receive chain"):
        tx_payload.rx_band(tx_terminal)
    assert tx_payload.tx_band(tx_terminal) == KA_BAND


def test_out_of_band_terminal_figures_are_rejected():
    # eirp_at/gt_at enforce the same carrier-in-band check as for_link
    # instead of extrapolating the hardware figures out of band.
    payload, terminal = lox.CommsPayload.eirp_only("a", KA_BAND, 55.0 * lox.dB)
    with pytest.raises(ValueError, match="outside the supported range"):
        payload.eirp_at(terminal, 2.4 * lox.GHz)
    payload, terminal = lox.CommsPayload.gt_only("b", KA_BAND, 3.0 * lox.dB)
    with pytest.raises(ValueError, match="outside the supported range"):
        payload.gt_at(terminal, 2.4 * lox.GHz)


def test_foreign_terminal_ids_are_rejected():
    # Two payloads built with the same insertion order mint colliding
    # internal keys; the payload identity must keep the handles apart.
    payload_a, terminal_a = lox.CommsPayload.eirp_only("a", KA_BAND, 55.0 * lox.dB)
    payload_b, terminal_b = lox.CommsPayload.eirp_only("b", KA_BAND, 99.0 * lox.dB)
    assert terminal_a != terminal_b
    with pytest.raises(ValueError, match="unknown terminal"):
        payload_b.eirp_at(terminal_a, 29.0 * lox.GHz)
    assert float(payload_a.eirp_at(terminal_a, 29.0 * lox.GHz)) == pytest.approx(55.0)


def test_comms_payload_pickle_round_trips_with_fresh_ids():
    payload, terminal = lox.CommsPayload.eirp_only("a", KA_BAND, 55.0 * lox.dB)
    restored = pickle.loads(pickle.dumps(payload))

    restored_terminal = restored.find_terminal("a")
    assert restored_terminal is not None
    assert float(restored.eirp_at(restored_terminal, 29.0 * lox.GHz)) == pytest.approx(55.0)
    with pytest.raises(ValueError, match="unknown terminal"):
        restored.eirp_at(terminal, 29.0 * lox.GHz)


def test_transceiver_static_and_direction_getter():
    payload, terminal = lox.CommsPayload.transceiver(
        "sat",
        lox.ConstantAntenna(gain=46.0 * lox.dB),
        lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W),
        lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K),
        antenna_noise_temperature=150.0 * lox.K,
        tx_feed_loss=1.0 * lox.dB,
        rx_feed_loss=0.5 * lox.dB,
        band=KA_BAND,
    )
    terminals = payload.terminals()
    assert terminals[0][2] == "transceiver"
    assert "sat" in repr(payload)

    rx_payload, rx_terminal = lox.CommsPayload.gt_only("gs", KA_BAND, 30.0 * lox.dB)
    link = lox.LinkStats.for_link(
        payload, terminal, rx_payload, rx_terminal,
        carrier=29.0 * lox.GHz, bandwidth=5.0 * lox.MHz,
        range=1000.0 * lox.km, direction="downlink",
        rx_angle=0.0 * lox.deg,
    )
    assert link.direction == "downlink"
    assert float(link.eirp) == pytest.approx(46.0 + 10.0 - 1.0, abs=1e-9)


def test_id_reprs():
    payload = lox.CommsPayload()
    dish = payload.add_antenna("dish", lox.ConstantAntenna(gain=46.0 * lox.dB))
    assert "AntennaId" in repr(dish)
    pa = payload.add_transmitter(
        "pa", lox.AmplifierTransmitter(band=KA_BAND, power=10.0 * lox.W)
    )
    assert "TransmitterId" in repr(pa)
    eirp = payload.add_eirp_model("e", KA_BAND, 55.0 * lox.dB)
    assert "EirpModelId" in repr(eirp)
    gt = payload.add_gt_model("g", KA_BAND, 3.0 * lox.dB)
    assert "GtModelId" in repr(gt)
    terminal = payload.add_tx_terminal("t", eirp_model=eirp)
    assert "TerminalId" in repr(terminal)


def test_spacecraft_comms_payload():
    payload, _terminal = lox.CommsPayload.eirp_only("sc", KA_BAND, 30.0 * lox.dB)
    tle = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""
    sc = lox.Spacecraft("iss", lox.SGP4(tle), comms_payload=payload)
    restored = sc.comms_payload()
    assert restored is not None
    assert restored.find_terminal("sc") is not None


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

    receiver = lox.NoiseTempReceiver(band=KA_BAND, noise_temperature=500.0 * lox.K)
    restored = pickle.loads(pickle.dumps(receiver))
    assert restored == receiver
