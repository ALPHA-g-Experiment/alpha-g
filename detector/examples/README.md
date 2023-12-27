# Examples

The following examples show the most common operations that you will want to
perform on an ALPHA-g data file. Most of these use the
[`midasio`](https://github.com/MIDAS-rs/midasio) crate to read and iterate
through a MIDAS file.

- [bv_adc](./bv_adc.rs): Iterate over all the Barrel Veto ADC waveforms from the
SiPMs.
- [tpc_adc](./tpc_adc.rs): Iterate over all the radial Time Projection Chamber
ADC waveforms from the anode wires.
- [tpc_pads](./tpc_pads.rs): Iterate over all the radial Time Projection Chamber 
cathode pad waveforms.
- [trg](./trg.rs): Iterate over all the trigger data packets from the TRG board.
