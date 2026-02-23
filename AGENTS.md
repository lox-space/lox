<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# AGENTS.md

## Project Overview

Lox is a safe, ergonomic Rust astrodynamics library with Python bindings (via PyO3/maturin). It provides high-precision time systems, reference frame transformations, orbital mechanics, ground station analysis, and event detection for the modern space industry.

**License**: MPL-2.0 (REUSE-compliant — all source files must have SPDX headers).

## Repository Structure

```
crates/
  lox-core/        # Foundation: coordinates, orbital elements, anomalies
  lox-units/       # Physical unit newtypes (Angle, Distance, Velocity, etc.)
  lox-time/        # Astronomical time scales (TAI, TT, TDB, TCB, TCG, UT1, UTC)
  lox-math/        # Series evaluation, interpolation
  lox-bodies/      # Celestial body definitions (SPICE-derived constants)
  lox-frames/      # Reference frame transformations (ICRF, J2000, ITRF, TEME, IAU_*)
  lox-ephem/       # Ephemeris parsing (SPK/SPICE kernels)
  lox-earth/       # Earth-specific: EOP, nutation, precession
  lox-io/          # I/O for standard formats (NDM/XML, SPICE, CSV)
  lox-orbits/      # Orbit modeling, propagators, visibility, events
  lox-space/       # High-level facade API + Python bindings
  lox-derive/      # Procedural macros
  lox-test-utils/  # ApproxEq trait, benchmarking utilities
tools/
  lox-gen/         # Code generation tooling
data/              # Reference data (IERS, SPICE kernels, TLEs, CSV trajectories)
```

### Crate Dependency Graph

```
lox-core ← lox-units, lox-math, lox-time, lox-bodies
         ← lox-frames, lox-ephem, lox-earth, lox-io
         ← lox-orbits ← lox-space (facade + Python bindings)
```

`lox-space` is the public-facing crate that re-exports and wraps everything. Python users interact exclusively with `lox-space`.

## Build & Test

**Prerequisites**: Rust 1.90+ (edition 2024), [just](https://github.com/casey/just), [uv](https://github.com/astral-sh/uv) (for Python), cargo-nextest.

```bash
just test           # Run all tests (rstest + doctest + pytest)
just rstest         # Rust tests via cargo nextest
just doctest        # Rust doc tests
just pytest         # Build maturin + run Python tests
just build-pyo3     # Build Python extension module
just lint           # clippy + rustfmt + REUSE compliance
just coverage       # Generate code coverage report
```

## Critical Conventions

### Module Convention

This codebase uses the **modern Rust module convention** (`foo.rs` + `foo/` directory) exclusively. There are **no `mod.rs` files**. Do not introduce `mod.rs` files.

### Unit Convention (strictly enforced)

| Context | Position | Velocity | Notes |
|---------|----------|----------|-------|
| **Internal Rust storage** | meters (m) | m/s | All core types store SI units |
| **Python API** | kilometers (km) | km/s | Conversion at the PyO3 boundary |
| **CSV files** | kilometers | km/s | Conversion at parse time |
| **Ephemeris (SPK)** | kilometers | km/s | Conversion where consumed |

Key details:
- `Cartesian::from_vecs()` expects **meters**.
- `Distance` stores meters internally, even when created via `kilometers()`.
- `GravitationalParameter` stores m³/s² internally, even when created via `km3_per_s2()`.
- `Angle` stores **radians** internally.
- `GroundLocation` altitude is in **km** (converted to m internally in `body_fixed_position()`).

**Getting this wrong will produce silently incorrect results. Always verify units.**

### Code Style

- **SPDX license headers** are mandatory on all source files (checked by `reuse lint`).
- **Error handling**: Use `thiserror` with domain-specific enum error types (e.g., `TimeError`, `TrajectoryError`).
- **Type safety**: Zero-sized marker traits for time scales (`TimeScale`) and reference frames (`ReferenceFrame`). Prefer compile-time guarantees over runtime checks.
- **Builder pattern**: Used for complex types (`AzElBuilder`, `TimeBuilder`, `CartesianBuilder`).
- **Provider injection**: EOP and SPK providers are passed explicitly — no global state.
- **Feature flags**: Crate functionality is feature-gated in `lox-space` for minimal dependency footprint.
- **Clippy**: Runs with `-D warnings` (warnings are errors in CI).

### Architecture Patterns

- **Generic orbit type**: `Orbit<S, T, O, R>` parameterized on state, timescale, origin, and frame. `DynOrbit` provides runtime polymorphism when needed.
- **Time representation**: `Time<T: TimeScale>` with femtosecond precision (i64 seconds + attoseconds). Continuous time scales are the default; leap seconds are handled strictly at the UTC I/O boundary.
- **Frame transformations**: Matrix-based rotation pipelines. Transformation chains: ICRF <-> J2000, and CIO-based (CIRF -> TIRF -> ITRF) or equinox-based (MOD -> TOD -> PEF) paths.
- **Dual orbit representations**: Cartesian (position/velocity vectors via `glam::DVec3`) and Keplerian (classical orbital elements), with conversions between them.

## Testing

### Rust Tests
- Unit tests in `#[cfg(test)]` modules within source files.
- `rstest` for parameterized tests; `proptest` for property-based testing (e.g., anomaly conversions).
- `lox-test-utils` provides `ApproxEq` for floating-point comparisons.
- Benchmarks use `divan` and are located in `crates/lox-space/benches/`. All benchmarks should be added to `lox-space`, not to individual crates, so they are centralized in one place. Exception: `lox-test-utils` has its own benchmarks for the `ApproxEq` infrastructure.

### Python Tests
- Located in `crates/lox-space/tests/test_*.py`.
- `conftest.py` provides fixtures: `data_dir`, `provider` (EOPProvider), `oneweb` (TLE constellation), `estrack` (ground stations).
- Markers: `slow`, `benchmark`.

## Key Files for Common Tasks

| Task | Files |
|------|-------|
| Coordinate types | `crates/lox-core/src/coords.rs` |
| Unit types | `crates/lox-core/src/units.rs` |
| Time scales | `crates/lox-time/src/` |
| Frame transforms | `crates/lox-frames/src/` |
| Orbit propagation | `crates/lox-orbits/src/propagators/` |
| Ground analysis | `crates/lox-orbits/src/ground.rs`, `analysis.rs` |
| Event detection | `crates/lox-orbits/src/events.rs` |
| Python bindings | `crates/lox-space/src/*/python.rs` |
| Python API docs | `crates/lox-space/docs/` |

## CI/CD

GitHub Actions workflows in `.github/workflows/`:
- **rust.yml**: Rust tests, clippy, rustfmt, REUSE compliance, code coverage (Codecov).
- **python.yml**: Multi-platform wheel builds (Linux/Windows/macOS), pytest, PyPI release on `lox-space-v*` tags.
- **codspeed.yml**: Performance benchmarking.
- **audit.yml**: Dependency security audit.
- **release.yml**: Release automation.
