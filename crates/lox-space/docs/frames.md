<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Reference Frames

Coordinate reference frames and ephemeris data.

## Supported Frames

### Inertial Frames

| Abbreviation | Name | Description |
|--------------|------|-------------|
| ICRF | International Celestial Reference Frame | Primary inertial frame |
| J2000 | J2000 Mean Equator and Equinox | FK5-compatible inertial frame (alias: EME2000) |

### CIO-Based Frames (IAU2006/IERS2010)

| Abbreviation | Name | Description |
|--------------|------|-------------|
| CIRF | Celestial Intermediate Reference Frame | CIO-based celestial frame |
| TIRF | Terrestrial Intermediate Reference Frame | CIO-based terrestrial frame |
| ITRF | International Terrestrial Reference Frame | Earth-fixed |

### Equinox-Based Frames

| Abbreviation | Name | Description |
|--------------|------|-------------|
| MOD | Mean of Date | Precessed frame (default: IERS1996) |
| TOD | True of Date | Precessed + nutated frame (default: IERS1996) |
| PEF | Pseudo-Earth Fixed | Includes Earth rotation (default: IERS1996) |
| TEME | True Equator Mean Equinox | Used by SGP4/TLE |

Equinox-based frames accept an IERS convention suffix, e.g. `MOD(IERS2003)`, `TOD(IERS2010)`.
The bare forms (`MOD`, `TOD`, `PEF`) default to IERS1996.

### Body-Fixed Frames

| Abbreviation | Name | Description |
|--------------|------|-------------|
| IAU_EARTH | IAU Earth | Earth body-fixed frame |
| IAU_MOON | IAU Moon | Moon body-fixed frame |
| IAU_*BODY* | IAU body-fixed | Available for all bodies with defined rotational elements |

## Quick Example

```python
import lox_space as lox

# Create a frame
icrf = lox.Frame("ICRF")
itrf = lox.Frame("ITRF")

# Load ephemeris data
spk = lox.SPK("/path/to/de440.bsp")

# Transform states between frames
state_icrf = state.to_frame(itrf, provider=eop_provider)

# Transform between origins
state_moon = state.to_origin(lox.Origin("Moon"), spk)
```

---

::: lox_space.Frame
    options:
      show_source: false

---

::: lox_space.SPK
    options:
      show_source: false
