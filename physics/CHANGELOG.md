# Changelog

Note that this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) and all notable
changes will be documented in this file.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.1.3] - 2024-08-22

### Fixed

- Remove panics due to baseline, gain, and delay calibration for both wires and
  pads. These panics were caused by "safeguards" meant to prevent running for a
  long time without calibrating the detector. In reality, these caused more harm
  than good, so they were removed.
- Bump `alpha_g_detector` to v0.5.1, see 
  [its changelog](https://github.com/ALPHA-g-Experiment/alpha-g/blob/main/detector/CHANGELOG.md#051---2024-08-22)
  for details.

<!-- next-url -->
[Unreleased]: https://github.com/ALPHA-g-Experiment/alpha-g/compare/alpha_g_physics-v0.1.3...HEAD
[0.1.3]: https://github.com/ALPHA-g-Experiment/alpha-g/compare/alpha_g_physics-v0.1.2...alpha_g_physics-v0.1.3
