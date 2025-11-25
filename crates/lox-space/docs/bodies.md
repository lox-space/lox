<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Celestial Bodies

Celestial body definitions including physical properties and rotational elements.

## Supported Bodies

Lox supports the standard NAIF/SPICE body identifiers for:

- Planets (Mercury through Neptune)
- Planetary barycenters
- Natural satellites (major moons)
- The Sun
- Solar system barycenter

## Quick Example

```python
import lox_space as lox

# Create by name or ID
earth = lox.Origin("Earth")
moon = lox.Origin("Moon")
mars = lox.Origin(499)  # NAIF ID

# Access properties
print(f"Earth radius: {earth.mean_radius():.1f} km")
print(f"Earth GM: {earth.gravitational_parameter():.6e} km³/s²")

# Rotational elements at a given epoch
et = 0.0  # Ephemeris time (seconds from J2000)
ra, dec, w = earth.rotational_elements(et)
```

---

::: lox_space.Origin
    options:
      show_source: false
