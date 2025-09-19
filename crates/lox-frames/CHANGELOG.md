# Changelog

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