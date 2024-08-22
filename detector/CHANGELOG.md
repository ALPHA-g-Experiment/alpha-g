# Changelog

Note that this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) and all notable
changes will be documented in this file.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.5.1] - 2024-08-22

### Added

- Updated PWB mapping to include boards swapped from run 10418 onwards:

    | Column | Row | Old board | New board |
    |:-:|:-:|:-:|:-:|
    |2|0|pwb46|pwb90|
    |2|3|pwb77|pwb85|
    |4|0|pwb44|pwb89|
    |4|3|pwb78|pwb87|
    |4|6|pwb45|pwb84|
    |4|7|pwb15|pwb91|
    |5|7|pwb05|pwb81|
    |6|2|pwb06|pwb44|



### Fixed

- Removed panic in AWB and PWB mappings. These were caused by some "safeguards"
  meant to prevent going for a long time without checking that the mappings were
  still valid/up-to-date. In practice, these caused more harm than good, so they
  are now removed.

<!-- next-url -->
[Unreleased]: https://github.com/ALPHA-g-Experiment/alpha-g/compare/alpha_g_detector-v0.5.1...HEAD
[0.5.1]: https://github.com/ALPHA-g-Experiment/alpha-g/compare/alpha_g_detector-v0.5.0...alpha_g_detector-v0.5.1
