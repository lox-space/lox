<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Lox Python API Reference

Welcome to the Python API reference for **lox** - an ergonomic astrodynamics library.

```python
import lox_space as lox
```

## Overview

Lox provides tools for space mission analysis and orbital mechanics:

- **High-precision time** handling with multiple astronomical time scales
- **Orbital states** in Cartesian and Keplerian representations
- **Propagators** for orbit prediction (analytical and SGP4)
- **Ground station** analysis and visibility calculations
- **RF link budgets** for space communication systems
- **Coordinate transformations** between reference frames

## Quick Start

```python
import lox_space as lox

# Create a time instant
t = lox.Time("TAI", 2024, 1, 1, 12, 0, 0.0)

# Define an orbital state (LEO)
state = lox.Cartesian(
    time=t,
    x=6678.0 * lox.km, y=0.0 * lox.km, z=0.0 * lox.km,
    vx=0.0 * lox.km_per_s, vy=7.73 * lox.km_per_s, vz=0.0 * lox.km_per_s,
)

# Convert to Keplerian elements
kep = state.to_keplerian()
print(f"Semi-major axis: {kep.semi_major_axis().to_kilometers():.1f} km")
print(f"Orbital period: {kep.orbital_period()}")

# Propagate the orbit
propagator = lox.Vallado(state)
future_state = propagator.propagate(t + lox.TimeDelta.from_hours(1.5))
```

## API Sections

| Section | Description |
|---------|-------------|
| [Time & Dates](time.md) | `Time`, `UTC`, `TimeDelta`, `TimeScale` |
| [Celestial Bodies](bodies.md) | `Origin` |
| [Reference Frames](frames.md) | `Frame`, `SPK` |
| [Orbital States](states.md) | `Cartesian`, `Keplerian`, `Trajectory` |
| [Propagators](propagators.md) | `Vallado`, `SGP4`, `TLE`, `GroundPropagator` |
| [Ground Stations](ground.md) | `GroundLocation`, `ElevationMask`, `Observables`, `Pass` |
| [Events & Visibility](events.md) | `Event`, `Interval`, `find_events`, `find_windows`, `intersect_intervals`, `union_intervals`, `complement_intervals`, `GroundStation`, `Spacecraft`, `Scenario`, `Ensemble`, `VisibilityAnalysis`, `VisibilityResults` |
| [Communications](comms.md) | `Decibel`, `Transmitter`, `CommunicationSystem`, `Channel`, `LinkStats`, `fspl`, `freq_overlap` |
| [Data Providers](data.md) | `EOPProvider`, `Series` |
| [Units](units.md) | `Angle`, `Distance`, `Frequency`, `Velocity` |

## Related Resources

- [Main Documentation](https://docs.lox.rs) - Tutorials and guides
- [GitHub Repository](https://github.com/lox-space/lox) - Source code
