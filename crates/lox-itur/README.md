<!--
SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MIT AND MPL-2.0
-->

# lox-itur

ITU-R P-series atmospheric propagation models for the [Lox](https://github.com/lox-space/lox)
ecosystem. A Rust port of the Python [ITU-Rpy](https://github.com/iportillo/ITU-Rpy) library.

Computes atmospheric attenuation on Earth-to-space and terrestrial radio paths
from rain, gaseous absorption, clouds, and scintillation, following international
standards published by the ITU Radiocommunication Sector.

## Implemented Recommendations

| Recommendation | Description |
|----------------|-------------|
| P.453-13 | Radio refractive index |
| P.618-14 | Earth-space propagation (rain, scintillation, XPD) |
| P.676-13 | Attenuation by atmospheric gases (O₂ + H₂O) |
| P.835-6 | Reference standard atmospheres |
| P.836-6 | Water vapour surface density and columnar content |
| P.837-7 | Characteristics of precipitation |
| P.838-3 | Specific attenuation model for rain |
| P.839-4 | Rain height model |
| P.840-9 | Attenuation due to clouds and fog |
| P.1510-1 | Annual mean surface temperature |
| P.1511-2 | Topography for Earth-to-space propagation modelling |

## Usage

```rust
use lox_core::units::{Angle, Distance, Frequency};
use lox_itur::atmospheric_attenuation_slant_path;

let losses = atmospheric_attenuation_slant_path(
    Angle::degrees(40.4),       // latitude (Madrid)
    Angle::degrees(-3.7),       // longitude
    Frequency::gigahertz(14.25), // Ku-band
    Angle::degrees(30.0),       // elevation
    0.01,                        // exceeded 0.01% of the year
    Distance::meters(1.2),      // antenna diameter
    Angle::degrees(45.0),       // circular polarisation
);
```

The returned `EnvironmentalLosses` struct contains individual contributions
(rain, gaseous, cloud, scintillation, depolarization) and the combined total,
ready to plug into `lox-comms` link budget calculations.

## Data

Grid-based models require reference data from the ITU. The build script
automatically downloads the [itur](https://pypi.org/project/itur/) Python
package from PyPI and converts the data during `cargo build`. No manual
setup is needed.

The data directory can be overridden with the `LOX_ITUR_DATA` environment
variable.

## License

This crate is dual-licensed under MIT (from the original ITU-Rpy library,
Copyright 2016 Inigo del Portillo, Massachusetts Institute of Technology)
and MPL-2.0.
