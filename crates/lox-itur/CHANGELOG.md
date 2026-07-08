# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.5...lox-itur-v0.1.0-alpha.6) - 2026-07-08

### Added

- *(lox-approx)* add lox-approx crate

### Other

- dependency cleanup
- *(lox-itur)* [**breaking**] replace EnvironmentalLosses with ItuProvider::propagation_losses

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.4...lox-itur-v0.1.0-alpha.5) - 2026-05-26

### Added

- *(lox-itur)* provider methods for p618 rain + scintillation
- *(lox-itur)* provider methods for p837
- *(lox-itur)* provider methods for p840
- *(lox-itur)* provider methods for p839
- *(lox-itur)* provider method for p453 Nwet
- *(lox-itur)* provider methods for p836
- *(lox-itur)* provider methods for p1510
- *(lox-itur)* add bundle packager (cargo run --bin pack)
- *(lox-itur)* ItuProvider grid_xyz cache helper
- *(lox-itur)* add ItuProvider skeleton (open + manifest)
- *(lox-itur)* add bundle manifest type
- *(lox-itur)* add NPY parser (npz module)

### Other

- *(lox-itur)* enable missing_docs lint and document public API
- *(lox-itur)* index-slot grid cache (lock-free, no per-call hashing)
- *(lox-itur)* fix rustdoc intra-doc link and bare URL warnings
- *(lox-itur)* README documents bundle workflow
- *(lox-itur)* rewrite crate-level rustdoc for provider pattern
- *(lox-itur)* drop build.rs + data.rs + zstd dep
- *(lox-itur)* delete LazyGrid free fns; tests use provider
- *(lox-itur)* EnvironmentalLosses::new takes &ItuProvider
- *(lox-itur)* provider topographic_altitude parity (p1511)
- *(lox-itur)* baseline divan benches before provider refactor

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.3...lox-itur-v0.1.0-alpha.4) - 2026-05-18

### Added

- *(lox-math/lox-units)* add no_std compat

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.2...lox-itur-v0.1.0-alpha.3) - 2026-05-16

### Other

- updated the following local packages: lox-core, lox-test-utils

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.1...lox-itur-v0.1.0-alpha.2) - 2026-04-26

### Other

- updated the following local packages: lox-core

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-itur-v0.1.0-alpha.0...lox-itur-v0.1.0-alpha.1) - 2026-04-20

### Other

- updated the following local packages: lox-core

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-itur-v0.1.0-alpha.0) - 2026-04-03

### Added

- *(lox-itur)* port `itur` to Rust

### Fixed

- *(lox-itur)* clamp elevation to 5 deg to avoid singularity

### Other

- *(lox-itur)* refactor constructors
