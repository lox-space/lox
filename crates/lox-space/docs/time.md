<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Time & Dates

High-precision time handling with femtosecond resolution and support for
multiple astronomical time scales.

## Time Scales

Lox supports the following astronomical time scales:

| Scale | Name | Description |
|-------|------|-------------|
| TAI | International Atomic Time | Primary atomic time scale |
| TT | Terrestrial Time | Used for geocentric ephemerides |
| TDB | Barycentric Dynamical Time | Used for solar system ephemerides |
| TCB | Barycentric Coordinate Time | Relativistic coordinate time |
| TCG | Geocentric Coordinate Time | Relativistic coordinate time |
| UT1 | Universal Time | Tied to Earth's rotation |

## Quick Example

```python
import lox_space as lox

# Create a time instant
t = lox.Time("TAI", 2024, 6, 15, 12, 30, 45.5)

# From ISO string
t = lox.Time.from_iso("2024-06-15T12:30:45.5 TAI")

# Convert between scales
t_tt = t.to_scale("TT")

# Time arithmetic
dt = lox.TimeDelta.from_hours(1.5)
t2 = t + dt

# Work with UTC
utc = lox.UTC(2024, 6, 15, 12, 30, 45.5)
t_tai = utc.to_scale("TAI")
```

---

::: lox_space.Time
    options:
      show_source: false

---

::: lox_space.UTC
    options:
      show_source: false

---

::: lox_space.TimeDelta
    options:
      show_source: false

---

::: lox_space.TimeScale
    options:
      show_source: false
