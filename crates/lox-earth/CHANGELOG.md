# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.20](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.19...lox-earth-v0.1.0-alpha.20) - 2026-03-05

### Other

- add crate-level READMEs
- *(lox-analysis/lox-earth/lox-ephem)* add doc comments
- clean up metadata

## [0.1.0-alpha.19](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.18...lox-earth-v0.1.0-alpha.19) - 2026-03-05

### Other

- updated the following local packages: lox-core, lox-math, lox-bodies, lox-time, lox-frames, lox-io

## [0.1.0-alpha.18](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.17...lox-earth-v0.1.0-alpha.18) - 2026-03-04

### Other

- updated the following local packages: lox-frames

## [0.1.0-alpha.17](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.16...lox-earth-v0.1.0-alpha.17) - 2026-03-02

### Other

- updated the following local packages: lox-core, lox-time, lox-test-utils, lox-math, lox-bodies, lox-frames, lox-io

## [0.1.0-alpha.16](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.15...lox-earth-v0.1.0-alpha.16) - 2026-02-27

### Other

- updated the following local packages: lox-core, lox-time, lox-math, lox-bodies, lox-frames, lox-io

## [0.1.0-alpha.15](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.14...lox-earth-v0.1.0-alpha.15) - 2026-02-25

### Other

- updated the following local packages: lox-test-utils, lox-core, lox-bodies, lox-time, lox-frames, lox-math, lox-io

## [0.1.0-alpha.14](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.13...lox-earth-v0.1.0-alpha.14) - 2026-02-22

### Added

- *(lox-time)* use compensated sum for two-float deltas
- *(lox-orbits)* re-design trajectories
- add SSO builder and Earth ephemeris

### Other

- *(lox-time)* simplify TAI<->UTC conversions
- *(lox-core)* simplify Series type
- *(lox-frames)* rewrite frame transforms
- *(lox-time)* implement offsets via `OffsetProvider` trait
- *(lox-time)* make LSP trait easier to implement
- *(lox-time)* implement offsets via `OffsetProvider` trait
- refactor orbit tracing

## [0.1.0-alpha.13](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.12...lox-earth-v0.1.0-alpha.13) - 2025-10-29

### Added

- *(lox-time)* add attosecond-resolution deltas
- *(lox-units)* extend and document `lox_units::units` module
- *(lox-derive)* implement OffsetProvider derive macro
- add lox-test-utils crate and refine EOP parser interface
- *(lox-earth)* implement new EOP parser and data provider

### Fixed

- fix formatting and use stable rustfmt

### Other

- move lox-units code to lox-core
- *(lox-units)* restructure constants to match std
- add 3rd-party data licenses
- update SPDX headers and add helper script
- make Lox REUSE-compliant
- get rid of float_eq and lox_math::is_close
- *(lox-units)* make Angle value private
- *(lox-bodies/lox-earth)* anglify lox-bodies and lox-earth
- start reimplementation of frame transformations
- *(lox-earth)* add comment
- move type aliases to lox-units
- *(lox-units/lox-math)* move constants to lox-units

## [0.1.0-alpha.12](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.11...lox-earth-v0.1.0-alpha.12) - 2025-09-19

### Other

- update Cargo.toml dependencies

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.10...lox-earth-v0.1.0-alpha.11) - 2025-07-18

### Other

- updated the following local packages: lox-time

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.9...lox-earth-v0.1.0-alpha.10) - 2025-07-01

### Other

- updated the following local packages: lox-math, lox-io, lox-time, lox-bodies

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.8...lox-earth-v0.1.0-alpha.9) - 2025-06-19

### Other

- update Cargo.toml dependencies

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.7...lox-earth-v0.1.0-alpha.8) - 2025-03-04

### Other

- update formatting

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.6...lox-earth-v0.1.0-alpha.7) - 2025-02-11

### Other

- updated the following local packages: lox-time

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.5...lox-earth-v0.1.0-alpha.6) - 2025-02-10

### Other

- updated the following local packages: lox-bodies, lox-math, lox-time

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.4...lox-earth-v0.1.0-alpha.5) - 2025-01-24

### Other

- Set big arrays as static

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.3...lox-earth-v0.1.0-alpha.4) - 2024-12-19

### Other

- updated the following local packages: lox-bodies

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.2...lox-earth-v0.1.0-alpha.3) - 2024-12-19

### Other

- updated the following local packages: lox-bodies

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.1...lox-earth-v0.1.0-alpha.2) - 2024-12-18

### Other

- implement dynamic origin and frame types

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-earth-v0.1.0-alpha.0...lox-earth-v0.1.0-alpha.1) - 2024-11-12

### Other

- updated the following local packages: lox-bodies, lox-math, lox-io, lox-time

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-earth-v0.1.0-alpha.0) - 2024-07-19

### Other
- Rename lox-utils to lox-math because the former is taken ([#146](https://github.com/lox-space/lox/pull/146))
- Add crate descriptions ([#145](https://github.com/lox-space/lox/pull/145))
- Align versions ([#143](https://github.com/lox-space/lox/pull/143))
- Release preparation ([#140](https://github.com/lox-space/lox/pull/140))
- Implement `DeltaUt1Tai` provider ([#101](https://github.com/lox-space/lox/pull/101))
- Split up `lox-eop` ([#100](https://github.com/lox-space/lox/pull/100))
- Align casing of types with Rust API guidelines ([#86](https://github.com/lox-space/lox/pull/86))
- Hoist shared constants and type aliases ([#84](https://github.com/lox-space/lox/pull/84))
- Replace lox_time::continuous with smaller top-level modules ([#72](https://github.com/lox-space/lox/pull/72))
- Core No More ([#68](https://github.com/lox-space/lox/pull/68))
