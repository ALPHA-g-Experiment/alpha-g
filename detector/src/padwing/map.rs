use crate::padwing::{AfterId, BoardId, PadChannelId};
use lazy_static::lazy_static;
use std::collections::HashMap;
use thiserror::Error;

/// The error type returned when conversion from [`usize`] to Row or Column
/// fails.
#[derive(Debug, Error)]
#[error("unknown conversion from {input} to row or column")]
pub struct TryPositionFromIndexError {
    input: usize,
}

/// Column of a Padwing board in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpcPwbColumn(usize);
impl TryFrom<usize> for TpcPwbColumn {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=7`) to a [`TpcPwbColumn`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 8 {
            Ok(Self(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

/// Row of a Padwing board in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpcPwbRow(usize);
impl TryFrom<usize> for TpcPwbRow {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=7`) to a [`TpcPwbRow`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 8 {
            Ok(Self(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

// Map of all PWB boards as installed on the rTPC in run number 4418 (included).
// First index is column, second index is row.
// The value is the board name.
//
// When you add a new map, remember to add the unit tests for it:
//     - Uniqueness of all &str.
//     - Validity of all &str.
//     - Test inverse map.
//
// Also remember to add the inverse (actually needed) map to the lazy_static
// below and update TpcPwbPosition::try_new.
const PADWING_BOARDS_4418: [[&str; 8]; 8] = [
    ["12", "13", "14", "02", "11", "17", "18", "19"],
    ["20", "21", "22", "23", "24", "25", "26", "27"],
    ["46", "29", "08", "77", "10", "33", "34", "35"],
    ["36", "37", "01", "39", "76", "41", "42", "40"],
    ["44", "49", "07", "78", "03", "04", "45", "15"],
    ["52", "53", "54", "55", "56", "57", "58", "05"],
    ["60", "00", "06", "63", "64", "65", "66", "67"],
    ["68", "69", "70", "71", "72", "73", "74", "75"],
];

fn inverse_pwb_map(map: [[&str; 8]; 8]) -> HashMap<BoardId, TpcPwbPosition> {
    let mut inverse = HashMap::new();
    for (column, row) in map.iter().enumerate() {
        for (row, name) in row.iter().enumerate() {
            inverse.insert(
                // Safe to unwrap. Unit tests should validate that this cant fail.
                BoardId::try_from(*name).unwrap(),
                TpcPwbPosition {
                    column: TpcPwbColumn::try_from(column).unwrap(),
                    row: TpcPwbRow::try_from(row).unwrap(),
                },
            );
        }
    }
    inverse
}

lazy_static! {
    // Whenever a new map is added, just add it to this list.
    static ref INV_PADWING_BOARDS_4418: HashMap<BoardId, TpcPwbPosition> =
        inverse_pwb_map(PADWING_BOARDS_4418);
}

/// The error type returned when mapping a [`BoardId`] to a [`TpcPwbPosition`]
/// fails.
#[derive(Debug, Error)]
pub enum MapTpcPwbPositionError {
    /// There is no mapping available for the given `run_number`.
    #[error("no rTPC PWB mapping available for run number {run_number}")]
    MissingMap { run_number: u32 },
    /// The given [`BoardId`] is not in the map for the given `run_number`.
    #[error("pwb `{}` not found in map for run number {run_number}", board_id.name())]
    BoardIdNotFound { run_number: u32, board_id: BoardId },
}

/// Position of a Padwing board in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TpcPwbPosition {
    column: TpcPwbColumn,
    row: TpcPwbRow,
}
impl TpcPwbPosition {
    /// Map a [`BoardId`] to a [`TpcPwbPosition`] for a given `run_number`.
    /// Returns an error if there is no map available for the given `run_number`
    /// or if the given [`BoardId`] is not installed in the rTPC for that
    /// `run_number`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::TpcPwbPosition;
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("26")?;
    ///
    /// let position = TpcPwbPosition::try_new(run_number, board_id)?;
    /// # Ok(())
    /// # }
    pub fn try_new(run_number: u32, board_id: BoardId) -> Result<Self, MapTpcPwbPositionError> {
        let position_map = match run_number {
            4418.. => &INV_PADWING_BOARDS_4418,
            _ => return Err(MapTpcPwbPositionError::MissingMap { run_number }),
        };

        position_map
            .get(&board_id)
            .copied()
            .ok_or(MapTpcPwbPositionError::BoardIdNotFound {
                run_number,
                board_id,
            })
    }
    /// Return the column of the Padwing board within the rTPC.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{TpcPwbPosition, TpcPwbColumn};
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("26")?;
    /// let position = TpcPwbPosition::try_new(run_number, board_id)?;
    ///
    /// assert_eq!(position.column(), TpcPwbColumn::try_from(1)?);
    /// # Ok(())
    /// # }
    pub fn column(&self) -> TpcPwbColumn {
        self.column
    }
    /// Return the row of the Padwing board within the rTPC.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{TpcPwbPosition, TpcPwbRow};
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("26")?;
    /// let position = TpcPwbPosition::try_new(run_number, board_id)?;
    ///
    /// assert_eq!(position.row(), TpcPwbRow::try_from(6)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn row(&self) -> TpcPwbRow {
        self.row
    }
}

/// Column of a pad in a Padwing Board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PwbPadColumn(usize);
impl TryFrom<usize> for PwbPadColumn {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=3`) to a [`PwbPadColumn`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 4 {
            Ok(Self(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

/// Row of a pad in a Padwing Board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PwbPadRow(usize);
impl TryFrom<usize> for PwbPadRow {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=71`) to a [`PwbPadRow`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 72 {
            Ok(Self(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

// I don't see the following mapping between (AFTER, channel) -> Position
// changing or being updated any time soon. It would imply an excessive amount
// of hardware work. Nonetheless, I am leaving this mapping as a function of
// `run_number` to be consistent with the anode wire mapping. If it changes at
// some point, just do the same as the above PWB mapping or the anode wire
// mapping.
lazy_static! {
    // Map copied directly from agana/Feam.hh written by K.O.
    static ref INV_PADS_0: HashMap<(AfterId, PadChannelId), PwbPadPosition> = {
        let mut inverse = HashMap::new();
        for after in 0..=3u8 {
            let offset = (after % 2) * 36;
            for channel in 1..=72u8 {
                let mut col: u8;
                let mut row: u8;
                match channel {
                    0..=18 => {
                        col = 1;
                        row = channel - 1 + offset;
                    },
                    19..=36 => {
                        col = 0;
                        row = 36 - channel + offset;
                    },
                    37..=54 => {
                        col = 0;
                        row = 72 - channel + offset;
                    },
                    55..=72 => {
                        col = 1;
                        row = channel - 37 + offset;
                    }
                    _ => unreachable!(),
                }
                if after > 1 {
                    col = 3 - col;
                    row = 71 - row;
                }
                inverse.insert(
                    (
                        AfterId::try_from(after).unwrap(),
                        PadChannelId::try_from(u16::from(channel)).unwrap(),
                    ),
                    PwbPadPosition {
                        column: PwbPadColumn::try_from(usize::from(col)).unwrap(),
                        row: PwbPadRow::try_from(usize::from(row)).unwrap(),
                    },
                );
            }
        }
        inverse
    };
}

/// The error type returned when mapping an [`AfterId`] and [`PadChannelId`] to a
/// [`PwbPadPosition`] fails.
#[derive(Debug, Error)]
#[error("no PWB pad mapping available for run number {run_number}")]
pub struct MapPwbPadPositionError {
    run_number: u32,
}

/// Position of a pad in a Padwing Board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PwbPadPosition {
    column: PwbPadColumn,
    row: PwbPadRow,
}
impl PwbPadPosition {
    /// Map an [`AfterId`] and [`PadChannelId`] to a [`PwbPadPosition`] for a
    /// given `run_number`. Returns an error if there is no map available for
    /// that `run_number`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::PwbPadPosition;
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId};
    ///
    /// let run_number = 5000;
    /// let after_id = AfterId::try_from('A')?;
    /// let pad_channel_id = PadChannelId::try_from(1)?;
    ///
    /// let position = PwbPadPosition::try_new(run_number, after_id, pad_channel_id)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_new(
        _run_number: u32,
        after_id: AfterId,
        pad_channel_id: PadChannelId,
    ) -> Result<PwbPadPosition, MapPwbPadPositionError> {
        let position_map = &INV_PADS_0;
        Ok(*position_map.get(&(after_id, pad_channel_id)).unwrap())
    }
    /// Return the column of the pad within the Padwing Board.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{PwbPadPosition, PwbPadColumn};
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId};
    ///
    /// let run_number = 5000;
    /// let after_id = AfterId::try_from('A')?;
    /// let pad_channel_id = PadChannelId::try_from(1)?;
    ///
    /// let position = PwbPadPosition::try_new(run_number, after_id, pad_channel_id)?;
    ///
    /// assert_eq!(position.column(), PwbPadColumn::try_from(1)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn column(&self) -> PwbPadColumn {
        self.column
    }
    /// Return the row of the pad within the Padwing Board.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{PwbPadPosition, PwbPadRow};
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId};
    ///
    /// let run_number = 5000;
    /// let after_id = AfterId::try_from('A')?;
    /// let pad_channel_id = PadChannelId::try_from(1)?;
    ///
    /// let position = PwbPadPosition::try_new(run_number, after_id, pad_channel_id)?;
    ///
    /// assert_eq!(position.row(), PwbPadRow::try_from(0)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn row(&self) -> PwbPadRow {
        self.row
    }
}

/// Column of a pad in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpcPadColumn(usize);
impl TryFrom<usize> for TpcPadColumn {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=31`) to a [`TpcPadColumn`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 32 {
            Ok(TpcPadColumn(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

/// Row of a pad in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpcPadRow(usize);
impl TryFrom<usize> for TpcPadRow {
    type Error = TryPositionFromIndexError;

    /// Convert from a `usize` (`0..=575`) to a [`TpcPadRow`].
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 576 {
            Ok(TpcPadRow(value))
        } else {
            Err(TryPositionFromIndexError { input: value })
        }
    }
}

/// Position of a pad in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TpcPadPosition {
    column: TpcPadColumn,
    row: TpcPadRow,
}
impl TpcPadPosition {
    /// Map a [`TpcPwbPosition`] and [`PwbPadPosition`] to a [`TpcPadPosition`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{TpcPadPosition, TpcPwbPosition, PwbPadPosition};
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId, BoardId};
    ///
    /// let run_number = 5000;
    /// let board = BoardId::try_from("26")?;
    /// let board_pos = TpcPwbPosition::try_new(run_number, board)?;
    ///
    /// let after = AfterId::try_from('A')?;
    /// let pad_channel = PadChannelId::try_from(1)?;
    /// let pad_pos = PwbPadPosition::try_new(run_number, after, pad_channel)?;
    ///
    /// let tpc_pad_position = TpcPadPosition::new(board_pos, pad_pos);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(board_position: TpcPwbPosition, pad_position: PwbPadPosition) -> Self {
        let TpcPwbPosition { column, row } = board_position;
        let PwbPadPosition {
            column: pad_column,
            row: pad_row,
        } = pad_position;
        let column = TpcPadColumn::try_from(column.0 * 4 + pad_column.0).unwrap();
        let row = TpcPadRow::try_from(row.0 * 72 + pad_row.0).unwrap();
        TpcPadPosition { column, row }
    }
    /// Return the column of the pad within the rTPC.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{TpcPadPosition, TpcPadColumn, TpcPwbPosition, PwbPadPosition};
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId, BoardId};
    ///
    /// let run_number = 5000;
    /// let board = BoardId::try_from("26")?;
    /// let board_pos = TpcPwbPosition::try_new(run_number, board)?;
    ///
    /// let after = AfterId::try_from('A')?;
    /// let pad_channel = PadChannelId::try_from(1)?;
    /// let pad_pos = PwbPadPosition::try_new(run_number, after, pad_channel)?;
    ///
    /// let tpc_pad_position = TpcPadPosition::new(board_pos, pad_pos);
    ///
    /// assert_eq!(tpc_pad_position.column(), TpcPadColumn::try_from(5)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn column(&self) -> TpcPadColumn {
        self.column
    }
    /// Return the row of the pad within the rTPC.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{TpcPadPosition, TpcPadRow, TpcPwbPosition, PwbPadPosition};
    /// use alpha_g_detector::padwing::{AfterId, PadChannelId, BoardId};
    ///
    /// let run_number = 5000;
    /// let board = BoardId::try_from("26")?;
    /// let board_pos = TpcPwbPosition::try_new(run_number, board)?;
    ///
    /// let after = AfterId::try_from('A')?;
    /// let pad_channel = PadChannelId::try_from(1)?;
    /// let pad_pos = PwbPadPosition::try_new(run_number, after, pad_channel)?;
    ///
    /// let tpc_pad_position = TpcPadPosition::new(board_pos, pad_pos);
    ///
    /// assert_eq!(tpc_pad_position.row(), TpcPadRow::try_from(432)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn row(&self) -> TpcPadRow {
        self.row
    }
}

#[cfg(test)]
mod tests;
