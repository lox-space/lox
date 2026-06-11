<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Communications

RF link budget analysis for space communication systems.

## Modulation Schemes

| Name | Bits per Symbol |
|------|-----------------|
| `BPSK` | 1 |
| `QPSK` | 2 |
| `8PSK` | 3 |
| `16QAM` | 4 |
| `32QAM` | 5 |
| `64QAM` | 6 |
| `128QAM` | 7 |
| `256QAM` | 8 |

## Antenna Patterns

| Pattern | Description |
|---------|-------------|
| `ParabolicPattern` | Airy disk model for parabolic reflector antennas |
| `GaussianPattern` | Gaussian roll-off approximation |
| `DipolePattern` | Short and general dipole radiation patterns |

Pattern gain is evaluated at two angles: `theta`, the polar angle from
boresight, and `phi`, the azimuth about boresight measured from the
antenna-frame `+X` axis toward `+Y`. All built-in patterns are axially
symmetric and ignore `phi`, so it can be omitted:

```python
import lox_space as lox

pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
gain = pattern.gain(29 * lox.GHz, theta=1.0 * lox.deg)
```

## Antenna Frames

An `AntennaFrame` orients an antenna pattern in a parent frame as a
right-handed orthonormal basis: `+Z` is the boresight (`theta = 0`) and `+X`
is the `phi = 0` reference direction. Construct it from a boresight and a
reference direction, or use `AntennaFrame.identity()` for a frame aligned
with the parent frame (boresight along `+Z`):

```python
import lox_space as lox

frame = lox.AntennaFrame.from_boresight_and_reference(
    boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0]
)
theta, phi = frame.angles_for([0.0, 0.0, 1.0])
```

A `PatternedAntenna` combines a pattern with a frame (identity by default)
and can evaluate its gain directly toward a parent-frame direction vector
via `gain_toward`:

```python
pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
antenna = lox.PatternedAntenna(pattern=pattern, frame=frame)
gain = antenna.gain_toward(29 * lox.GHz, [1.0, 0.0, 0.0])  # on boresight
```

## Link Terminals

A link terminal is one end of a radio link. The component tier composes an
antenna with a radio, a feed loss, and the supported frequency range: a
`TxChain` is an antenna fed by an `AmplifierTransmitter`, an `RxChain` is a
`NoiseTempReceiver` or `CascadeReceiver` fed by an antenna. Components are
pure physics (power, noise, gain patterns); the band lives on the terminal.
Shared hardware is expressed by reusing the same component values — a
diplexer-style transceiver is a TX and an RX chain sharing one dish:

```python
import lox_space as lox

ka_band = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)

dish = lox.ConstantAntenna(gain=46.0 * lox.dB)
tx = lox.TxChain(
    dish,
    lox.AmplifierTransmitter(power=10 * lox.W),
    band=ka_band,
    feed_loss=1.0 * lox.dB,
)
rx = lox.RxChain(
    dish,
    lox.NoiseTempReceiver(noise_temperature=500 * lox.K),
    band=ka_band,
    antenna_noise_temperature=0.0 * lox.K,
    feed_loss=0.5 * lox.dB,
)
```

Wherever a band is expected, an IEEE letter-band name is accepted as
shorthand for the full band range (case-insensitive):

```python
tx = lox.TxChain(
    dish,
    lox.AmplifierTransmitter(power=10 * lox.W),
    band="Ka",  # 27–40 GHz
    feed_loss=1.0 * lox.dB,
)
```

Terminals expose their headline figures directly — `eirp_at` and `gt_at`
evaluate at a carrier and an optional pointing:

```python
eirp = tx.eirp_at(29.0 * lox.GHz)  # 46 + 10·log10(10) − 1 = 55 dBW
gt = rx.gt_at(29.0 * lox.GHz, angle=1.0 * lox.deg)
t_sys = rx.system_noise_temperature()
```

`LinkStats.for_link` computes a link budget between two terminals. The
carrier is a link-level input and must lie inside both terminals'
frequency ranges:

```python
link = lox.LinkStats.for_link(
    tx,
    rx,
    carrier=29.0 * lox.GHz,
    bandwidth=5.0 * lox.MHz,
    range=1000.0 * lox.km,
)
print(f"C/N0 = {float(link.c_n0):.2f} dB·Hz")
```

Terminals attach to assets for scenario analysis as named collections via
`GroundStation(..., tx_terminals={...}, rx_terminals={...})` and
`Spacecraft(..., tx_terminals={...}, rx_terminals={...})`; selecting a
terminal by name is the configuration choice (e.g. high-gain vs. low-gain
antenna).

## Lumped EIRP and G/T

For early-phase mission studies — where manufacturer datasheets typically
publish only aggregate figures — you can build a link budget directly from
lumped `EirpModel` and `GtModel` terminals:

```python
import lox_space as lox

link = lox.LinkStats.for_link(
    lox.EirpModel("Ka", 55.0 * lox.dB),
    lox.GtModel("Ka", 3.01 * lox.dB),
    carrier=29.0 * lox.GHz,
    bandwidth=5.0 * lox.MHz,
    range=1000.0 * lox.km,
)
print(f"C/N0 = {float(link.c_n0):.2f} dB·Hz")
```

For lumped links, `link.carrier_rx_power` and `link.noise_power` are `None` —
the absolute carrier and noise power are not recoverable from EIRP and G/T
alone. The carrier-to-noise density ratio (`c_n0`) and carrier-to-noise ratio
(`c_n`) remain available.

To compute modulation-aware figures (`Es/N0`, `Eb/N0`, link margin), apply a
`Channel`:

```python
channel = lox.Channel(
    link_type="downlink",
    symbol_rate=5 * lox.MHz,
    required_eb_n0=10.0 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)
modulated = channel.apply(link)
print(f"Margin = {float(modulated.margin):.2f} dB")
```

Use the component tier (configure antennas, amplifiers, receiver noise) when
you need the full breakdown — for example for noise-budget allocation or
detailed component trade studies.

## Quick Example

An end-to-end budget for a LEO Earth-observation X-band downlink: a 500 km
satellite transmitting in the 8.025–8.4 GHz EESS band to a 3.7 m ground
station at 5° elevation. (The same scenario is available as a Rust example:
`cargo run --example x_band_downlink -p lox-space`.)

```python
import lox_space as lox

eess_band = lox.FrequencyRange(8.025 * lox.GHz, 8.4 * lox.GHz)
carrier = 8.2 * lox.GHz
elevation = 5.0 * lox.deg
slant_range = lox.slant_range(elevation, 6371.0 * lox.km, 500.0 * lox.km)

# Spacecraft: 0.25 m gimballed dish, 2 W amplifier, 0.8 dB feed run
spacecraft = lox.TxChain(
    lox.PatternedAntenna(pattern=lox.ParabolicPattern(0.25 * lox.m, 0.6)),
    lox.AmplifierTransmitter(power=2.0 * lox.W, output_back_off=0.5 * lox.dB),
    band=eess_band,
    feed_loss=0.8 * lox.dB,
)

# Ground station: 3.7 m dish, Friis-cascade front end (LNA → downconverter),
# 0.3 dB feed run, 60 K clear-sky antenna noise temperature
front_end = lox.CascadeReceiver(
    stages=[
        lox.NoiseStage(35.0 * lox.dB, 50.0 * lox.K),
        lox.NoiseStage(0.0 * lox.dB, 1540.0 * lox.K),  # NF ≈ 8 dB
    ],
    demodulator_loss=0.5 * lox.dB,
    implementation_loss=0.5 * lox.dB,
)
ground_station = lox.RxChain(
    lox.PatternedAntenna(pattern=lox.ParabolicPattern(3.7 * lox.m, 0.6)),
    front_end,
    band=eess_band,
    antenna_noise_temperature=60.0 * lox.K,
    feed_loss=0.3 * lox.dB,
)

# Atmospherics at X-band, 5° elevation (static values; lox-itur computes
# them from the ITU-R P-series maps)
losses = lox.EnvironmentalLosses.from_values(
    rain=1.2 * lox.dB,
    gaseous=0.4 * lox.dB,
    scintillation=0.3 * lox.dB,
    cloud=0.1 * lox.dB,
)

# DVB-S2-style channel: QPSK 3/4 at 150 Msps → 225 Mbit/s net
channel = lox.Channel(
    link_type="downlink",
    symbol_rate=150 * lox.MHz,
    required_eb_n0=4.5 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.25,
    fec=0.75,
)

# 2° residual pointing error on the spacecraft gimbal; the station
# autotracks on boresight.
link = lox.LinkStats.for_link(
    spacecraft,
    ground_station,
    carrier=carrier,
    bandwidth=channel.bandwidth(),
    range=slant_range,
    tx_angle=2.0 * lox.deg,
    losses=losses,
)
modulated = channel.apply(link)

print(f"EIRP:        {float(link.eirp):.2f} dBW")
print(f"FSPL:        {float(link.fspl):.2f} dB")
print(f"G/T:         {float(link.gt):.2f} dB/K")
print(f"C/N0:        {float(link.c_n0):.2f} dB·Hz")
print(f"Eb/N0:       {float(modulated.eb_n0):.2f} dB")
print(f"Data rate:   {float(channel.information_rate()) / 1e6:.1f} Mbit/s")
print(f"Link margin: {float(modulated.margin):.2f} dB")

# Regulatory check: PFD on the ground vs. the RR Art. 21.16 mask
pfd = lox.power_flux_density(link.eirp, slant_range, channel.bandwidth(), 4.0 * lox.kHz)
mask = lox.PfdMask.art_21_16(-150.0 * lox.dB)
assert float(pfd) <= float(mask.value_at(elevation))
```

### Direction-Aware Pointing

For patterned antennas the link budget can derive the pattern angles directly
from a line-of-sight vector expressed in the antenna's parent frame, using the
antenna's `AntennaFrame`:

```python
import lox_space as lox

# Dish boresight along +X
frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
antenna = lox.PatternedAntenna(
    pattern=lox.ParabolicPattern(0.25 * lox.m, 0.6), frame=frame
)
spacecraft = lox.TxChain(
    antenna,
    lox.AmplifierTransmitter(power=2.0 * lox.W),
    band=eess_band,
    feed_loss=0.8 * lox.dB,
)

link = lox.LinkStats.for_link(
    spacecraft,
    ground_station,
    carrier=carrier,
    bandwidth=channel.bandwidth(),
    range=slant_range,
    tx_direction=[0.9, 0.1, 0.0],  # line of sight in the TX parent frame
)
```

### Working with Decibels

```python
import lox_space as lox

# Create from dB value or linear ratio
gain = 30.0 * lox.dB
gain_linear = lox.Decibel.from_linear(1000.0)

# Arithmetic
total = gain + 3.0 * lox.dB   # 33.0 dB
diff = gain - 10.0 * lox.dB   # 20.0 dB

# Convert back
print(f"{float(gain)} dB = {gain.to_linear():.0f} linear")
```

### Free-Space Path Loss

```python
import lox_space as lox

# FSPL at 1000 km range and 29 GHz
loss = lox.fspl(distance=1000 * lox.km, frequency=29 * lox.GHz)
print(f"FSPL: {float(loss):.1f} dB")
```

### Environmental Losses

```python
import lox_space as lox

losses = lox.EnvironmentalLosses.from_values(
    rain=2.0 * lox.dB,
    gaseous=0.3 * lox.dB,
    atmospheric=0.5 * lox.dB,
)
print(f"Total: {float(losses.total()):.1f} dB")

# Pass to LinkStats.for_link via the losses parameter
link = lox.LinkStats.for_link(
    spacecraft,
    ground_station,
    carrier=carrier,
    bandwidth=channel.bandwidth(),
    range=slant_range,
    losses=losses,
)
```

---

::: lox_space.Decibel
    options:
      show_source: false

---

::: lox_space.Modulation
    options:
      show_source: false

---

::: lox_space.ParabolicPattern
    options:
      show_source: false

---

::: lox_space.GaussianPattern
    options:
      show_source: false

---

::: lox_space.DipolePattern
    options:
      show_source: false

---

::: lox_space.ConstantAntenna
    options:
      show_source: false

---

::: lox_space.AntennaFrame
    options:
      show_source: false

---

::: lox_space.PatternedAntenna
    options:
      show_source: false

---

::: lox_space.AmplifierTransmitter
    options:
      show_source: false

---

::: lox_space.NoiseTempReceiver
    options:
      show_source: false

---

::: lox_space.CascadeReceiver
    options:
      show_source: false

---

::: lox_space.NoiseStage
    options:
      show_source: false

---

::: lox_space.Channel
    options:
      show_source: false

---

::: lox_space.EnvironmentalLosses
    options:
      show_source: false

---

::: lox_space.LinkStats
    options:
      show_source: false

---

::: lox_space.ModulatedLinkStats
    options:
      show_source: false

---

::: lox_space.fspl
    options:
      show_source: false

---

::: lox_space.freq_overlap
    options:
      show_source: false

---

::: lox_space.power_flux_density
    options:
      show_source: false

---

::: lox_space.PfdMask
    options:
      show_source: false

---

::: lox_space.FrequencyRange
    options:
      show_source: false

---

::: lox_space.TxChain
    options:
      show_source: false

---

::: lox_space.RxChain
    options:
      show_source: false

---

::: lox_space.EirpModel
    options:
      show_source: false

---

::: lox_space.GtModel
    options:
      show_source: false

---

::: lox_space.slant_range
    options:
      show_source: false
