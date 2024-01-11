# Examples

The following examples show the most common operations that you will want to
perform on an ALPHA-g data file. Most of these use the
[`midasio`](https://github.com/MIDAS-rs/midasio) crate to read and iterate
through a MIDAS file. Additionally, the
[`alpha_g_detector`](https://github.com/ALPHA-g-Experiment/alpha-g/tree/main/detector)
library is used internally to handle the low-level details of the ALPHA-g data
banks.

- [vertices](./vertices.rs): Reconstruct the primary annihilation vertex of each
event in a MIDAS file.
