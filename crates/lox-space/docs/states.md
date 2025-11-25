<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Orbital States

Representations of orbital states in Cartesian and Keplerian forms.

## Quick Example

```python
import lox_space as lox

# Create a Cartesian state
t = lox.Time("TAI", 2024, 1, 1)
state = lox.State(
    time=t,
    position=(6678.0, 0.0, 0.0),  # km
    velocity=(0.0, 7.73, 0.0),    # km/s
)

# Convert to Keplerian elements
kep = state.to_keplerian()
print(f"a = {kep.semi_major_axis():.1f} km")
print(f"e = {kep.eccentricity():.6f}")
print(f"i = {kep.inclination():.4f} rad")

# Create from Keplerian elements
kep = lox.Keplerian(
    time=t,
    semi_major_axis=6678.0,
    eccentricity=0.001,
    inclination=0.5,
    longitude_of_ascending_node=0.0,
    argument_of_periapsis=0.0,
    true_anomaly=0.0,
)
state = kep.to_cartesian()

# Work with trajectories
trajectory = lox.Trajectory([state1, state2, state3])
interpolated = trajectory.interpolate(t + lox.TimeDelta(100))
```

---

::: lox_space.State
    options:
      show_source: false

---

::: lox_space.Keplerian
    options:
      show_source: false

---

::: lox_space.Trajectory
    options:
      show_source: false

---

::: lox_space.Ensemble
    options:
      show_source: false
