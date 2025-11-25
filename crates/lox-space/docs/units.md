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

### Distance

| Constant | Value |
|----------|-------|
| `m` | 1 meter |
| `km` | 1000 meters |
| `au` | 1 astronomical unit |

### Frequency

| Constant | Value |
|----------|-------|
| `hz` | 1 Hz |
| `khz` | 1 kHz |
| `mhz` | 1 MHz |
| `ghz` | 1 GHz |
| `thz` | 1 THz |

### Velocity

| Constant | Value |
|----------|-------|
| `ms` | 1 m/s |
| `kms` | 1 km/s |

## Quick Example

```python
import lox_space as lox

# Use unit constants for readable code
angle = 45 * lox.deg
distance = 100 * lox.km
frequency = 2.4 * lox.ghz
velocity = 7.8 * lox.kms

# Convert to float
angle_rad = float(angle)
distance_m = float(distance)
```

---

::: lox_space.Angle
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

::: lox_space.Velocity
    options:
      show_source: false
