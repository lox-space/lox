# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-time-v0.1.0-alpha.0...lox-time-v0.1.0-alpha.1) - 2024-11-12

### Other

- fix clippy lints
- update pyo3 and fix deprecations

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-time-v0.1.0-alpha.0) - 2024-07-19

### Other
- Rename lox-utils to lox-math because the former is taken ([#146](https://github.com/lox-space/lox/pull/146))
- Add crate descriptions ([#145](https://github.com/lox-space/lox/pull/145))
- Align versions ([#143](https://github.com/lox-space/lox/pull/143))
- Add day of year accessor ([#137](https://github.com/lox-space/lox/pull/137))
- Add `Time` constructor for two-part Julian Dates ([#132](https://github.com/lox-space/lox/pull/132))
- Validate elevation and visibility analysis
- Add `from_seconds` constructor to Python
- Expose time component to Python
- Fix rebase
- Impl event and window detection from Python
- Implement `TimeDelta` ranges
- Fix transformations
- Add trajectory transformation
- Rename
- Fix some copypasta and AI bugs
- Re-implement propagation and Keplerian elements
- Prototype trajectory
- Generate body-fixed frame transformations
- Implement no-ops; remove blanket impl transforms
- Implement body-fixed transformation
- Update documentation for top-level lox-time modules ([#110](https://github.com/lox-space/lox/pull/110))
- Implement new Python API for `lox-time` and add `TryToScale` trait ([#103](https://github.com/lox-space/lox/pull/103))
- Refactor time scale transformations ([#102](https://github.com/lox-space/lox/pull/102))
- Implement `DeltaUt1Tai` provider ([#101](https://github.com/lox-space/lox/pull/101))
- Split up `lox-eop` ([#100](https://github.com/lox-space/lox/pull/100))
- Implement `LeapSecondsProvider` trait with builtin and LSK impls ([#99](https://github.com/lox-space/lox/pull/99))
- Refactor `lox-time` Rust and Python API - Part I ([#94](https://github.com/lox-space/lox/pull/94))
- Calculate delta UT1-TAI from EarthOrientationParams ([#93](https://github.com/lox-space/lox/pull/93))
- Clean up todos ([#88](https://github.com/lox-space/lox/pull/88))
- Align casing of types with Rust API guidelines ([#86](https://github.com/lox-space/lox/pull/86))
- Hoist shared constants and type aliases ([#84](https://github.com/lox-space/lox/pull/84))
- Implement TAI <-> UTC conversion ([#81](https://github.com/lox-space/lox/pull/81))
- Fix TimeDelta.from_decimal_seconds
- Implement TT <-> TDB transformations ([#73](https://github.com/lox-space/lox/pull/73))
- Replace lox_time::continuous with smaller top-level modules ([#72](https://github.com/lox-space/lox/pull/72))
- Implement two-way TCB <-> TDB conversion ([#71](https://github.com/lox-space/lox/pull/71))
- Implement two-way TT-TCG transformation ([#70](https://github.com/lox-space/lox/pull/70))
- Include InvalidTimeDelta detail in error message ([#69](https://github.com/lox-space/lox/pull/69))
- Subsecond-based time implementation ([#67](https://github.com/lox-space/lox/pull/67))
- Core No More ([#68](https://github.com/lox-space/lox/pull/68))
- Factor lox-time into new crate ([#65](https://github.com/lox-space/lox/pull/65))
