# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
