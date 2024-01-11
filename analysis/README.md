# ALPHA-g-Analysis

[![Test Status](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/alpha-g-analysis?labelColor=383f47)](https://crates.io/crates/alpha-g-analysis)

Rust package with multiple binary crates.

## Getting Started

The core analysis programs should work out of the box on any platform. Some of
the other programs (calibration, TUI visualization, etc.) may require additional
dependencies (see below). If you are still having trouble getting a program
to run after following the instructions below, please open an issue.

The easiest way to install all `alpha-g-analysis` binaries is with
[`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Once `cargo` is installed, run the following command to install the package:

```bash
cargo install alpha-g-analysis
```

Finally, to check that the installation was successful, run:

```bash
alpha-g-vertices --version
```

## Binaries

### Analysis

- [`alpha-g-trg-scalers`](src/bin/alpha-g-trg-scalers/README.md):
Visualize the rate of the TRG scalers for a single run.
- [`alpha-g-vertices`](src/bin/alpha-g-vertices/README.md):
Reconstruct the annihilation vertices for a single run.

### Calibration

#### Anode Wires

- [`alpha-g-wire-noise-statistics`](src/bin/alpha-g-wire-noise-statistics/README.md):
Statistical analysis of the anode wire signals during a noise run.
- [`alpha-g-wire-baseline-comparison`](src/bin/alpha-g-wire-baseline-comparison/README.md):
Compare anode wire baseline calibration files to determine if there is a
statistically significant difference between them.
- [`alpha-g-wire-gain-calibration`](src/bin/alpha-g-wire-gain-calibration/README.md):
Gain calibration of the anode wire signals.

#### Pads
- [`alpha-g-pad-noise-statistics`](src/bin/alpha-g-pad-noise-statistics/README.md):
Statistical analysis of the pad signals during a noise run.
- [`alpha-g-pad-baseline-comparison`](src/bin/alpha-g-pad-baseline-comparison/README.md):
Compare pad baseline calibration files to determine if there is a statistically
significant difference between them.
- [`alpha-g-pad-gain-calibration`](src/bin/alpha-g-pad-gain-calibration/README.md):
Gain calibration of the cathode pad signals.

### TUI Visualization

- [`alpha-g-alpha16-signal-viewer`](src/bin/alpha-g-alpha16-signal-viewer/README.md): 
Visualize the ADC waveforms from the BV and the rTPC.
- [`alpha-g-padwing-signal-viewer`](src/bin/alpha-g-padwing-signal-viewer/README.md):
Visualize the cathode pad waveforms from the rTPC.
- [`alpha-g-tpc-occupancy-viewer`](src/bin/alpha-g-tpc-occupancy-viewer/README.md):
Visualize the anode wire and pad occupancy of the rTPC.

### Other

- [`alpha-g-copy`](src/bin/alpha-g-copy/README.md): Make local copies of MIDAS 
files from remote hosts.
