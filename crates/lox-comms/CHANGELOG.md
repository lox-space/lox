# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.4...lox-comms-v0.1.0-alpha.5) - 2026-03-05

### Other

- updated the following local packages: lox-core

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.3...lox-comms-v0.1.0-alpha.4) - 2026-03-04

### Added

- *(lox-analysis)* add `lox-analysis` crate

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.2...lox-comms-v0.1.0-alpha.3) - 2026-03-02

### Other

- updated the following local packages: lox-core, lox-test-utils

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.1...lox-comms-v0.1.0-alpha.2) - 2026-02-27

### Other

- updated the following local packages: lox-core

## [0.1.0-alpha.1](https://github.com/lox-space/lox/releases/tag/lox-comms-v0.1.0-alpha.1) - 2026-02-25

### Added

- *(lox-comms)* add Python wrapper
- *(lox-comms)* add serde optional feature
- *(lox-comms)* implement link budgets
- *(lox-comms)* add comms systems
- *(lox-comms)* implement Transmitter/Receiver/Channel abstractions
- *(lox-comms)* implement antenna patterns
- *(lox-comms)* implement FSPL
- *(lox-comms)* add `lox-comms` crate scaffolding

### Fixed

- correct inconsistent trait-contract AntennaGain::beamwidth()
- explicitly handle case where beamwidth is not defined

### Other

- fix formatting
- Remove `lox-comms` crate ([#112](https://github.com/lox-space/lox/pull/112))
- Core No More ([#68](https://github.com/lox-space/lox/pull/68))
