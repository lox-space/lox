# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.16](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.15...lox-comms-v0.1.0-alpha.16) - 2026-06-24

### Added

- add a flip() function to LinkDirection
- *(lox-comms)* line-item budget report and C/N combining (Track F.1/F.2)
- *(lox-comms)* integrate ACM into the pipeline via LinkBudget::modulate_best
- *(lox-space)* [**breaking**] expose rain-degraded G/T via link_type in Python API
- *(lox-comms)* rain-degraded G/T for downlinks (ITU-R P.618 §8.2)
- *(lox-comms)* direction-aware link-budget pointing via Pointing enum
- *(lox-comms)* replace pfd_mask with piecewise-linear PfdMask
- *(lox-comms)* add phi angle to antenna gain trait
- *(lox-comms)* add AntennaFrame

### Fixed

- validate bandwidth in budget views, restore stub coverage
- bandwidth-mismatch guard in evaluate, stale docs, broken pyi stub
- *(lox-comms)* floor gaussian and parabolic pattern gains

### Other

- *(lox-comms)* flatten noise_power to Result via AbsolutePowerUnavailable
- *(lox-comms)* [**breaking**] rework the link budget into a discoverable pipeline
- close coverage gaps in modcod, channel, and Python bindings
- *(lox-comms)* [**breaking**] restructure channel layer around waveform/modcod split
- *(lox-comms)* cover LinkParameters serde validation
- *(lox-comms)* [**breaking**] consume PropagationLosses and drop the lox-itur dependency
- *(lox-core)* [**breaking**] move FrequencyBand and FrequencyRange to new comms module
- *(lox-comms)* [**breaking**] introduce LinkParameters for LinkStats::for_link
- *(lox-comms)* [**breaking**] replace CommunicationSystem with link terminals and Eirp/GOverT traits
- *(lox-comms)* use to_radians in slant_range
- *(lox-comms)* replace dipole directivity integration with closed form
- *(lox-comms)* use Angle::ZERO everywhere
- *(lox-comms)* use antenna frame for patterned antenna
- *(lox-comms)* remove beamwidth from AntennaGain trait

## [0.1.0-alpha.15](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.14...lox-comms-v0.1.0-alpha.15) - 2026-06-08

### Added

- *(lox-space)* proper pickle for lumped CommunicationSystem; close test gaps
- *(lox-comms)* add Channel::apply for modulation-aware link stats
- *(lox-comms)* add Eirp/Gt lumped variants and Option<Antenna> on CommunicationSystem
- *(lox-comms)* add LinkBudgetError; SI-2019 Boltzmann constant

### Fixed

- *(lox-comms)* return Result from with_interference; pickle lumped TX/RX
- *(lox-comms)* wire up FrequencyMismatch error; tighten Python stub union
- *(lox-comms)* tighten Gt sentinel docs, receiver_with guard, lumped test rigor
- *(lox-comms)* expand error test coverage; refresh Boltzmann references

### Other

- *(lox-comms)* close coverage gaps for enum dispatch and error paths
- *(lox-comms)* derive LinkBudgetError via thiserror
- *(lox-comms)* changelog for link-budget API redesign
- *(lox-comms)* move lox-test-utils to dev-dependencies
- *(lox-comms)* split LinkStats; introduce ModulatedLinkStats
- *(lox-comms)* mark AntennaPattern/Modulation/LinkDirection non_exhaustive
- *(lox-comms)* lift Transmitter into enum over AmplifierTransmitter
- *(lox-comms)* rename Receiver Simple/Complex to NoiseTemperature/Cascade
- *(lox-comms)* fix tautological ConstantAntenna doc comment
- *(lox-comms)* rename Antenna Simple/Complex to Constant/Patterned
- *(lox-comms)* return Result from CommunicationSystem accessors
<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->


### Changed

- *(lox-comms)* Antenna variants renamed: `Simple`/`Complex` → `Constant`/`Patterned` (inner structs follow: `ConstantAntenna`/`PatternedAntenna`)
- *(lox-comms)* Receiver variants renamed: `Simple`/`Complex` → `NoiseTemperature`/`Cascade` (inner structs: `NoiseTempReceiver`/`CascadeReceiver`)
- *(lox-comms)* `Transmitter` lifted from struct into an enum; original struct becomes `AmplifierTransmitter`
- *(lox-comms)* `CommunicationSystem.antenna` is now `Option<Antenna>`; lumped (`Eirp`/`Gt`) variants pair with `None`
- *(lox-comms)* `LinkStats::calculate` returns `Result<_, LinkBudgetError>` and takes a `bandwidth: Frequency` parameter (no longer takes `Channel`)
- *(lox-comms)* `LinkStats` split: modulation-derived fields move to a new `ModulatedLinkStats` produced via `Channel::apply(link)`
- *(lox-comms)* `LinkStats.carrier_rx_power` and `LinkStats.noise_power` are now `Option<Decibel>` (`None` for lumped-G/T receivers)
- *(lox-comms)* Boltzmann constant updated to the SI-2019 exact value `1.380_649e-23`
- *(lox-comms)* `Antenna`, `Transmitter`, `Receiver`, `AntennaPattern`, `Modulation`, `LinkDirection`, and `LinkBudgetError` marked `#[non_exhaustive]`

### Added

- *(lox-comms)* `EirpTransmitter` and `Transmitter::Eirp` lumped variant
- *(lox-comms)* `GtReceiver` and `Receiver::Gt` lumped variant
- *(lox-comms)* `LinkBudgetError` enum for validation failures
- *(lox-comms)* `CommunicationSystem::eirp_only`, `gt_only`, `amplifier_with`, `receiver_with` tier constructors
- *(lox-comms)* `Channel::apply(LinkStats) -> ModulatedLinkStats`
- *(lox-comms)* `CommunicationSystem::eirp_at(angle)` and `gt_at(angle)` accessors

## [0.1.0-alpha.14](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.13...lox-comms-v0.1.0-alpha.14) - 2026-05-26

### Other

- updated the following local packages: lox-core, lox-itur, lox-test-utils

## [0.1.0-alpha.13](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.12...lox-comms-v0.1.0-alpha.13) - 2026-05-18

### Added

- *(lox-math/lox-units)* add no_std compat

## [0.1.0-alpha.12](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.11...lox-comms-v0.1.0-alpha.12) - 2026-05-16

### Other

- updated the following local packages: lox-core, lox-test-utils, lox-itur

## [0.1.0-alpha.11](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.10...lox-comms-v0.1.0-alpha.11) - 2026-04-26

### Other

- updated the following local packages: lox-core, lox-itur

## [0.1.0-alpha.10](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.9...lox-comms-v0.1.0-alpha.10) - 2026-04-20

### Other

- updated the following local packages: lox-core, lox-itur

## [0.1.0-alpha.9](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.8...lox-comms-v0.1.0-alpha.9) - 2026-04-03

### Added

- *(lox-itur)* port `itur` to Rust

## [0.1.0-alpha.8](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.7...lox-comms-v0.1.0-alpha.8) - 2026-03-31

### Added

- *(lox-comms)* add better receiver model and PFD calc

## [0.1.0-alpha.7](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.6...lox-comms-v0.1.0-alpha.7) - 2026-03-26

### Other

- re-export glam types from lox-core
- fix all cargo-shear lints

## [0.1.0-alpha.6](https://github.com/lox-space/lox/compare/lox-comms-v0.1.0-alpha.5...lox-comms-v0.1.0-alpha.6) - 2026-03-05

### Other

- add crate-level READMEs
- *(lox-derive/lox-test-utils/lox-comms/lox-time)* add doc comments
- clean up metadata

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
