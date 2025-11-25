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
    longitude=0.0,          # radians (Greenwich)
    latitude=0.9,           # radians (~51.5° N)
    altitude=0.0,           # km
)

# Calculate observables for a spacecraft state
obs = gs.observables(state)
print(f"Azimuth: {obs.azimuth():.2f} rad")
print(f"Elevation: {obs.elevation():.2f} rad")
print(f"Range: {obs.range():.1f} km")

# Define elevation mask
mask = lox.ElevationMask.fixed(0.1)  # ~5.7° minimum elevation

# Or variable mask based on azimuth
import numpy as np
azimuth = np.linspace(0, 2*np.pi, 36)
elevation = np.full(36, 0.1)  # same minimum everywhere
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
