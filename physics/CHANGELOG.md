# Changelog

Note that this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) and all notable
changes will be documented in this file.

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Fixed

- New pad and wire calibration from run 11084. Some PWBs were replaced since
  run 10418, but the detector was only turned on and taking data again since
  run 11084 this year.

  Pad baseline calibration was done using run 11192. Both pad and wire gain
  calibration were done using run 11186 (wire data suppression at 6000 instead
  of the nominal 1500 to remove some noise). Wire baseline was tested with run
  11185, but no significant changes were observed to grant a new calibration.

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
