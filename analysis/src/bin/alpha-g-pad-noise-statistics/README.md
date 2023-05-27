# `alpha-g-pad-noise-statistics`

Statistical analysis of the pad signals during a noise run. Generate
(among other things) a calibration file with the baseline of all pad channels.
Run the `alpha-g-pad-noise-statistics --help` command to make sure you have
installed the `alpha-g-analysis` package and print help information.

## Requirements

All input MIDAS files should belong to the same run with the following settings:
- Pulser enabled.
- Field wire pulser disabled.
- BSC pulser disabled.
- Trigger pulser must be enabled.
- There should be no other active trigger sources.
- Pad data suppression must be disabled.
