# ALPHA-g-Analysis

[![Test Status](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml/badge.svg)](https://github.com/ALPHA-g-Experiment/alpha-g/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/alpha-g-analysis?labelColor=383f47)](https://crates.io/crates/alpha-g-analysis)

Rust package with multiple binary crates.

## Getting Started

The core analysis programs should work out of the box on any platform. Some of
the other programs (calibration, TUI visualization, etc.) may require additional
dependencies (see below). If you are still having trouble getting a program
to run after following the instructions below, please open an issue.

The easiest way to get access to all `alpha-g-analysis` binaries is with
[`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Once `cargo` is installed, run the following command:

```bash
cargo install alpha-g-analysis
```

Finally, to check that the installation was successful, run:

```bash
alpha-g-vertices --version
```

## System Dependencies

The following is a complete list of dependencies required to run all of the
`alpha-g-analysis` binaries. To see which dependencies are required for a
specific program, see its README.

- ncurses: This is the default backend used for the TUI. In the very unlikely
 scenario that you don't have this already, see [how to install
ncurses](https://github.com/gyscos/cursive/wiki/Install-ncurses).
- pdflatex: Access to the `pdflatex` command. Additionally, the `standalone` and
`pgfplots` packages are required to generate the plots. To make sure that these
are installed properly, run:
	```
	pdflatex "\documentclass{standalone}\usepackage{pgfplots}\begin{document}\begin{tikzpicture}\begin{axis}\end{axis}\end{tikzpicture}\end{document}"
	```
	If successful, this will create a PDF file with an empty plot in the current
working directory.
- PDF viewer: Install a PDF viewer which refreshes the output whenever a
 document is re-compiled. There are multiple viewers that support this feature;
all are equally valid. I personally use
[zathura](https://wiki.archlinux.org/title/zathura).
- Default viewer: To make sure that the correct PDF viewer is used to open the
plots, run:
	```
	xdg-open any_pdf_document_on_your_system.pdf
	```
	This should open the PDF with the viewer from the previous point. Otherwise,
use `xdg-mime` to configure the appropriate default PDF viewer. [Example
instructions for
zathura](https://wiki.archlinux.org/title/zathura#Make_zathura_the_default_pdf_viewer).

## Binaries

### Analysis

- [`alpha-g-trg-scalers`](src/bin/alpha-g-trg-scalers/README.md):
Extract the TRG scalers for a single run.
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

- [`alpha-g-tpc-occupancy-viewer`](src/bin/alpha-g-tpc-occupancy-viewer/README.md):
Visualize the anode wire and pad occupancy of the rTPC.

### Other

- [`alpha-g-copy`](src/bin/alpha-g-copy/README.md): Make local copies of MIDAS 
files from remote hosts.
