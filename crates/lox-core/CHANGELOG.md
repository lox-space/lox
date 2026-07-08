# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.15](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.14...lox-core-v0.1.0-alpha.15) - 2026-07-08

### Added

- *(lox-core)* add tolerance and iteration builders to root finders
- *(lox-approx)* add lox-approx crate
- *(lox-core)* add itemized PropagationLosses to the comms module
- *(lox-core)* add FrequencyRange, FrequencyBand parsing, and Power/Temperature unit traits

### Fixed

- *(lox-core)* harden root-finder convergence errors
- *(lox-core)* harden Brent and remove unused Secant root-finder
- *(lox-core)* harden Brent and remove unused Secant root-finder

### Other

- *(lox-core)* extract callback and zero-crossing modules from roots
- *(lox-core)* split NonFinite into callback and diverged-step errors
- *(lox-core)* split NonFinite into callback and diverged-step errors
- *(lox-core)* introduce type-erased LoxError and make plain closures the Callback default
- *(lox-core)* improve roots test coverage
- *(lox-core/lox-orbits)* move ZeroCrossing into roots and reuse bracket endpoint values
- *(lox-core/lox-time)* remove TimeDelta sentinels
- *(lox-core)* [**breaking**] move FrequencyBand and FrequencyRange to new comms module

## [0.1.0-alpha.14](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.13...lox-core-v0.1.0-alpha.14) - 2026-05-26

### Other

- *(lox-core)* fix broken intra-doc links

## [0.1.0-alpha.13](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.12...lox-core-v0.1.0-alpha.13) - 2026-05-18

### Added

- *(lox-core)* [**breaking**] port ERFA spherical and geodetic helpers
- *(lox-core)* [**breaking**] port ERFA angle and day-fraction helpers
- *(lox-time)* add no_std compat
- *(lox-bodies)* add no_std compat
- *(lox-core)* make lox-core no_std compatible

### Other

- *(lox-core)* remove num-traits dependency
- *(lox-core)* add no_std tests and clippy
- *(lox-core)* use num_traits::Float for trig math
- swap std to core
- *(lox-core)* use core::error::Error
- *(lox-core)* replace datetime regex with nom parser

## [0.1.0-alpha.12](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.11...lox-core-v0.1.0-alpha.12) - 2026-05-16

### Added

- *(lox-odm)* add OMM type
- *(lox-core)* add new units for S/C modelling

### Other

- document trajectory constructor panics
- *(lox-odm)* add fuzzer and improve test cov
- move `MeanElements` to `lox-core`

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.10...lox-core-v0.1.0-alpha.11) - 2026-04-26

### Fixed

- add missing serde derives

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.9...lox-core-v0.1.0-alpha.10) - 2026-04-20

### Added

- *(core)* add modified equinoctial elements

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.8...lox-core-v0.1.0-alpha.9) - 2026-04-03

### Added

- *(lox-itur)* port `itur` to Rust

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.7...lox-core-v0.1.0-alpha.8) - 2026-03-31

### Added

- *(lox-comms)* add better receiver model and PFD calc

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-core-v0.1.0-alpha.6...lox-core-v0.1.0-alpha.7) - 2026-03-26

### Added

- *(lox-core)* add equinoctial type I elements
- *(lox-core)* implement Hermite interpolation for trajectories
- *(lox-time)* add time series

### Other

- re-export glam types from lox-core
- fix all cargo-shear lints

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
