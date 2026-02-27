<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Orbital States

Representations of orbital states in Cartesian and Keplerian forms.

## Quick Example

```python
import lox_space as lox

t = lox.Time("TAI", 2024, 1, 1)

# Array pattern — position in meters, velocity in m/s
state = lox.Cartesian(
    time=t,
    position=[6678e3, 0.0, 0.0],
    velocity=[0.0, 7730.0, 0.0],
)

# Elementwise pattern — unitful keyword arguments
state = lox.Cartesian(
    time=t,
    x=6678.0 * lox.km, y=0.0 * lox.km, z=0.0 * lox.km,
    vx=0.0 * lox.km_per_s, vy=7.73 * lox.km_per_s, vz=0.0 * lox.km_per_s,
)

# Access as numpy arrays (meters, m/s)
pos = state.position()   # shape (3,)
vel = state.velocity()   # shape (3,)

# Access as unitful scalars
print(f"x = {state.x.to_kilometers():.1f} km")

# Convert to Keplerian elements
kep = state.to_keplerian()
print(f"a = {kep.semi_major_axis().to_kilometers():.1f} km")
print(f"e = {kep.eccentricity():.6f}")
print(f"i = {kep.inclination().to_degrees():.2f} deg")

# Create from Keplerian elements
kep = lox.Keplerian(
    time=t,
    semi_major_axis=6678.0 * lox.km,
    eccentricity=0.001,
    inclination=0.5 * lox.rad,
    longitude_of_ascending_node=0.0 * lox.rad,
    argument_of_periapsis=0.0 * lox.rad,
    true_anomaly=0.0 * lox.rad,
)
state = kep.to_cartesian()

# Create a Sun-synchronous orbit from altitude
eop = lox.EOPProvider("finals.csv")
sso = lox.Keplerian.sso(
    t,
    altitude=800 * lox.km,
    ltan=(13, 30),
    provider=eop,
)

# Work with trajectories
trajectory = lox.Trajectory([state1, state2, state3])
interpolated = trajectory.interpolate(t + lox.TimeDelta(100))
```

---

::: lox_space.Cartesian
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
