/// Alpha16 DAQ boards.
///
/// There are a total of 8 Alpha16 boards; each digitizes the analog signals
/// from 16 channels of the Barrel Veto (SiPMs) and 32 channels of the radial
/// Time projection Chamber (anode wires).
pub mod alpha16;

/// MIDAS files.
///
/// Iterating through the contents of a MIDAS file can be done with the
/// [`midasio`](https://docs.rs/midasio) crate. This module defines ALPHA-g
/// specific objects to simplify handling MIDAS files.
pub mod midas;

#[cfg(test)]
mod tests;
