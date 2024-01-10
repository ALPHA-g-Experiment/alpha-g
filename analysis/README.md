# ALPHA-g-Analysis

[![Test Status](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/alpha-g/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/alpha-g-analysis?labelColor=383f47)](https://crates.io/crates/alpha-g-analysis)

Rust package with multiple binary crates. Each executable is a useful tool to 
perform common post-processing/offline analysis on the data of the ALPHA-g 
experiment at CERN.

## Installation

The package can be installed with
[`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```bash
cargo install alpha-g-analysis
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

### Other

- [`alpha-g-copy`](src/bin/alpha-g-copy/README.md): Make local copies of MIDAS 
files from remote hosts.

### TUI Visualization

- [`alpha-g-alpha16-signal-viewer`](src/bin/alpha-g-alpha16-signal-viewer/README.md): 
Visualize the ADC waveforms from the BV and the rTPC.
- [`alpha-g-padwing-signal-viewer`](src/bin/alpha-g-padwing-signal-viewer/README.md):
Visualize the cathode pad waveforms from the rTPC.
- [`alpha-g-tpc-occupancy-viewer`](src/bin/alpha-g-tpc-occupancy-viewer/README.md):
Visualize the anode wire and pad occupancy of the rTPC.
