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
frequency = 29e9  # Hz

# Transmitter: satellite with parabolic antenna
tx_pattern = lox.ParabolicPattern(diameter_m=0.98, efficiency=0.45)
tx_antenna = lox.ComplexAntenna(pattern=tx_pattern, boresight=[0.0, 0.0, 1.0])
tx = lox.Transmitter(frequency_hz=frequency, power_w=10.0, line_loss_db=1.0)
tx_system = lox.CommunicationSystem(antenna=tx_antenna, transmitter=tx)

# Receiver: ground station with known system noise temperature
rx_antenna = lox.SimpleAntenna(gain_db=40.0, beamwidth_deg=0.5)
rx = lox.SimpleReceiver(frequency_hz=frequency, system_noise_temperature_k=200.0)
rx_system = lox.CommunicationSystem(antenna=rx_antenna, receiver=rx)

# Define a QPSK channel at 10 Mbit/s
channel = lox.Channel(
    link_type="downlink",
    data_rate=10e6,          # bps
    required_eb_n0_db=10.0,  # dB
    margin_db=3.0,           # dB
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)

# Compute a full link budget at 1000 km slant range
stats = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    channel=channel,
    range_km=1000.0,
    tx_angle_deg=0.0,
    rx_angle_deg=0.0,
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
gain = lox.Decibel(30.0)
gain_linear = lox.Decibel.from_linear(1000.0)

# Arithmetic
total = gain + lox.Decibel(3.0)   # 33.0 dB
diff = gain - lox.Decibel(10.0)   # 20.0 dB

# Convert back
print(f"{float(gain)} dB = {gain.to_linear():.0f} linear")
```

### Free-Space Path Loss

```python
import lox_space as lox

# FSPL at 1000 km range and 29 GHz
loss = lox.fspl(distance_km=1000.0, frequency_hz=29e9)
print(f"FSPL: {float(loss):.1f} dB")
```

### Environmental Losses

```python
import lox_space as lox

losses = lox.EnvironmentalLosses(
    rain_db=2.0,
    gaseous_db=0.3,
    atmospheric_db=0.5,
)
print(f"Total: {float(losses.total()):.1f} dB")

# Pass to LinkStats.calculate via the losses parameter
stats = lox.LinkStats.calculate(
    tx_system=tx_system,
    rx_system=rx_system,
    channel=channel,
    range_km=1000.0,
    tx_angle_deg=0.0,
    rx_angle_deg=0.0,
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
