# Changelog

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.10...lox-frames-v0.1.0-alpha.11) - 2026-03-05

### Other

- add crate-level READMEs
- *(lox-frames)* add doc comments
- clean up metadata

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.9...lox-frames-v0.1.0-alpha.10) - 2026-03-05

### Other

- updated the following local packages: lox-core, lox-math, lox-units, lox-bodies, lox-time

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.8...lox-frames-v0.1.0-alpha.9) - 2026-03-04

### Other

- *(lox-analysis)* add more tests
- *(lox-analysis)* monomorphize scenarios and analyses

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.7...lox-frames-v0.1.0-alpha.8) - 2026-03-02

### Other

- updated the following local packages: lox-core, lox-time, lox-test-utils, lox-math, lox-units, lox-bodies

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.6...lox-frames-v0.1.0-alpha.7) - 2026-02-27

### Other

- updated the following local packages: lox-core, lox-time, lox-math, lox-units, lox-bodies

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.5...lox-frames-v0.1.0-alpha.6) - 2026-02-25

### Added

- add conversions to `dyn` types

### Fixed

- fix various lints

### Other

- add into_dyn tests
- add fast paths for no-op conversions

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.4...lox-frames-v0.1.0-alpha.5) - 2026-02-22

### Added

- *(lox-frames)* add J2000 frame
- add optional serde feature
- *(lox-orbits)* re-design trajectories
- *(lox-frames)* add frame ID to builtin frames

### Fixed

- *(lox-frames/lox-orbits)* fix TEME transformation

### Other

- *(lox-frames)* test Earth-based frames
- *(lox-time)* simplify TAI<->UTC conversions
- *(lox-frames)* rewrite frame transforms
- *(lox-time)* implement offsets via `OffsetProvider` trait
- *(lox-time)* make LSP trait easier to implement
- *(lox-time)* implement offsets via `OffsetProvider` trait

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.3...lox-frames-v0.1.0-alpha.4) - 2025-10-29

### Added

- *(lox-units)* add Keplerian elements
- *(lox-derive)* implement OffsetProvider derive macro
- *(lox-earth)* implement new EOP parser and data provider

### Other

- move lox-units code to lox-core
- update SPDX headers and add helper script
- make Lox REUSE-compliant
- get rid of float_eq and lox_math::is_close
- start reimplementation of frame transformations
- refactor all the things
- *(lox-time)* use provider pattern
- *(lox-units/lox-math)* move constants to lox-units

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.2...lox-frames-v0.1.0-alpha.3) - 2025-09-19

### Other

- update Cargo.toml dependencies

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-frames-v0.1.0-alpha.1...lox-frames-v0.1.0-alpha.2) - 2025-07-18

### Other

- *(lox-space)* release v0.1.0-alpha.24

## 0.1.0-alpha.1

### Added

- Initial release of `lox-frames` crate
- Reference frame transformations factored out from `lox-orbits`
- Frame types: `Icrf`, `Cirf`, `Tirf`, `Itrf`, `Iau<T>`, `DynFrame`
- Frame traits: `ReferenceFrame`, `QuasiInertial`, `BodyFixed`, `TryRotateTo`
- Rotation type and operations
- IAU body-fixed frame transformations
- IERS frame transformations (ICRF/CIRF/TIRF/ITRF)