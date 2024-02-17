/// Alpha16 DAQ boards.
///
/// There are a total of 8 Alpha16 boards; each digitizes the analog signals
/// from 16 channels of the Barrel Veto (SiPMs) and 32 channels of the radial
/// Time projection Chamber (anode wires).
pub mod alpha16;

/// PadWing DAQ boards.
///
/// There are a total of 64 PadWing boards (8 rows, 8 columns); each with 4
/// AFTER chips (A, B, C, and D). Every chip reads out 72 pads, for a total
/// of 18432 cathode pads in the radial Time Projection Chamber.
pub mod padwing;

/// MIDAS files.
///
/// Iterating through the contents of a MIDAS file can be done with the
/// [`midasio`](https://docs.rs/midasio) crate. This module defines ALPHA-g
/// specific characteristics to simplify interpreting MIDAS files.
pub mod midas;

/// Trigger DAQ board.
///
/// There is a single Trigger board which collects information from the anode
/// wires and SiPMs to make real-time decisions about recording an event to
/// disk.
pub mod trigger;

/// Chronobox.
///
/// There are multiple Chronoboxes, each with 59 input channels plus a system
/// clock channel.
pub mod chronobox;

#[cfg(test)]
mod tests;
