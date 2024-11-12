# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-space-v0.1.0-alpha.2...lox-space-v0.1.0-alpha.3) - 2024-11-12

### Added

- *(lox-orbits)* implement frame and origin change for Python classes

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-space-v0.1.0-alpha.1...lox-space-v0.1.0-alpha.2) - 2024-07-19

### Added
- *(lox-space)* Expose `Observables` constructor ([#152](https://github.com/lox-space/lox/pull/152))

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-space-v0.1.0-alpha.0...lox-space-v0.1.0-alpha.1) - 2024-07-19

### Other
- Use GitHub app for release workflow ([#150](https://github.com/lox-space/lox/pull/150))

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-space-v0.1.0-alpha.0) - 2024-07-19

### Other
- Rename lox-utils to lox-math because the former is taken ([#146](https://github.com/lox-space/lox/pull/146))
- Add crate descriptions ([#145](https://github.com/lox-space/lox/pull/145))
- Align versions ([#143](https://github.com/lox-space/lox/pull/143))
- Release preparation ([#140](https://github.com/lox-space/lox/pull/140))
- Implement trajectory to Numpy array method ([#134](https://github.com/lox-space/lox/pull/134))
- Implement `from_numpy` constructor for `PyTrajectory` ([#133](https://github.com/lox-space/lox/pull/133))
- Fix tests
- Implement state to ground
- Implement PyObservables
- Wrap `Series`
- Simplify elevation analysis
- Expose elevation
- Fix Python ground propagator
- Implement visibility window detection
- Add `from_seconds` constructor to Python
- Expose event/window detection func from Python
- Python API fixes
- Implement SGP4 propagator
- Implement ground propagator
- Return Numpy arrays
- Use `State` as callback parameter
- Impl event and window detection from Python
- Expose `Frame` class from Python
- Fix benchmarks
- Remove `lox-coords` crate
- Prototype trajectory
- Prototype orbit state representations
- Fix benchmark deps ([#108](https://github.com/lox-space/lox/pull/108))
- Move Python wrappers to `lox-bodies` ([#107](https://github.com/lox-space/lox/pull/107))
- Fix typings ([#106](https://github.com/lox-space/lox/pull/106))
- Implement new Python API for `lox-time` and add `TryToScale` trait ([#103](https://github.com/lox-space/lox/pull/103))
- Refactor `lox-time` Rust and Python API - Part I ([#94](https://github.com/lox-space/lox/pull/94))
- Align casing of types with Rust API guidelines ([#86](https://github.com/lox-space/lox/pull/86))
- Hoist shared constants and type aliases ([#84](https://github.com/lox-space/lox/pull/84))
- Implement TAI <-> UTC conversion ([#81](https://github.com/lox-space/lox/pull/81))
- Replace lox_time::continuous with smaller top-level modules ([#72](https://github.com/lox-space/lox/pull/72))
- Subsecond-based time implementation ([#67](https://github.com/lox-space/lox/pull/67))
- Core No More ([#68](https://github.com/lox-space/lox/pull/68))
- Factor lox-time into new crate ([#65](https://github.com/lox-space/lox/pull/65))
- Streamline public API for the `time` module ([#62](https://github.com/lox-space/lox/pull/62))
- Refactor Time ([#56](https://github.com/lox-space/lox/pull/56))
- Add pickle support for bodies ([#51](https://github.com/lox-space/lox/pull/51))
- Refine time representations ([#44](https://github.com/lox-space/lox/pull/44))
- Refactor two-body state vector representation and expose from Python ([#46](https://github.com/lox-space/lox/pull/46))
- Calculate celestial to intermediate-frame-of-date matrix ([#38](https://github.com/lox-space/lox/pull/38))
- Implement IAU1980 nutation ([#23](https://github.com/lox-space/lox/pull/23))
- Add NaifId newtype and mapping to bodies ([#18](https://github.com/lox-space/lox/pull/18))
- Define bodies manually ([#17](https://github.com/lox-space/lox/pull/17))
- Update copyright
- Use flat cargo workspace
