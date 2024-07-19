# Lox – Oxidized Astrodynamics

### A safe, ergonomic astrodynamics library for the modern space industry

[![codecov](https://codecov.io/gh/lox-space/lox/graph/badge.svg?token=R1W6HLN2N2)](https://codecov.io/gh/lox-space/lox) ![Rust](https://github.com/lox-space/lox/actions/workflows/rust.yml/badge.svg) ![Python](https://github.com/lox-space/lox/actions/workflows/python.yml/badge.svg)

![A star chart of a crab constellation](public/crabstellation.webp)


> **Note:** Lox is under active development and does not yet have a stable release. The API of all crates is subject to
> significant change.

## Features

Lox exposes a comprehensive astrodynamics API at varying levels of granularity. The high-level interface offered
by `lox-space` is designed specifically for mission planning and analysis, while crates like `lox-time`, `lox-earth`
and `lox-orbits` provide tools for advanced users.

* A fully featured space mission simulator backend.
* Python bindings for interactive use.
* Tools for working with time in astronomical and terrestrial time scales.
* Define orbits as Keplerian elements or state vectors in different coordinate frames.
* Ephemeris, size and shape data for all major celestial bodies.
* Ingest and interpolate Earth orientation parameters with ease.
* Extensible – bring your own time scales, transformation algorithms, data sources and more.

## Crates

### lox-space

The entrypoint to the Lox ecosystem, suitable for most use cases. Provides a high-level interface for mission planning
and analysis. Also includes Lox's Python bindings.

### lox-time

Tools for working with time in all commonly-used astronomical time scales based on a high-precision timestamp
representation. Offers leap-second aware conversion from UTC to continuous time scales.

### lox-bodies

Provides structs representing all major celestial bodies, conveniently categorized by a variety of traits exposing
SPICE-derived data.

### lox-earth

Essential algorithms for Earth-centric astrodynamics, including nutation-precession models, Earth rotation angle, CIP
and CIO locations, and coordinate transformations.

### lox-ephem

Parses ephemeris data from external sources such as SPICE kernels.

### lox-io

Utilities for reading and writing data in various formats.

### lox-math

A collection of mathematical utilities used across the Lox ecosystem.

## Used by...

### Ephemerista

[![The Ephemerista logo](public/ephemerista-logo.webp)][ephemerista]

A next-generation, open-source space mission simulator [commissioned by the European Space Agency][artes].

## Why "Lox"?

> Liquid oxygen—abbreviated LOx, LOX or Lox in the aerospace, submarine and gas industries—is the liquid form of
> molecular oxygen. It was used as the _oxidizer_ in the first liquid-fueled rocket invented in 1926 by Robert H.
> Goddard,
> an application which has continued to the present. [Wikipedia](https://en.wikipedia.org/wiki/Liquid_oxygen)

[ephemerista]: https://gitlab.com/librespacefoundation/ephemerista/ephemerista-simulator

[artes]: https://connectivity.esa.int/projects/ossmisi