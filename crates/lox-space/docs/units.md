<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Units

Every dimensioned value in the Python API is a quantity object such as `Distance` or `Angle`.
Quantities store their value in SI units and never convert to a plain `float` implicitly, so a
distance cannot end up where an angle or a bare number is expected.

## Creating Quantities

Multiply a number by a unit constant, or use one of the static constructors named after the
units:

```python
import lox_space as lox

altitude = 500 * lox.km
inclination = 97.4 * lox.deg
carrier = 8.4 * lox.GHz
duration = 90 * lox.minutes

altitude = lox.Distance.kilometers(500)
```

Unit constants such as `lox.km` are unit objects (`DistanceUnit`), not quantities. A unit can
also be looked up by its symbol, e.g. `lox.DistanceUnit("km")`.

## Converting to Numbers

`to(unit)` returns the value as a `float` in the given unit. Each quantity also has `to_*`
methods named after its units:

```python
altitude.to(lox.km)  # 500.0
altitude.to_meters()  # 500000.0
duration.to(lox.seconds)  # 5400.0
```

## Formatting

`str()` shows the value in SI units with the unit symbol, and format specs apply to the number.
`display_in` formats the value in another unit:

```python
str(altitude)  # "500000 m"
f"{altitude:.1f}"  # "500000.0 m"
altitude.display_in(lox.km)  # "500.0 km"
altitude.display_in(lox.km, ".1f")  # "500.0 km"
```

## Arithmetic

Quantities of the same kind can be added, subtracted and compared, and scaled by numbers.
Dividing two quantities of the same kind gives a `float`. A few products and quotients change
the dimension:

| Expression | Result |
|------------|--------|
| `Distance / TimeDelta` | `Velocity` |
| `Velocity * TimeDelta` | `Distance` |
| `Angle / TimeDelta` | `AngularRate` |
| `AngularRate * TimeDelta` | `Angle` |
| `Distance * Distance` | `Area` |
| `Area / Mass` | `AreaToMass` |
| `DataRate * TimeDelta` | `DataVolume` |
| `DataVolume / TimeDelta` | `DataRate` |
| `DecibelWatts ± Decibel` | `DecibelWatts` |
| `DecibelWatts - DecibelWatts` | `Decibel` |

Products work with the operands in either order.

```python
total = 100 * lox.km + 500 * lox.m  # Distance
ratio = (2 * lox.km) / (500 * lox.m)  # 4.0
speed = (7 * lox.km) / (1 * lox.seconds)  # Velocity
eirp = 10 * lox.dBW + 30 * lox.dB  # DecibelWatts
```

## Unit Constants

### Angle

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.rad` | rad | radians |
| `lox.urad` | µrad | microradians |
| `lox.deg` | deg | degrees |
| `lox.arcsec` | arcsec | arcseconds |
| `lox.mas` | mas | milliarcseconds |
| `lox.uas` | µas | microarcseconds |

### AngularRate

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.rad_per_s` | rad/s | radians per second |
| `lox.deg_per_s` | deg/s | degrees per second |

### Area

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m2` | m² | square meters |
| `lox.km2` | km² | square kilometers |

### AreaToMass

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m2_per_kg` | m²/kg | square meters per kilogram |

### DataRate

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.bps` | bit/s | bits per second |
| `lox.kbps` | kbit/s | kilobits per second |
| `lox.Mbps` | Mbit/s | megabits per second |
| `lox.Gbps` | Gbit/s | gigabits per second |

### DataVolume

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.bit` | bit | bits |
| `lox.kbit` | kbit | kilobits |
| `lox.Mbit` | Mbit | megabits |
| `lox.Gbit` | Gbit | gigabits |
| `lox.Tbit` | Tbit | terabits |
| `lox.B` | B | bytes |
| `lox.kB` | kB | kilobytes (10³ bytes) |
| `lox.MB` | MB | megabytes (10⁶ bytes) |
| `lox.GB` | GB | gigabytes (10⁹ bytes) |
| `lox.TB` | TB | terabytes (10¹² bytes) |
| `lox.KiB` | KiB | kibibytes (2¹⁰ bytes) |
| `lox.MiB` | MiB | mebibytes (2²⁰ bytes) |
| `lox.GiB` | GiB | gibibytes (2³⁰ bytes) |
| `lox.TiB` | TiB | tebibytes (2⁴⁰ bytes) |

### Decibel

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.dB` | dB | decibels |

### DecibelWatts

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.dBW` | dBW | decibel watts |
| `lox.dBm` | dBm | decibel milliwatts |

### Distance

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m` | m | meters |
| `lox.km` | km | kilometers |
| `lox.au` | au | astronomical units |

### Frequency

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.Hz` | Hz | hertz |
| `lox.kHz` | kHz | kilohertz |
| `lox.MHz` | MHz | megahertz |
| `lox.GHz` | GHz | gigahertz |
| `lox.THz` | THz | terahertz |

### GravitationalParameter

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m3_per_s2` | m³/s² | cubic meters per second squared |
| `lox.km3_per_s2` | km³/s² | cubic kilometers per second squared |

### Mass

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.g` | g | grams |
| `lox.kg` | kg | kilograms |
| `lox.t` | t | metric tons |

### Power

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.W` | W | watts |
| `lox.kW` | kW | kilowatts |

### Pressure

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.Pa` | Pa | pascals |
| `lox.hPa` | hPa | hectopascals |

### SpecificAngularMomentum

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m2_per_s` | m²/s | square meters per second |
| `lox.km2_per_s` | km²/s | square kilometers per second |

### SpecificEnergy

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m2_per_s2` | m²/s² | square meters per second squared |
| `lox.km2_per_s2` | km²/s² | square kilometers per second squared |

### Temperature

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.K` | K | kelvin |

### Velocity

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.m_per_s` | m/s | meters per second |
| `lox.km_per_s` | km/s | kilometers per second |
| `lox.au_per_d` | au/d | astronomical units per day |
| `lox.c` | c | fraction of the speed of light |

### TimeDelta

| Constant | Symbol | Unit |
|----------|--------|------|
| `lox.attoseconds` | as | attoseconds |
| `lox.femtoseconds` | fs | femtoseconds |
| `lox.picoseconds` | ps | picoseconds |
| `lox.nanoseconds` | ns | nanoseconds |
| `lox.microseconds` | µs | microseconds |
| `lox.milliseconds` | ms | milliseconds |
| `lox.seconds` | s | seconds |
| `lox.minutes` | min | minutes |
| `lox.hours` | h | hours |
| `lox.days` | d | days of 86400 s |
| `lox.julian_years` | a | Julian years of 365.25 days |
| `lox.julian_centuries` | cy | Julian centuries of 36525 days |

---

::: lox_space.Angle
    options:
      show_source: false

::: lox_space.AngleUnit
    options:
      show_source: false

---

::: lox_space.AngularRate
    options:
      show_source: false

::: lox_space.AngularRateUnit
    options:
      show_source: false

---

::: lox_space.Area
    options:
      show_source: false

::: lox_space.AreaUnit
    options:
      show_source: false

---

::: lox_space.AreaToMass
    options:
      show_source: false

::: lox_space.AreaToMassUnit
    options:
      show_source: false

---

::: lox_space.DataRate
    options:
      show_source: false

::: lox_space.DataRateUnit
    options:
      show_source: false

---

::: lox_space.DataVolume
    options:
      show_source: false

::: lox_space.DataVolumeUnit
    options:
      show_source: false

---

::: lox_space.Decibel
    options:
      show_source: false

::: lox_space.DecibelUnit
    options:
      show_source: false

---

::: lox_space.DecibelWatts
    options:
      show_source: false

::: lox_space.DecibelWattsUnit
    options:
      show_source: false

---

::: lox_space.Distance
    options:
      show_source: false

::: lox_space.DistanceUnit
    options:
      show_source: false

---

::: lox_space.Frequency
    options:
      show_source: false

::: lox_space.FrequencyUnit
    options:
      show_source: false

---

::: lox_space.GravitationalParameter
    options:
      show_source: false

::: lox_space.GravitationalParameterUnit
    options:
      show_source: false

---

::: lox_space.Mass
    options:
      show_source: false

::: lox_space.MassUnit
    options:
      show_source: false

---

::: lox_space.Power
    options:
      show_source: false

::: lox_space.PowerUnit
    options:
      show_source: false

---

::: lox_space.Pressure
    options:
      show_source: false

::: lox_space.PressureUnit
    options:
      show_source: false

---

::: lox_space.SpecificAngularMomentum
    options:
      show_source: false

::: lox_space.SpecificAngularMomentumUnit
    options:
      show_source: false

---

::: lox_space.SpecificEnergy
    options:
      show_source: false

::: lox_space.SpecificEnergyUnit
    options:
      show_source: false

---

::: lox_space.Temperature
    options:
      show_source: false

::: lox_space.TemperatureUnit
    options:
      show_source: false

---

::: lox_space.Velocity
    options:
      show_source: false

::: lox_space.VelocityUnit
    options:
      show_source: false
