use lazy_static::lazy_static;
use serde::Deserialize;
use thiserror::Error;
use uom::si::f64::{Angle, Length, Time};

/// The error type returned when a drift time lookup fails.
#[derive(Debug, Error)]
pub enum TryDriftLookupError {
    #[error("drift time `{0:?}` is out of range")]
    DriftTimeOutOfRange(Time),
    #[error("axial position `{0:?}` is out of range")]
    AxialPositionOutOfRange(Length),
}

// Represents the radius and Lorentz correction as a function of drift time.
// The drift time is in ascending order. Given that this structure only exists
// in a static lookup table loaded at compile time, this is guaranteed by unit
// tests.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DriftTable(Vec<(Time, Length, Angle)>);

impl DriftTable {
    fn at(&self, t: Time) -> Result<(Length, Angle), TryDriftLookupError> {
        // Unit tests guarantee that the inner vector has at least 2 elements.
        if t < self.0[0].0 || t > self.0[self.0.len() - 1].0 {
            return Err(TryDriftLookupError::DriftTimeOutOfRange(t));
        }
        // Identify the two points that bracket the given drift time. A linear
        // interpolation is good enough.
        let rhs_index = self
            .0
            .iter()
            .position(|&(time, _, _)| time > t)
            // If t is the last element, then the last two elements bracket t.
            .unwrap_or(self.0.len() - 1);
        let lhs_index = rhs_index - 1;
        let (lhs_time, lhs_radius, lhs_correction) = self.0[lhs_index];
        let (rhs_time, rhs_radius, rhs_correction) = self.0[rhs_index];

        let fraction = (t - lhs_time) / (rhs_time - lhs_time);
        let radius = lhs_radius + fraction * (rhs_radius - lhs_radius);
        let correction = lhs_correction + Angle::from(fraction * (rhs_correction - lhs_correction));

        Ok((radius, correction))
    }
}

// The magnetic field is not uniform throughout the full detector length. Hence
// there is a different drift table for different `z` regions.
// Each drift table is coupled with the upper bound of the `z` region (from 0
// up to half the detector length). The negative `z` region is symmetric to the
// positive `z` region.
// Same as `DriftTable`, the `z` upper bound is in ascending order and checked
// by unit tests.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DriftTables(Vec<(DriftTable, Length)>);

impl DriftTables {
    pub(crate) fn at(&self, z: Length, t: Time) -> Result<(Length, Angle), TryDriftLookupError> {
        let z_abs = z.abs();
        if z_abs > self.0[self.0.len() - 1].1 {
            return Err(TryDriftLookupError::AxialPositionOutOfRange(z));
        }

        let (table, _) = self
            .0
            .iter()
            .find(|(_, z_upper_bound)| z_upper_bound >= &z_abs)
            .unwrap();

        table.at(t)
    }
}

const TABLE_BYTES: &[u8] =
    include_bytes!("../data/simulation/drift_table/drift_1T_70Ar_30CO2.json");

lazy_static! {
    pub(crate) static ref DRIFT_TABLES: DriftTables = serde_json::from_slice(TABLE_BYTES).unwrap();
}

#[cfg(test)]
mod tests;
