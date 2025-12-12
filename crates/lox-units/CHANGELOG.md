# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-units-v0.1.0-alpha.1...lox-units-v0.1.0-alpha.2) - 2025-12-12

### Other

- updated the following local packages: lox-core

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-units-v0.1.0-alpha.0...lox-units-v0.1.0-alpha.1) - 2025-10-29

### Added

- *(lox-orbits)* add new Orbit types
- *(lox-units)* add Keplerian elements
- *(lox-time)* add attosecond-resolution deltas
- *(lox-units)* extend and document `lox_units::units` module
- *(lox-derive)* implement OffsetProvider derive macro
- *(lox-units)* add basic cartesian state type
- *(lox-units)* make no_std compatible

### Other

- move lox-units code to lox-core
- *(lox-units)* spell out all the things
- *(lox-units)* restructure constants to match std
- update SPDX headers and add helper script
- make Lox REUSE-compliant
- get rid of float_eq and lox_math::is_close
- *(lox-units)* make Angle value private
- *(lox-bodies/lox-earth)* anglify lox-bodies and lox-earth
- start reimplementation of frame transformations
- consolidate constants in lox-units
- move type aliases to lox-units
- *(lox-space/lox-units)* move unit wrappers to lox-space and add feature
