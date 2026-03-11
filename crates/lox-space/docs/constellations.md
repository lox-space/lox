<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Constellations

The `Constellation` class provides factory methods for designing satellite
constellations using several well-known algorithms.

## Walker Delta

A Walker Delta constellation distributes satellites evenly across orbital planes
with a RAAN spread of 360 degrees.

```python
import lox_space as lox

epoch = lox.Time("2025-01-01T00:00:00.000 TAI")

constellation = lox.Constellation.walker_delta(
    "iridium",
    epoch,
    lox.Origin("Earth"),
    nsats=66,
    nplanes=6,
    semi_major_axis=7159 * lox.km,
    inclination=53 * lox.deg,
    phasing=1,
)

print(len(constellation))  # 66
print(constellation.name)  # "iridium"
```

All constellation types accept an optional `longitude_of_ascending_node`
parameter to set the longitude of ascending node offset for the first
orbital plane. The remaining planes are spaced relative to this offset.

```python
constellation = lox.Constellation.walker_delta(
    "iridium_offset",
    epoch,
    lox.Origin("Earth"),
    nsats=66,
    nplanes=6,
    semi_major_axis=7159 * lox.km,
    inclination=53 * lox.deg,
    phasing=1,
    longitude_of_ascending_node=30 * lox.deg,
)
```

## Walker Star

Same as Walker Delta but with a RAAN spread of 180 degrees.

```python
constellation = lox.Constellation.walker_star(
    "polar",
    epoch,
    lox.Origin("Earth"),
    nsats=8,
    nplanes=4,
    semi_major_axis=7000 * lox.km,
    inclination=90 * lox.deg,
)
```

## Street-of-Coverage

Optimizes satellite placement for continuous coverage using the method of
Huang, Colombo, and Bernelli-Zazzera (2021).

```python
constellation = lox.Constellation.street_of_coverage(
    "coverage",
    epoch,
    lox.Origin("Earth"),
    nsats=24,
    nplanes=4,
    semi_major_axis=7159 * lox.km,
    inclination=53 * lox.deg,
    coverage_fold=1,
)
```

## Flower

Flower constellations produce repeating ground tracks. The orbital shape
can be computed from a perigee altitude (the mean radius, gravitational
parameter, and rotation rate are derived from the origin automatically)
or provided directly as semi-major axis and eccentricity.

```python
constellation = lox.Constellation.flower(
    "flower14",
    epoch,
    lox.Origin("Earth"),
    n_petals=14,
    n_days=1,
    nsats=28,
    phasing_numerator=1,
    phasing_denominator=28,
    inclination=53 * lox.deg,
    perigee_altitude=780 * lox.km,
)
```

## Using with Scenarios

Constellations can be added to a `Scenario` for visibility analysis:

```python
scenario = lox.Scenario(t0, t1).with_constellation(constellation)
```

Each satellite is converted to a `Spacecraft` using the propagator specified
at constellation creation time (default: `"vallado"`, also available: `"j2"`).

---

::: lox_space.Constellation
    options:
      show_source: false

---

::: lox_space.ConstellationSatellite
    options:
      show_source: false
