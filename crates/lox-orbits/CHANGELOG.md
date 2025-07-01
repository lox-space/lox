# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.17](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.16...lox-orbits-v0.1.0-alpha.17) - 2025-07-01

### Fixed

- *(lox-orbits)* fix observables in passes

### Other

- fix clippy lints

## [0.1.0-alpha.16](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.15...lox-orbits-v0.1.0-alpha.16) - 2025-06-23

### Added

- add Pass struct

## [0.1.0-alpha.15](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.14...lox-orbits-v0.1.0-alpha.15) - 2025-06-19

### Other

- fix clippy lints
- try Claude-optimised parallel visibility

## [0.1.0-alpha.14](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.13...lox-orbits-v0.1.0-alpha.14) - 2025-03-04

### Other

- update formatting

## [0.1.0-alpha.13](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.12...lox-orbits-v0.1.0-alpha.13) - 2025-02-12

### Fixed

- *(lox-orbits)* expose methods for PyElevationMask

## [0.1.0-alpha.12](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.11...lox-orbits-v0.1.0-alpha.12) - 2025-02-12

### Added

- *(lox-orbits)* check los with other bodies

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.10...lox-orbits-v0.1.0-alpha.11) - 2025-02-11

### Fixed

- make `Time` and `TimeScale` pickable

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.9...lox-orbits-v0.1.0-alpha.10) - 2025-02-10

### Other

- *(lox-orbits)* switch loop order for visibility

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.8...lox-orbits-v0.1.0-alpha.9) - 2025-02-10

### Added

- *(lox-orbits)* implement line-of-sight calculations

### Other

- *(lox-orbits)* expose concrete `KeplerianElements`
- *(lox-orbits)* parallelise visibility

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.7...lox-orbits-v0.1.0-alpha.8) - 2025-01-24

### Added

- implement `DynTimeScale`

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.6...lox-orbits-v0.1.0-alpha.7) - 2024-12-19

### Other

- updated the following local packages: lox-bodies

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.5...lox-orbits-v0.1.0-alpha.6) - 2024-12-19

### Other

- prefer `Result` over `Option` for `Origin` props

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.4...lox-orbits-v0.1.0-alpha.5) - 2024-12-18

### Other

- implement dynamic origin and frame types

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.3...lox-orbits-v0.1.0-alpha.4) - 2024-11-15

### Fixed

- add pickle support for `ElevationMask`

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.2...lox-orbits-v0.1.0-alpha.3) - 2024-11-14

### Added

- implement elevation masks for ground locations

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.1...lox-orbits-v0.1.0-alpha.2) - 2024-11-12

### Added

- *(lox-orbits)* implement frame and origin change for Python classes
- *(lox-orbits)* implement origin change for `State`

### Other

- fix formatting
- fix clippy lints
- update pyo3 and fix deprecations

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-orbits-v0.1.0-alpha.0...lox-orbits-v0.1.0-alpha.1) - 2024-07-19

### Added
- *(lox-space)* Expose `Observables` constructor ([#152](https://github.com/lox-space/lox/pull/152))

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-orbits-v0.1.0-alpha.0) - 2024-07-19

### Other
- Rename lox-utils to lox-math because the former is taken ([#146](https://github.com/lox-space/lox/pull/146))
- Add crate descriptions ([#145](https://github.com/lox-space/lox/pull/145))
- Align versions ([#143](https://github.com/lox-space/lox/pull/143))
- Release preparation ([#140](https://github.com/lox-space/lox/pull/140))
- Add day of year accessor ([#137](https://github.com/lox-space/lox/pull/137))
- Expose topocentric rotation from Python ([#136](https://github.com/lox-space/lox/pull/136))
- Add accessors for `PyTrajectory` ([#135](https://github.com/lox-space/lox/pull/135))
- Implement trajectory to Numpy array method ([#134](https://github.com/lox-space/lox/pull/134))
- Implement `from_numpy` constructor for `PyTrajectory` ([#133](https://github.com/lox-space/lox/pull/133))
- Fix bodyfixed frames for satellites
- Fix tests
- Implement secant methods and use for geodetic conv
- Implement state to ground
- Return empty windows vec if no events detected
- Implement rotation to LVLH
- Implement PyObservables
- Simplify elevation analysis
- Implement observables
- Validate elevation and visibility analysis
- Implement trajectory csv parser
- Expose elevation
- Fix Python ground propagator
- Implement visibility window detection
- Add `from_seconds` constructor to Python
- Expose event/window detection func from Python
- Python API fixes
- Implement SGP4 propagator
- Implement ensembles and analysis functions
- Implement ground propagator
- Return Numpy arrays
- Use `State` as callback parameter
- Don't push the MSRV to high
- Impl event and window detection from Python
- Implement `TimeDelta` ranges
- Impl event and window detection on trajectories
- Implement generic event and window detection
- Expose `Frame` class from Python
- Generate `to_frame` impl for `PyState`
- Fix typo
- Fix transformations
- Add trajectory transformation
- Rename
- Remove `lox-coords` crate
- Fix some copypasta and AI bugs
- Re-implement propagation and Keplerian elements
- Cleanup
- Prototype trajectory
- Add `CoordinateOrigin` trait
- Generate body-fixed frame transformations
- Implement ICRF <-> body-fixed transformations
- Implement body-fixed transformation
- Start code generator for `PyFrame`
- Prototype orbit state representations
