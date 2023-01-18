use crate::alpha16::{Adc32ChannelId, BoardId};
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

// These maps change whenever an Alpha16 board is replaced/moved.
//
// Maps from board_name -> (preamp_1, preamp_2) for specific run numbers.
// `preamp_1` corresponds to the first connector, `preamp_2` to the second.
// There are only preamplifiers on the top of the detector i.e. T0, ..., T15.
//
// When you add a new map, remember to:
//     - Add all 3 unit tests.
//     - Add the new lazy_static! for the map.
//     - Add the new map for the corresponding run number.
//
// Run 2941+ (including 2941):
const PREAMPS_2941: [(&str, (usize, usize)); 8] = [
    ("09", (0, 1)),
    ("10", (2, 3)),
    ("11", (4, 5)),
    ("12", (6, 7)),
    ("13", (8, 9)),
    ("14", (10, 11)),
    ("18", (12, 13)),
    ("16", (14, 15)),
];

fn preamps_map(map: [(&str, (usize, usize)); 8]) -> HashMap<BoardId, (usize, usize)> {
    let mut m = HashMap::new();
    for (board_name, preamps) in map.iter() {
        m.insert(BoardId::try_from(*board_name).unwrap(), *preamps);
    }
    m
}

lazy_static! {
    // Whenever a new map is added, add it here (without removing the old ones).
    static ref PREAMPS_MAP_2941: HashMap<BoardId, (usize, usize)> = preamps_map(PREAMPS_2941);
}

// These maps do not usually change.
// It only changes whenever there is a new revision of the Alpha16 boards.
//
// Maps Adc32ChannelId (index) -> wire channel within AW boards.
// The first 16 channels are for preamp_1, the second 16 for preamp_2.
//
// When you add a new map, remember to:
//    - Add unit test.
//    - Add the new map for the corresponding run number.
//
// Revision 1.1 was implemented in run 2724
const INV_CHANNELS_2724: [usize; 32] = [
    4, 2, 0, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15, 16, 18, 20, 22, 24, 26, 28, 30, 17, 19,
    21, 23, 25, 27, 29, 31,
];

/// The error type returned when mapping a [`BoardId`] and [`Adc32ChannelId`] to
/// a [`TpcWirePosition`] fails.
#[derive(Debug, Error)]
pub enum MapTpcWirePositionError {
    #[error("no rTPC preamp mapping available for run number {run_number}")]
    MissingPreampMap { run_number: u32 },
    #[error("alpha16 `{}` not found in map for run number {run_number}", board_id.name())]
    BoardIdNotFound { board_id: BoardId, run_number: u32 },
    #[error("no rTPC wire mapping available for run number {run_number}")]
    MissingWireMap { run_number: u32 },
}

/// Position of an anode wire in the TPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpcWirePosition(usize);
impl TpcWirePosition {
    /// Map a [`BoardId`] and [`Adc32ChannelId`] to a [`TpcWirePosition`] for a
    /// given run number. Returns an error if the mapping is not available for
    /// the given `run_number` or if the given [`BoardId`] is not installed for
    /// that `run_number`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::alpha16::{Adc32ChannelId, BoardId};
    /// use alpha_g_detector::alpha16::aw_map::TpcWirePosition;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("09")?;
    /// let channel_id = Adc32ChannelId::try_from(0)?;
    ///
    /// let position = TpcWirePosition::try_new(run_number, board_id, channel_id)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_new(
        run_number: u32,
        board_id: BoardId,
        channel_id: Adc32ChannelId,
    ) -> Result<Self, MapTpcWirePositionError> {
        // This map changes whenever a board is replaced/moved.
        let preamp_map = match run_number {
            2941.. => &PREAMPS_MAP_2941,
            _ => return Err(MapTpcWirePositionError::MissingPreampMap { run_number }),
        };
        // This map will rarely change. Needs new revision of Alpha16 boards.
        let channel_map = match run_number {
            2724.. => &INV_CHANNELS_2724,
            _ => return Err(MapTpcWirePositionError::MissingWireMap { run_number }),
        };
        // The logic below doesn't change even if a map above does.
        let (preamp_1, preamp_2) =
            preamp_map
                .get(&board_id)
                .ok_or(MapTpcWirePositionError::BoardIdNotFound {
                    board_id,
                    run_number,
                })?;
        let mapped_channel = channel_map[usize::from(channel_id.0)];
        let wire_position = match mapped_channel {
            0..=15 => preamp_1 * 16 + mapped_channel,
            16..=31 => preamp_2 * 16 + (mapped_channel - 16),
            _ => unreachable!(),
        };
        Ok(Self(wire_position))
    }
}

#[cfg(test)]
mod tests;