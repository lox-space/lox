# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.5...lox-core-v0.1.0-alpha.6) - 2026-03-05

### Other

- add crate-level READMEs
- *(lox-core/lox-math/lox-units)* add doc comments
- clean up metadata

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.4...lox-core-v0.1.0-alpha.5) - 2026-03-05

### Added

- *(lox-orbits/lox-analysis)* add constellation design tools

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.3...lox-core-v0.1.0-alpha.4) - 2026-03-02

### Added

- *(lox-core)* add time units

### Other

- *(lox-core/lox-time)* deduplicate time range APIs

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.2...lox-core-v0.1.0-alpha.3) - 2026-02-27

### Added

- *(lox-space)* make Python wrapper unitful

### Other

- *(lox-space)* more tests
- *(lox-core)* improve interpolation performance
- *(lox-orbits)* refactor event detection

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.1...lox-core-v0.1.0-alpha.2) - 2026-02-25

### Added

- *(lox-core)* add `Decibel` unit type and `Kelvin` alias

### Other

- *(lox-orbits)* address review comments
- *(lox-orbits/lox-core)* move LLA <-> Cartesian logic to lox-core

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.0...lox-core-v0.1.0-alpha.1) - 2026-02-22

### Added

- add optional serde feature
- *(lox-core)* implement chrono interop
- *(lox-time)* implement chrono interop
- *(lox-orbits)* implement basic J2 numerical propagator
- *(lox-time)* use compensated sum for two-float deltas
- *(lox-orbits)* re-design trajectories
- add SSO builder and Earth ephemeris

### Fixed

- *(lox-orbits)* fix unit mismatches
- *(lox-time)* improve precision of TCB and TCG conversions

### Other

- set q0 as remainder (from Helge)
- rewrite from closures to a Callback trait, should_panic for roots tests
- assert err directly, no need to match it first
- Combine all root errors to common error domain, bubble up errors everywhere.
- *(lox-core)* simplify Series type
- *(lox-frames)* rewrite frame transforms
- *(lox-time)* make LSP trait easier to implement
- refactor orbit tracing

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-core-v0.1.0-alpha.0) - 2025-10-29

### Fixed

- fix doctests

### Other

- move time primitives to lox-core
- move math code to lox-core
- move lox-units code to lox-core
