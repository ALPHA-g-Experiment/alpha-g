/// Handle data output from Alpha16 DAQ boards.
///
/// There are a total of 8 Alpha16 boards; each digitizes the analog signals
/// from 16 channels of the Barrel Veto (SiPMs) and 32 channels of the radial
/// Time projection Chamber (anode wires).
pub mod alpha16;

#[cfg(test)]
mod tests;
