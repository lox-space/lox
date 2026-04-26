# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
