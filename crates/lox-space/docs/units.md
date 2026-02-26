<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Units

Physical quantity types for type-safe unit handling.

## Available Units

### Angle

| Constant | Value |
|----------|-------|
| `rad` | 1 radian |
| `deg` | π/180 radians |

### AngularRate

| Constant | Value |
|----------|-------|
| `rad_per_s` | 1 rad/s |
| `deg_per_s` | π/180 rad/s |

### DataRate

| Constant | Value |
|----------|-------|
| `bps` | 1 bit/s |
| `kbps` | 1 kbit/s |
| `Mbps` | 1 Mbit/s |

### Distance

| Constant | Value |
|----------|-------|
| `m` | 1 meter |
| `km` | 1000 meters |
| `au` | 1 astronomical unit |

### Frequency

| Constant | Value |
|----------|-------|
| `Hz` | 1 Hz |
| `kHz` | 1 kHz |
| `MHz` | 1 MHz |
| `GHz` | 1 GHz |
| `THz` | 1 THz |

### Power

| Constant | Value |
|----------|-------|
| `W` | 1 W |
| `kW` | 1 kW |

### Temperature

| Constant | Value |
|----------|-------|
| `K` | 1 K |

### Velocity

| Constant | Value |
|----------|-------|
| `m_per_s` | 1 m/s |
| `km_per_s` | 1 km/s |

### Decibel

| Constant | Value |
|----------|-------|
| `dB` | 1 dB |

## Quick Example

```python
import lox_space as lox

# Use unit constants for readable code
angle = 45 * lox.deg
distance = 100 * lox.km
frequency = 2.4 * lox.GHz
velocity = 7.8 * lox.km_per_s
power = 10 * lox.W
temperature = 290 * lox.K
data_rate = 10 * lox.Mbps

# Convert to specific units
angle_deg = angle.to_degrees()       # 45.0
distance_km = distance.to_kilometers()  # 100.0
freq_ghz = frequency.to_gigahertz()  # 2.4

# Convert to float (returns internal SI value)
angle_rad = float(angle)    # radians
distance_m = float(distance)  # meters

# Arithmetic
total = 100 * lox.km + 500 * lox.m   # Distance
delta = 45 * lox.deg - 10 * lox.deg  # Angle
scaled = 2.0 * distance              # Distance
```

---

::: lox_space.Angle
    options:
      show_source: false

---

::: lox_space.AngularRate
    options:
      show_source: false

---

::: lox_space.DataRate
    options:
      show_source: false

---

::: lox_space.Distance
    options:
      show_source: false

---

::: lox_space.Frequency
    options:
      show_source: false

---

::: lox_space.GravitationalParameter
    options:
      show_source: false

---

::: lox_space.Power
    options:
      show_source: false

---

::: lox_space.Temperature
    options:
      show_source: false

---

::: lox_space.Velocity
    options:
      show_source: false
