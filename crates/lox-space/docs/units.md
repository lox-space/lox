<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Units

Physical quantity types for type-safe unit handling.

A **quantity** is a value with a dimension — a `Distance`, an `Angle`, a `Frequency`.
A **unit** is what you multiply a number by to get one: `lox.km` is a `DistanceUnit`,
and `100 * lox.km` is a `Distance`.

```python
import lox_space as lox

slant_range = 909.42494 * lox.km

f"{slant_range:.1f}"        # '909.4 km'
slant_range > 500 * lox.km  # True
slant_range / lox.m         # 909424.94
```

## Available units

### Angle — `AngleUnit`

| Constant | Value |
|----------|-------|
| `rad` | 1 radian |
| `deg` | π/180 radians |

### AngularRate — `AngularRateUnit`

| Constant | Value |
|----------|-------|
| `rad_per_s` | 1 rad/s |
| `deg_per_s` | π/180 rad/s |

### Distance — `DistanceUnit`

| Constant | Value |
|----------|-------|
| `m` | 1 meter |
| `km` | 1000 meters |
| `au` | 1 astronomical unit |

### Frequency — `FrequencyUnit`

| Constant | Value |
|----------|-------|
| `Hz` | 1 Hz |
| `kHz` | 1 kHz |
| `MHz` | 1 MHz |
| `GHz` | 1 GHz |
| `THz` | 1 THz |

### Power — `PowerUnit`

| Constant | Value |
|----------|-------|
| `W` | 1 W |
| `kW` | 1 kW |

### Pressure — `PressureUnit`

| Constant | Value |
|----------|-------|
| `Pa` | 1 pascal |
| `hPa` | 1 hectopascal |

### Temperature — `TemperatureUnit`

| Constant | Value |
|----------|-------|
| `K` | 1 Kelvin |

### Velocity — `VelocityUnit`

| Constant | Value |
|----------|-------|
| `m_per_s` | 1 m/s |
| `km_per_s` | 1 km/s |

### Decibel — `DecibelUnit`

| Constant | Value |
|----------|-------|
| `dB` | 1 dB |

Durations use the same machinery but live with the time API: `lox.seconds`,
`lox.minutes`, `lox.hours` and `lox.days` are `TimeDeltaUnit` constants, and
`TimeDelta` compares, hashes, divides and formats just like the quantities
below. See [Time & Dates](time.md#durations).

## Creating quantities

Multiply a number by a unit constant, or use a `from_*` constructor — the two are
equivalent, and the constructors are the ones your editor will autocomplete:

```python
distance = 100 * lox.km
distance = lox.Distance.from_kilometers(100)

frequency = 8.2 * lox.GHz
frequency = lox.Frequency.from_gigahertz(8.2)
```

Every `to_*` accessor has a matching `from_*` constructor.

## Arithmetic

Quantities add, subtract, negate, and scale by a number:

```python
distance = 100 * lox.km

total = distance + 500 * lox.m       # Distance
delta = 45 * lox.deg - 10 * lox.deg  # Angle
scaled = 2.0 * distance              # Distance
```

Division does one of two things, depending on what you divide by:

```python
half = (500 * lox.km) / 2               # Distance
ratio = (500 * lox.km) / (100 * lox.km) # 5.0, a plain float
```

Quantities compare, sort and hash:

```python
margin = 26.63 * lox.dB

assert margin > 3 * lox.dB
nearest = min(500 * lox.km, 100 * lox.km)
bands = {8.2 * lox.GHz: "X-band"}
```

### Dimensional safety

Scalar × quantity is a quantity. Anything else is a `TypeError`:

```python
2 * lox.km              # DistanceUnit -> Distance
(1 * lox.km) * 2        # Distance
(1 * lox.km) * (1 * lox.km)   # TypeError — lox has no area type
(1 * lox.km) + (1 * lox.rad)  # TypeError
lox.km * lox.m                # TypeError
```

The constructors are just as strict, so a quantity cannot slip in where a plain
number belongs:

```python
lox.Distance(500.0)                 # Distance
lox.Distance(lox.Angle(1.0))        # TypeError
lox.Distance.from_kilometers(500)   # Distance
```

There is no `int()` conversion: silently truncating to whole metres or whole hertz is
rarely what anyone means. Use `round()`, which returns a quantity, or `float()`.

## Displaying quantities

`float()` returns the **base SI** value; formatting returns the **display unit** with
its suffix. This is the one thing to keep straight:

```python
d = 909.42494 * lox.km

float(d)        # 909424.94   <- metres
f"{d}"          # '909.42494 km'
f"{d:.1f}"      # '909.4 km'
f"{d:>12.1f}"   # '    909.4 km'   (the whole string is padded, so columns line up)
```

Base units per type: radians, rad/s, metres, hertz, watts, pascals, Kelvin, m/s, dB,
m³/s².

To render in a different unit, name it at the end of the format spec, or divide by it
to get a plain number:

```python
d = 909.42494 * lox.km

f"{d:.1f m}"    # '909424.9 m'
f"{d:.3f au}"   # '0.000 au'
d / lox.m       # 909424.94
d.to_meters()   # 909424.94
```

`repr()` always shows the exact base-SI value and round-trips through `eval`; `str()`
renders at 15 significant digits, which keeps unit scaling from leaking floating-point
artefacts into reports:

```python
str(29 * lox.GHz)    # '29 GHz'
repr(29 * lox.GHz)   # 'Frequency(29000000000.0)'
```

## NumPy interoperability

Quantities are **scalars** — there is no array-valued quantity type. Converting to
NumPy gives float64 in the **base SI unit**, and units do not survive the crossing:

```python
import numpy as np

np.array([500 * lox.km, 100 * lox.km])   # array([500000., 100000.])  metres
np.asarray(500 * lox.km).item()          # 500000.0
```

Scalar arithmetic keeps the type in both directions, including with NumPy scalars:

```python
np.float64(2) * (500 * lox.km)   # Distance
```

Array × quantity raises `TypeError` rather than silently dropping the unit. For
vectorised work, convert first and say the unit in the name:

```python
elevations_deg = np.arange(5.0, 90.0, 5.0)
ranges_km = np.array([(el * lox.deg).to_degrees() for el in elevations_deg])
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

::: lox_space.Decibel
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

::: lox_space.Pressure
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

---

## Unit classes

You rarely name these directly — the module-level constants above are instances of
them — but they are what a unit constant *is*, and they can be looked up by name:

```python
lox.DistanceUnit("km") == lox.km        # True
lox.VelocityUnit("km/s") == lox.km_per_s  # the display suffix works too
```

::: lox_space.AngleUnit
    options:
      show_source: false

---

::: lox_space.AngularRateUnit
    options:
      show_source: false

---

::: lox_space.DecibelUnit
    options:
      show_source: false

---

::: lox_space.DistanceUnit
    options:
      show_source: false

---

::: lox_space.FrequencyUnit
    options:
      show_source: false

---

::: lox_space.PowerUnit
    options:
      show_source: false

---

::: lox_space.PressureUnit
    options:
      show_source: false

---

::: lox_space.TemperatureUnit
    options:
      show_source: false

---

::: lox_space.VelocityUnit
    options:
      show_source: false
