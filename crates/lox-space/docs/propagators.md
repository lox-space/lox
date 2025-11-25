<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Propagators

Orbit propagation methods for predicting future states.

## Available Propagators

| Propagator | Description | Use Case |
|------------|-------------|----------|
| `Vallado` | Analytical Kepler propagator | Two-body motion |
| `SGP4` | Simplified General Perturbations | TLE-based propagation |
| `GroundPropagator` | Ground station state | Earth-fixed locations |

## Quick Example

```python
import lox_space as lox

# Analytical (Kepler) propagation
t = lox.Time("TAI", 2024, 1, 1)
state = lox.State(t, (6678.0, 0.0, 0.0), (0.0, 7.73, 0.0))

propagator = lox.Vallado(state)

# Propagate to a single time
future = propagator.propagate(t + lox.TimeDelta.from_hours(1))

# Propagate to multiple times
times = [t + lox.TimeDelta(i * 60) for i in range(100)]
trajectory = propagator.propagate(times)

# SGP4 propagation from TLE
tle = """ISS (ZARYA)
1 25544U 98067A   24001.50000000  .00016717  00000-0  30472-3 0  9993
2 25544  51.6400  10.3600 0005000  50.0000 310.0000 15.50000000000010"""

sgp4 = lox.SGP4(tle)
state = sgp4.propagate(t)
```

---

::: lox_space.Vallado
    options:
      show_source: false

---

::: lox_space.SGP4
    options:
      show_source: false

---

::: lox_space.GroundPropagator
    options:
      show_source: false
