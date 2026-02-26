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
gs = lox.GroundLocation(
    origin=lox.Origin("Earth"),
    longitude=0.0 * lox.rad,    # Greenwich
    latitude=51.5 * lox.deg,    # ~51.5° N
    altitude=0.0 * lox.km,
)

# Calculate observables for a spacecraft state
obs = gs.observables(state)
print(f"Azimuth: {obs.azimuth().to_degrees():.2f} deg")
print(f"Elevation: {obs.elevation().to_degrees():.2f} deg")
print(f"Range: {obs.range().to_kilometers():.1f} km")

# Define elevation mask
mask = lox.ElevationMask.fixed(5 * lox.deg)

# Or variable mask based on azimuth
import numpy as np
azimuth = np.linspace(0, 2*np.pi, 36)
elevation = np.full(36, 0.1)  # radians
mask = lox.ElevationMask.variable(azimuth, elevation)
```

---

::: lox_space.GroundLocation
    options:
      show_source: false

---

::: lox_space.ElevationMask
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
