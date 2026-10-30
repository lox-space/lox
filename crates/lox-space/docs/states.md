<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Orbital States

Representations of orbital states in Cartesian, Keplerian, and equinoctial forms.

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
    x=6678.0 * lox.km,
    y=0.0 * lox.km,
    z=0.0 * lox.km,
    vx=0.0 * lox.km_per_s,
    vy=7.73 * lox.km_per_s,
    vz=0.0 * lox.km_per_s,
)

# Access as numpy arrays (meters, m/s)
pos = state.position()  # shape (3,)
vel = state.velocity()  # shape (3,)

# Access as unitful scalars
print(f"x = {state.x.to_kilometers():.1f} km")

# Convert to Keplerian elements (raises ValueError for states with zero angular momentum)
kep = state.to_keplerian()
# semi_major_axis() returns None for parabolic orbits
print(f"a = {kep.semi_major_axis().to_kilometers():.1f} km")
print(f"e = {kep.eccentricity():.6f}")
print(f"i = {kep.inclination().to_degrees():.2f} deg")

# Create from Keplerian elements (semi-major axis + eccentricity)
kep = lox.Keplerian(
    time=t,
    semi_major_axis=6678.0 * lox.km,
    eccentricity=0.001,
    inclination=51.6 * lox.deg,
)
state = kep.to_cartesian()

# Create from the semi-parameter, which also supports parabolic orbits
kep = lox.Keplerian(
    time=t,
    semi_parameter=14000.0 * lox.km,
    eccentricity=1.0,
)

# Create from periapsis/apoapsis radii
kep = lox.Keplerian(
    time=t,
    periapsis_radius=7000.0 * lox.km,
    apoapsis_radius=7400.0 * lox.km,
)

# Create from periapsis/apoapsis altitudes
kep = lox.Keplerian(
    time=t,
    periapsis_altitude=600.0 * lox.km,
    apoapsis_altitude=1000.0 * lox.km,
)

# Derived quantities
print(f"hp = {kep.periapsis_altitude().to_kilometers():.1f} km")
print(f"ha = {kep.apoapsis_altitude().to_kilometers():.1f} km")
print(f"n = {float(kep.mean_motion()):.3e} rad/s")
print(f"energy = {kep.specific_orbital_energy():.3f} km²/s²")

# Create a circular orbit from altitude
circ = lox.Keplerian.circular(
    t,
    altitude=800 * lox.km,
    inclination=51.6 * lox.deg,
)

# Create a Sun-synchronous orbit from altitude
eop = lox.EOPProvider("finals.csv")
sso = lox.Keplerian.sso(
    t,
    altitude=800 * lox.km,
    ltan=(13, 30),
    provider=eop,
)

# Equinoctial elements are non-singular for circular and equatorial orbits.
# Retrograde orbits use the retrograde set (retrograde factor I = -1) automatically.
eq = state.to_equinoctial()  # closed orbits only, raises ValueError otherwise
print(f"λ = {eq.mean_longitude().to_degrees():.3f} deg, retrograde = {eq.retrograde()}")

# Modified equinoctial elements also cover parabolic and hyperbolic orbits
mee = state.to_modified_equinoctial()
state = mee.to_equinoctial().to_cartesian()

# Work with trajectories
trajectory = lox.Trajectory([state1, state2, state3])
interpolated = trajectory.at(t + lox.TimeDelta(100))

# Queries up to 1 ms before the first or after the last state are
# extrapolated, queries beyond raise a ValueError. Widen the limit on demand:
extended = trajectory.with_extrapolation_limit(lox.TimeDelta(10))

# Use step_by_inclusive to sample an interval including its end
times = lox.Interval(t, t + lox.TimeDelta(150)).step_by_inclusive(lox.TimeDelta(60))
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

::: lox_space.Equinoctial
    options:
      show_source: false

---

::: lox_space.ModifiedEquinoctial
    options:
      show_source: false

---

::: lox_space.Trajectory
    options:
      show_source: false
