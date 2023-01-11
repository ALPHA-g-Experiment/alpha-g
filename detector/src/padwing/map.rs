use crate::padwing::BoardId;
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

/// Position of a Padwing board in the rTPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TpcPwbPosition {
    column: TpcPwbColumn,
    row: TpcPwbRow,
}
impl TpcPwbPosition {
    /// Return the column of the Padwing board within the rTPC.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alpha_g_detector::padwing::ParseBoardIdError;
    /// # use alpha_g_detector::padwing::map::MapTpcPwbPositionError;
    /// # use alpha_g_detector::padwing::map::TryPositionFromIndexError;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{tpc_pwb_position, TpcPwbColumn};
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("26")?;
    /// let position = tpc_pwb_position(run_number, board_id)?;
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
    /// # use alpha_g_detector::padwing::ParseBoardIdError;
    /// # use alpha_g_detector::padwing::map::MapTpcPwbPositionError;
    /// # use alpha_g_detector::padwing::map::TryPositionFromIndexError;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use alpha_g_detector::padwing::map::{tpc_pwb_position, TpcPwbRow};
    /// use alpha_g_detector::padwing::BoardId;
    ///
    /// let run_number = 5000;
    /// let board_id = BoardId::try_from("26")?;
    /// let position = tpc_pwb_position(run_number, board_id)?;
    ///
    /// assert_eq!(position.row(), TpcPwbRow::try_from(6)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn row(&self) -> TpcPwbRow {
        self.row
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
    static ref INV_PADWING_BOARDS_4418: HashMap<BoardId, TpcPwbPosition> =
        inverse_pwb_map(PADWING_BOARDS_4418);
}

/// The error type returned when mapping a [`BoardId`] to a [`TpcPwbPosition`]
/// fails.
#[derive(Debug, Error)]
pub enum MapTpcPwbPositionError {
    /// There is no mapping available for the given `run_number`.
    #[error("no mapping available for run number {run_number}")]
    MissingMap { run_number: u32 },
    /// The given [`BoardId`] is not in the map for the given `run_number`.
    #[error("pwb `{}` not found in map for run number {run_number}", board_id.name())]
    BoardIdNotFound { run_number: u32, board_id: BoardId },
}

/// Map a [`BoardId`] to a [`TpcPwbPosition`] for a given `run_number`.
/// Returns an error if there is no map available for the given `run_number` or
/// if the given [`BoardId`] is not in installed in the rTPC for the given
/// `run_number`.
///
/// # Examples
///
/// ```
/// # use alpha_g_detector::padwing::ParseBoardIdError;
/// # use alpha_g_detector::padwing::map::MapTpcPwbPositionError;
/// # use alpha_g_detector::padwing::map::TryPositionFromIndexError;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use alpha_g_detector::padwing::map::{tpc_pwb_position, TpcPwbColumn, TpcPwbRow};
/// use alpha_g_detector::padwing::BoardId;
///
/// let run_number = 5000;
/// let board_id = BoardId::try_from("26")?;
///
/// let position = tpc_pwb_position(run_number, board_id)?;
///
/// assert_eq!(position.column(), TpcPwbColumn::try_from(1)?);
/// assert_eq!(position.row(), TpcPwbRow::try_from(6)?);
/// # Ok(())
/// # }
pub fn tpc_pwb_position(
    run_number: u32,
    board_id: BoardId,
) -> Result<TpcPwbPosition, MapTpcPwbPositionError> {
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

#[cfg(test)]
mod tests;
