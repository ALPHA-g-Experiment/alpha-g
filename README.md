# ALPHA-g

[![Test Status](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml)

This is a [Cargo
Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) with
multiple library and binary crates useful in the data analysis of the ALPHA-g
experiment at CERN.

## Packages

- [Analysis](analysis/README.md): Core analysis binaries.
- [Detector](detector/README.md): Low-level library to handle the raw output of
the ALPHA-g detectors.
- [Physics](physics/README.md): Higher-level library to reconstruct the
annihilation events observed by the ALPHA-g detectors.
