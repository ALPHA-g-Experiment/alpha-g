# ALPHA-g-Analysis

[![Test Status](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/alpha-g-analysis?labelColor=383f47)](https://crates.io/crates/alpha-g-analysis)

Rust package with multiple binary crates.

## Getting Started

The core analysis programs should work out of the box on any platform. If you
are still having trouble getting a program to run after following the
instructions below, please open an issue.

The easiest way to get access to all `alpha-g-analysis` binaries is with
[`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Once `cargo` is installed, run the following command:

```bash
cargo install --locked alpha-g-analysis
```

Finally, to check that the installation was successful, run:

```bash
alpha-g-vertices --version
```

## Binaries

- [`alpha-g-chronobox-timestamps`](src/bin/alpha-g-chronobox-timestamps/README.md):
Extract the Chronobox timestamps for a single run.
- [`alpha-g-odb`](src/bin/alpha-g-odb/README.md):
Get an ODB dump from a MIDAS file.
- [`alpha-g-sequencer`](src/bin/alpha-g-sequencer/README.md):
Extract the sequencer data for a single run.
- [`alpha-g-trg-scalers`](src/bin/alpha-g-trg-scalers/README.md):
Extract the TRG scalers for a single run.
- [`alpha-g-vertices`](src/bin/alpha-g-vertices/README.md):
Reconstruct the annihilation vertices for a single run.
