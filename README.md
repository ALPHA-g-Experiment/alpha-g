# ALPHA-g

[![Test Status](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml)

This is a [Cargo
Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) with a set
of packages with multiple library and binary crates useful in the data analysis
of the ALPHA-g experiment at CERN.

## Packages

Read the individual README.md file of each package to obtain more information.

- [Detector](detector/README.md): Library crate to handle the raw output of the
 ALPHA-g detectors.
- [Analysis](analysis/README.md): Set of executables that perform common 
post-processing/offline analysis.

## Want to contribute?

There are multiple ways to contribute:
- Install and test individual ALPHA-g packages. If they don't work as expected
 please [open an issue](https://github.com/DJDuque/alpha-g/issues/new).
- Comment/propose a fix on some of the current [open 
issues](https://github.com/DJDuque/alpha-g/issues).
- Read through the documentation of individual packages. If there is 
  something confusing, or you have a suggestion for something that could be 
  improved, please let the maintainer(s) know.
- Help evaluate [open pull requests](https://github.com/DJDuque/alpha-g/pulls),
  by testing locally and reviewing what is proposed.
