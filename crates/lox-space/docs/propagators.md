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
| `J2` | Kozai J2 secular (± Kwok osculating) | Fast J2 oblateness, works for all orbits |
| `J4` | Kozai J4 secular (± Kwok osculating) | Higher-order J2²+J4 effects |
| `BrouwerLyddane` | Brouwer-Lyddane J2 propagator | Legacy; fails for circular orbits |
| `Numerical` | Numerical orbit propagator (J2 perturbation) | High-fidelity oblateness |
| `TLE` | Two-Line Element set parser | Satellite catalog data |
| `SGP4` | Simplified General Perturbations | TLE-based propagation |
| `GroundPropagator` | Ground station state | Earth-fixed locations |

## Quick Example

```python
import lox_space as lox

# Analytical (Kepler) propagation
t = lox.Time("TAI", 2024, 1, 1)
state = lox.Cartesian(
    t,
    position=(6678.0 * lox.km, 0.0 * lox.km, 0.0 * lox.km),
    velocity=(0.0 * lox.km_per_s, 7.73 * lox.km_per_s, 0.0 * lox.km_per_s),
)

propagator = lox.Vallado(state)

# Propagate to a single time
future = propagator.propagate(t + lox.TimeDelta.from_hours(1))

# Propagate to multiple times
times = [t + lox.TimeDelta(i * 60) for i in range(100)]
trajectory = propagator.propagate(times)

# J2 secular propagation (Kozai theory, works for all orbits)
j2 = lox.J2(state)
future = j2.propagate(t + lox.TimeDelta.from_hours(1))

# J2 with Kwok short-period corrections (osculating output)
j2_osc = lox.J2(state, osculating=True)

# J4 propagation (includes J2², J4 zonal harmonic terms)
j4 = lox.J4(state, osculating=True)

# Numerical propagation (accounts for J2 oblateness)
numerical = lox.Numerical(state)

# Propagate over a time interval (adaptive steps)
t_end = t + lox.TimeDelta.from_hours(2)
trajectory = numerical.propagate(t, end=t_end)

# Custom solver tolerances
numerical_tight = lox.Numerical(state, rtol=1e-12, atol=1e-10)

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

::: lox_space.J2
    options:
      show_source: false

---

::: lox_space.J4
    options:
      show_source: false

---

::: lox_space.BrouwerLyddane
    options:
      show_source: false

---

::: lox_space.Numerical
    options:
      show_source: false

---

::: lox_space.TLE
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
