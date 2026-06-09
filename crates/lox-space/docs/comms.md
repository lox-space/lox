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

## Lumped EIRP and G/T

For early-phase mission studies — where manufacturer datasheets typically
publish only aggregate figures — you can build a link budget directly from
an `EirpTransmitter` and a `GtReceiver`, without configuring an antenna or
amplifier separately. Use `CommunicationSystem.eirp_only` and
`CommunicationSystem.gt_only`:

```python
import lox_space as lox

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
    lox.EnvironmentalLosses.none(),
)
print(f"C/N0 = {link.c_n0.as_float():.2f} dB·Hz")
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
print(f"Margin = {modulated.margin.as_float():.2f} dB")
```

Use the component tier (configure antennas, amplifiers, receiver noise) when
you need the full breakdown — for example for noise-budget allocation or
detailed component trade studies.

## Quick Example

```python
import lox_space as lox

# Define a Ka-band downlink
frequency = 29 * lox.GHz

# Transmitter: satellite with parabolic antenna
tx_pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
tx_antenna = lox.PatternedAntenna(pattern=tx_pattern, boresight=[0.0, 0.0, 1.0])
tx = lox.AmplifierTransmitter(frequency=frequency, power=10 * lox.W, line_loss=1.0 * lox.dB)
tx_system = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx)

# Receiver: ground station with known system noise temperature
rx_antenna = lox.ConstantAntenna(gain=40.0 * lox.dB)
rx = lox.NoiseTempReceiver(frequency=frequency, system_noise_temperature=200 * lox.K)
rx_system = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx)

# Define a QPSK channel at 5 Msps
channel = lox.Channel(
    link_type="downlink",
    symbol_rate=5 * lox.MHz,
    required_eb_n0=10.0 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)

# Compute a full link budget at 1000 km slant range
link = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    range=1000 * lox.km,
    bandwidth=channel.bandwidth(),
    tx_angle=0.0 * lox.deg,
    rx_angle=0.0 * lox.deg,
)
modulated = channel.apply(link)

print(f"EIRP:        {float(link.eirp):.1f} dBW")
print(f"FSPL:        {float(link.fspl):.1f} dB")
print(f"C/N0:        {float(link.c_n0):.1f} dB·Hz")
print(f"Es/N0:       {float(modulated.es_n0):.1f} dB")
print(f"Eb/N0:       {float(modulated.eb_n0):.1f} dB")
print(f"Link margin: {float(modulated.margin):.1f} dB")
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

losses = lox.EnvironmentalLosses(
    rain=2.0 * lox.dB,
    gaseous=0.3 * lox.dB,
    atmospheric=0.5 * lox.dB,
)
print(f"Total: {float(losses.total()):.1f} dB")

# Pass to LinkStats.calculate via the losses parameter
link = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    range=1000 * lox.km,
    bandwidth=channel.bandwidth(),
    tx_angle=0.0 * lox.deg,
    rx_angle=0.0 * lox.deg,
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

::: lox_space.PatternedAntenna
    options:
      show_source: false

---

::: lox_space.AmplifierTransmitter
    options:
      show_source: false

---

::: lox_space.EirpTransmitter
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

::: lox_space.GtReceiver
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

::: lox_space.CommunicationSystem
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

::: lox_space.pfd_mask
    options:
      show_source: false

---

::: lox_space.slant_range
    options:
      show_source: false
