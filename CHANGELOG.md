# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0-alpha.5] - 2024-07-12

### Added

- Add a binary app package format that bundles meta-data and image.

## [0.3.0-alpha.4] - 2024-01-24

### Fixed

- Logic to assess if we are on a pre-release

### Added

- Allow passing a timestamp option to `generate` and `get-version`

## [0.3.0-alpha.3] - 2024-01-23

### Added

- Allow reading back pre-release version with `get-version` command

## [0.3.0-alpha.2] - 2024-01-23

### Fixed

- Command line parsing
- CMake module

## [0.3.0-alpha.1] - 2024-01-23

### Breaking

- This release breaks compatibility in the config file format.
- Default script file naming
- Data structure of `info.json`

### Added

- Migration to `semver`-crate
- Added extraction of firmware version from changelog
- Allow specifiyng a git repository and injecting pre-release information

## [0.2.3] - 2022-09-13

- Add version readback to CLI

## [0.2.2] - 2022-09-13

- Update dependencies
- Fix "--use-backdoor" cli flag

## [0.2.1] - 2022-09-06

- Add CI pipeline

## [0.2.0] - 2022-07-05

- Initial public release

### Changed

- Change default values in config: `blocking = true`.
- Rename `fw_id` to `node_id`
- Adjust `AddressRange.end` pointing one past the last byte index
