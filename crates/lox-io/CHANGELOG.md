# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.6...lox-io-v0.1.0-alpha.7) - 2025-07-01

### Other

- fix clippy lints

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.5...lox-io-v0.1.0-alpha.6) - 2025-06-19

### Other

- update Cargo.toml dependencies

## [0.1.0-alpha.5](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.4...lox-io-v0.1.0-alpha.5) - 2025-03-04

### Other

- update formatting
- Add support for ddd KVN dates

## [0.1.0-alpha.4](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.3...lox-io-v0.1.0-alpha.4) - 2025-02-10

### Other

- updated the following local packages: lox-math

## [0.1.0-alpha.3](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.2...lox-io-v0.1.0-alpha.3) - 2025-01-24

### Other

- updated the following local packages: lox-derive

## [0.1.0-alpha.2](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.1...lox-io-v0.1.0-alpha.2) - 2024-12-18

### Other

- update Cargo.toml dependencies

## [0.1.0-alpha.1](https://github.com/lox-space/lox/compare/lox-io-v0.1.0-alpha.0...lox-io-v0.1.0-alpha.1) - 2024-11-12

### Other

- fix clippy lints
- *(lox-io)* remove fast-float due to security advisory

## [0.1.0-alpha.0](https://github.com/lox-space/lox/releases/tag/lox-io-v0.1.0-alpha.0) - 2024-07-19

### Other
- Rename lox-utils to lox-math because the former is taken ([#146](https://github.com/lox-space/lox/pull/146))
- Add crate descriptions ([#145](https://github.com/lox-space/lox/pull/145))
- Align versions ([#143](https://github.com/lox-space/lox/pull/143))
- Release preparation ([#140](https://github.com/lox-space/lox/pull/140))
- Add support for empty lines
- Get clippy happy
- Change vec reference to slice
- Implement field prefix and postfix checking
- Clean-up test formatting
- Fix the unit test
- Add converter for state vector value to oem type
- Add minor comment explanation
- Implement support for META_START and META_STOP
- Fix end of input error
- Add rustdoc for KVN deser for OemType
- Make function non-public
- Fix id and version fields for KVN
- Implement OEM KVN test
- Implement type converters for public errors
- Implement covariance matrix parser
- Fix whitespace parsing
- Implement a parser for state vector
- Enable serializer for OEM type
- Remove _new suffix from parsers
- Remove nom types
- Remove superfluous lifetimes to get clippy happy
- Explain it is generated code
- Encapsulate kvn string split
- Add trait for XML deserialization
- Encapsulate the quickxml deserialization
- Fix comment wrap
- Clean-up extra commas
- Expand KVN spec comment
- Fix typo in rustdoc
- Make single-variant enum into struct
- Add Error derive
- Make KvnDeserializerErr cloneable
- Add _list suffix for consistency
- Make error payload an owned string
- Remove unused types
- Hide doctest line
- Document relaxations and limitations
- Document KVN parsers
- Change date format
- Add whitespace to fix false positives
- Simplify  version field handling by reodering
- Indicate module is for combined instantiation
- Add doc tests
- Simplify parsing imports
- Add KVN parsing tests
- Make the id value optional for KVN
- Simplify the wrapped types
- Make the parser visible in the crate
- Derive KVN deserialization code
- Clean-up string type
- Add module docs
- Reexport the KVN deserializer types
- Remove debug println
- Add missing copyright header
- Add submodule rustdoc
- Restructure public interface
- Remove superfluous types
- Move user-facing classes into the main namespace
- Fix unused with_unit
- Make clippy happy
- Move ndm parsing to lox-io
- Implement `DeltaUt1Tai` provider ([#101](https://github.com/lox-space/lox/pull/101))
- Split up `lox-eop` ([#100](https://github.com/lox-space/lox/pull/100))
- Enable LSK kernel parsing in `lox-io` ([#98](https://github.com/lox-space/lox/pull/98))
- Core No More ([#68](https://github.com/lox-space/lox/pull/68))
