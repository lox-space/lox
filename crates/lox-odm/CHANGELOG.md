# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-odm-v0.1.0-alpha.1...lox-odm-v0.1.0-alpha.2) - 2026-05-18

### Added

- *(lox-math/lox-units)* add no_std compat
- *(lox-core)* make lox-core no_std compatible

### Other

- remove unused deps and files
- *(lox-core)* add no_std tests and clippy

## [0.1.0-alpha.1](https://github.com/lox-space/lox/releases/tag/lox-odm-v0.1.0-alpha.1) - 2026-05-16

### Added

- *(lox-odm)* integrate with other crates
- *(lox-odm)* impl XML message formats
- *(lox-odm)* impl OMM JSON format
- *(lox-odm)* impl CI KVN format
- *(lox-odm)* impl OMM KVN format
- *(lox-odm)* impl OEM KVN format
- *(lox-odm)* impl OPM KVN format
- *(lox-odm)* add KVN parser
- *(lox-odm)* add KVN AST
- *(lox-odm)* add CI type
- *(lox-odm)* add OMM type
- *(lox-odm)* add OEM type
- *(lox-odm)* add OPM message
- *(lox-odm)* add lox-odm crate

### Fixed

- *(lox-odm)* fix offset handling
- *(lox-odm)* fix parser bugs

### Other

- *(lox-odm)* clean-up dead code
- *(lox-odm)* update docs
- *(lox-odm)* improve test coverage
- *(lox-odm)* address review comments
- *(lox-odm)* improve test coverage
- *(lox-odm)* address review comments
- *(lox-odm)* add fuzzer and improve test cov
- *(lox-odm)* implement `OdmTime` type
- *(lox-orbits)* fix try_new bug and consolidate trajectory error
