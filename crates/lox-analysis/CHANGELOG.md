# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.15](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.14...lox-analysis-v0.1.0-alpha.15) - 2026-06-24

### Other

- *(deps)* bump geo to 0.33 and geojson to 1.0
- *(lox-core)* [**breaking**] move FrequencyBand and FrequencyRange to new comms module
- *(lox-analysis)* [**breaking**] attach named comms terminals to assets

## [0.1.0-alpha.14](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.13...lox-analysis-v0.1.0-alpha.14) - 2026-06-08

### Added

- *(lox-analysis)* make ephemeris optional via NoEphemeris type-state

### Other

- *(lox-analysis)* split inter-sat LOS detect fn

## [0.1.0-alpha.13](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.12...lox-analysis-v0.1.0-alpha.13) - 2026-05-26

### Added

- *(lox-analysis)* wire pass_direction_of into AccessAnalysis::compute
- *(lox-analysis)* add AccessError::PassDirection variant
- *(lox-analysis)* add pass_direction_of classifier
- *(lox-analysis)* add PassDirection and AccessWindow types
- *(lox-analysis)* add SarAccessAnalysis and Sentinel-1 test
- *(lox-analysis)* implement AccessPayload for SarPayload
- *(lox-analysis)* add SarPayload
- *(lox-analysis)* add AccessPayload and PayloadAccessor traits

### Other

- *(lox-analysis)* clarify PassDirection contract; fix stale 6h test message
- *(lox-analysis)* assert PassDirection; pin both-directions over Europe
- *(lox-analysis)* [**breaking**] AccessResults stores AccessWindow
- *(lox-analysis)* extract sub_sat_sample helper
- address review comments
- cleanups
- *(lox-analysis)* [**breaking**] rename imaging_payload to optical_payload
- *(lox-analysis)* [**breaking**] replace ImagingAnalysis with generic analysis
- *(lox-analysis)* [**breaking**] port ImagingPayload to OpticalPayload, impl AccessPayload
- *(lox-analysis)* split imaging module into submodules
- *(lox-analysis)* [**breaking**] rename "coverage" feature to "imaging"

## [0.1.0-alpha.12](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.11...lox-analysis-v0.1.0-alpha.12) - 2026-05-18

### Added

- *(lox-core)* [**breaking**] port ERFA spherical and geodetic helpers
- *(lox-math/lox-units)* add no_std compat

### Other

- *(lox-core)* add no_std tests and clippy
- *(lox-core)* replace datetime regex with nom parser

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.10...lox-analysis-v0.1.0-alpha.11) - 2026-05-16

### Other

- updated the following local packages: lox-core, lox-time, lox-orbits, lox-test-utils, lox-bodies, lox-comms, lox-ephem, lox-frames, lox-math

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.9...lox-analysis-v0.1.0-alpha.10) - 2026-05-12

### Fixed

- *(lox-analysis)* densify sparse polygons

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.8...lox-analysis-v0.1.0-alpha.9) - 2026-05-11

### Fixed

- *(lox-time)* error when comparing mismatched time scales

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.7...lox-analysis-v0.1.0-alpha.8) - 2026-04-26

### Fixed

- add missing serde derives

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.6...lox-analysis-v0.1.0-alpha.7) - 2026-04-20

### Other

- updated the following local packages: lox-core, lox-bodies, lox-comms, lox-time, lox-ephem, lox-frames, lox-math, lox-orbits

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.5...lox-analysis-v0.1.0-alpha.6) - 2026-04-03

### Other

- updated the following local packages: lox-core, lox-comms, lox-bodies, lox-time, lox-ephem, lox-frames, lox-math, lox-orbits

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.4...lox-analysis-v0.1.0-alpha.5) - 2026-03-31

### Other

- updated the following local packages: lox-core, lox-comms, lox-bodies, lox-time, lox-ephem, lox-frames, lox-math, lox-orbits

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.3...lox-analysis-v0.1.0-alpha.4) - 2026-03-26

### Added

- *(lox-orbits)* add Kozai J2/J4 propagators
- *(lox-orbits)* add Brouwer-Lyddane J2 propagator
- *(lox-analysis)* implement AOI-based coverage detection
- *(lox-analysis)* add G/S<->S/C filter to vis analysis
- *(lox-analysis)* accept a closure as inter-satellite filter
- *(lox-analysis)* adds an optional satellite pair filter to visibility analysis
- *(lox-analysis)* add basic environmental analysis for power budget
- *(lox-analysis)* add accessors for network/constellation ids

### Fixed

- *(lox-analysis)* always check for occultation by central body

### Other

- re-export glam types from lox-core
- *(lox-analysis)* use `TimeSeries` for `Pass`
- fix all cargo-shear lints

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.2...lox-analysis-v0.1.0-alpha.3) - 2026-03-05

### Other

- add crate-level READMEs
- *(lox-analysis/lox-earth/lox-ephem)* add doc comments
- clean up metadata

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-analysis-v0.1.0-alpha.1...lox-analysis-v0.1.0-alpha.2) - 2026-03-05

### Added

- *(lox-orbits/lox-analysis)* add constellation design tools

## [0.1.0-alpha.1](https://github.com/lox-space/lox/releases/tag/lox-analysis-v0.1.0-alpha.1) - 2026-03-04

### Added

- *(lox-analysis)* add `lox-analysis` crate

### Other

- *(lox-analysis)* add more tests
- *(lox-analysis)* add body-fixed frame to G/S
- *(lox-analysis)* monomorphize scenarios and analyses
- *(lox-analysis)* use scenario-based analyses
