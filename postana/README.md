# ALPHA-g-Postana

[![Test Status](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/alpha-g-postana?labelColor=383f47)](https://crates.io/crates/alpha-g-postana)

Rust package with multiple binary crates. Each executable is a useful tool to 
perform common post-processing/offline analysis on the data of the ALPHA-g 
experiment at CERN.

## Installation

The package can be installed with `cargo`

```bash
cargo install alpha-g-postana
```

## Binaries

- [`alpha-g-copy`](src/bin/alpha-g-copy/README.md): Make local copies of MIDAS 
files from remote hosts.
- [`alpha-g-alpha16-viewer`](src/bin/alpha-g-alpha16-viewer/README.md): 
Visualize the ADC waveforms from the BV and the rTPC.
- [`alpha-g-padwing-viewer`](src/bin/alpha-g-padwing-viewer/README.md):
Visualize the cathode pad waveforms from the rTPC.

## Want to contribute?

There are multiple ways to contribute:
- Install and test individual binaries. If they don't work as expected
 please [open an issue](https://github.com/DJDuque/alpha-g/issues/new).
- Comment/propose a fix on some of the current [open 
issues](https://github.com/DJDuque/alpha-g/issues).
- Read through the documentation. If there is something confusing, or you have
 a suggestion for something that could be improved, please let the maintainer(s)
 know.
- Help evaluate [open pull requests](https://github.com/DJDuque/alpha-g/pulls),
  by testing locally and reviewing what is proposed.
