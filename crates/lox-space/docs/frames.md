<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Reference Frames

Coordinate reference frames and ephemeris data.

## Supported Frames

| Abbreviation | Name | Description |
|--------------|------|-------------|
| ICRF | International Celestial Reference Frame | Inertial frame |
| ECLIPJ2000 | Ecliptic J2000 | Ecliptic plane at J2000 |
| IAU_EARTH | IAU Earth | Earth body-fixed frame |
| ITRF | International Terrestrial Reference Frame | Earth-fixed |

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
