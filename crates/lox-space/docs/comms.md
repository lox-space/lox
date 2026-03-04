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

## Quick Example

```python
import lox_space as lox

# Define a Ka-band downlink
frequency = 29 * lox.GHz

# Transmitter: satellite with parabolic antenna
tx_pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
tx_antenna = lox.ComplexAntenna(pattern=tx_pattern, boresight=[0.0, 0.0, 1.0])
tx = lox.Transmitter(frequency=frequency, power=10 * lox.W, line_loss=1.0 * lox.dB)
tx_system = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx)

# Receiver: ground station with known system noise temperature
rx_antenna = lox.SimpleAntenna(gain=40.0 * lox.dB, beamwidth=0.5 * lox.deg)
rx = lox.SimpleReceiver(frequency=frequency, system_noise_temperature=200 * lox.K)
rx_system = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx)

# Define a QPSK channel at 10 Mbit/s
channel = lox.Channel(
    link_type="downlink",
    data_rate=10 * lox.Mbps,
    required_eb_n0=10.0 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)

# Compute a full link budget at 1000 km slant range
stats = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    channel=channel,
    range=1000 * lox.km,
    tx_angle=0.0 * lox.deg,
    rx_angle=0.0 * lox.deg,
)

print(f"EIRP:        {float(stats.eirp):.1f} dBW")
print(f"FSPL:        {float(stats.fspl):.1f} dB")
print(f"C/N0:        {float(stats.c_n0):.1f} dB·Hz")
print(f"Eb/N0:       {float(stats.eb_n0):.1f} dB")
print(f"Link margin: {float(stats.margin):.1f} dB")
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
stats = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    channel=channel,
    range=1000 * lox.km,
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

::: lox_space.SimpleAntenna
    options:
      show_source: false

---

::: lox_space.ComplexAntenna
    options:
      show_source: false

---

::: lox_space.Transmitter
    options:
      show_source: false

---

::: lox_space.SimpleReceiver
    options:
      show_source: false

---

::: lox_space.ComplexReceiver
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

::: lox_space.fspl
    options:
      show_source: false

---

::: lox_space.freq_overlap
    options:
      show_source: false
