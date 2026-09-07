<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Ground Stations

Ground-based tracking and observation support.

## Quick Example

```python
import lox_space as lox

# Define a ground station
gs = lox.EllipsoidLocation(
    frame="IAU_EARTH",          # any body-fixed frame, e.g. "ITRF"
    longitude=0.0 * lox.rad,    # Greenwich
    latitude=51.5 * lox.deg,    # ~51.5° N
    altitude=0.0 * lox.km,
)

# The frame's conventional ellipsoid is a default, not a constraint
gs_wgs84 = lox.EllipsoidLocation(
    frame="IAU_EARTH",
    longitude=0.0 * lox.rad,
    latitude=51.5 * lox.deg,
    altitude=0.0 * lox.km,
    ellipsoid=lox.Ellipsoid.WGS84,
)

# Calculate observables for a spacecraft state
obs = gs.observables(state)
print(f"Azimuth: {obs.azimuth().to_degrees():.2f} deg")
print(f"Elevation: {obs.elevation().to_degrees():.2f} deg")
print(f"Range: {obs.range().to_kilometers():.1f} km")

# Set an operational minimum elevation on a ground station
station = lox.GroundStation("ESOC", gs, min_elevation=5 * lox.deg)

# Or add a measured horizon profile; visibility uses the maximum of the
# horizon and the minimum elevation at each azimuth
import numpy as np
azimuth = np.linspace(-np.pi, np.pi, 36)
elevation = np.full(36, 0.1)  # radians
mask = lox.HorizonMask(azimuth, elevation)
station = lox.GroundStation("ESOC", gs, min_elevation=5 * lox.deg, horizon_mask=mask)
```

---

::: lox_space.EllipsoidLocation
    options:
      show_source: false

---

::: lox_space.Ellipsoid
    options:
      show_source: false

---

::: lox_space.HorizonMask
    options:
      show_source: false

---

::: lox_space.Observables
    options:
      show_source: false

---

::: lox_space.Pass
    options:
      show_source: false
