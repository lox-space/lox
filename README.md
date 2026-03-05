<!--
SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>

SPDX-License-Identifier: MPL-2.0
-->

# Lox – Oxidized Astrodynamics

[![codecov][codecov-badge]][codecov-url]
[![Rust][rust-badge]][rust-url]
[![Python][python-badge]][python-url]
[![CodSpeed][codspeed-badge]][codspeed-url]

Lox is an MPLv2-licensed Rust astrodynamics library with first-class Python bindings for
orbital mechanics, mission analysis, and telecommunications.

## Python Quick Start

```python
import lox_space as lox

# Parse a UTC epoch and convert to the TDB time scale
epoch = lox.UTC.from_iso("2025-01-01T12:00:00").to_scale("TDB")

# Load Earth orientation parameters
provider = lox.EOPProvider("finals2000A.all.csv")

# Design a sun-synchronous orbit at 800 km altitude with a 10:30 LTAN
sso = lox.Keplerian.sso(
    epoch, altitude=800 * lox.km, ltan=(10, 30), provider=provider
)

# Convert to Cartesian state and propagate with J2 perturbations
state = sso.to_cartesian()
j2 = lox.J2(state)
trajectory = j2.propagate(epoch, end=epoch + 100 * lox.minutes)
```

## Rust Quick Start

```rust
use lox_space::prelude::*;

let epoch = Utc::from_iso("2025-01-01T12:00:00").unwrap().to_time().to_scale(Tdb);
let provider = EopParser::new().from_path("finals2000A.all.csv").parse().unwrap();

let sso = SsoBuilder::default()
    .with_provider(&provider)
    .with_time(epoch)
    .with_altitude(800.0.km())
    .with_ltan(10, 30)
    .build()
    .unwrap();

// Convert to Cartesian state and propagate with J2 perturbations
let state = sso.to_cartesian();
let j2 = J2Propagator::new(state);
let end = epoch + TimeDelta::from_minutes(100);
let trajectory = j2.propagate(Interval::new(epoch, end)).unwrap();
```

## Installation

### Python

```sh
uv add lox-space
# or
pip install lox-space
```

### Rust

```sh
cargo add lox-space
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
lox-space = "0.1"
```

## Features

- **Orbital Mechanics** — Keplerian elements, state vectors, SSO design, Vallado/J2/SGP4 propagation, TLE parsing
- **Time Systems** — TAI, TT, TDB, TCB, TCG, UTC, UT1; femtosecond precision, leap-second aware
- **Reference Frames** — ICRF, ITRF, TEME; CIO and equinox-based transformation chains
- **Ground Stations** — Visibility windows, elevation masks, pass prediction
- **Constellation Design** — Walker Delta/Star, Street-of-Coverage, Flower
- **RF Link Budgets** — Antenna patterns, modulation schemes, path loss
- **Python Bindings** — Full API with type stubs and NumPy interop

## Status

Lox is pre-1.0. The API may change between releases.

## Documentation

- Python: https://python.lox.rs
- Rust: https://docs.rs/lox-space

## Why "Lox"?

> Liquid oxygen—abbreviated LOx, LOX or Lox in the aerospace, submarine and gas industries—is the liquid form of
> molecular oxygen. It was used as the _oxidizer_ in the first liquid-fueled rocket invented in 1926 by Robert H.
> Goddard,
> an application which has continued to the present. [Wikipedia](https://en.wikipedia.org/wiki/Liquid_oxygen)

[codecov-badge]: https://codecov.io/gh/lox-space/lox/graph/badge.svg?token=R1W6HLN2N2
[codecov-url]: https://codecov.io/gh/lox-space/lox
[rust-badge]: https://github.com/lox-space/lox/actions/workflows/rust.yml/badge.svg
[rust-url]: https://github.com/lox-space/lox/actions/workflows/rust.yml
[python-badge]: https://github.com/lox-space/lox/actions/workflows/python.yml/badge.svg
[python-url]: https://github.com/lox-space/lox/actions/workflows/python.yml
[codspeed-badge]: https://img.shields.io/endpoint?url=https://codspeed.io/badge.json
[codspeed-url]: https://codspeed.io/lox-space/lox
