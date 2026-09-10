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

# Time arithmetic with unit constants
t2 = t + 1.5 * lox.hours
dt = 30 * lox.minutes

# Work with UTC
utc = lox.UTC(2024, 6, 15, 12, 30, 45.5)
t_tai = utc.to_scale("TAI")
```

## Durations

`lox.seconds`, `lox.minutes`, `lox.hours` and `lox.days` are `TimeDeltaUnit`
constants, and multiplying one by a number gives a `TimeDelta`. Durations
behave like the [physical quantities](units.md): they compare, sort, hash and
format, `float()` gives base SI seconds, and formatting gives the display unit
with its name.

```python
dt = 90 * lox.minutes

float(dt)              # 5400.0   <- seconds
f"{dt:.1f}"            # '5400.0 seconds'
f"{dt:.1f minutes}"    # '90.0 minutes'
dt / lox.hours         # 1.5
dt / 2                 # TimeDelta(2700)
dt > 1 * lox.hours     # True
```

A duration divided by another duration is a plain ratio, and multiplying two
durations is a `TypeError` — lox has no type for seconds squared:

```python
(2 * lox.hours) / (30 * lox.minutes)   # 4.0
```

`TimeDelta` keeps whole seconds and an attosecond remainder rather than one
float, so it is exact well past what `float(dt)` can show, and hashing keeps
attosecond-distinct durations distinct.

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

::: lox_space.TimeDeltaUnit
    options:
      show_source: false

---

::: lox_space.TimeScale
    options:
      show_source: false

---

::: lox_space.TimeSeries
    options:
      show_source: false
