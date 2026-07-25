//! The [`CsvRecord`] trait: one GTFS entity per CSV row.

use crate::parsers::ParseError;
use crate::parsers::csv::Row;

/// A GTFS entity readable from one CSV row.
///
/// Implemented by the crate for its `model` entities; implement it
/// for custom types to read GTFS extensions with the same machinery.
///
/// # Examples
///
/// ```
/// use gtfs_rs::parsers::ParseError;
/// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
///
/// // a custom extension table: vehicle_types.txt
/// struct VehicleType {
///     vehicle_type_id: String,
///     description: Option<String>,
/// }
///
/// impl CsvRecord for VehicleType {
///     const FILE_NAME: &'static str = "vehicle_types.txt";
///
///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
///         Ok(VehicleType {
///             vehicle_type_id: row.req("vehicle_type_id")?.to_string(),
///             description: row.opt("description").map(str::to_string),
///         })
///     }
/// }
///
/// fn main() -> Result<(), ParseError> {
///     let data = "vehicle_type_id,description\nbus12,Low-floor bus\n";
///     let types: Vec<VehicleType> =
///         csv::read("vehicle_types.txt", data.as_bytes())?;
///     assert_eq!(types[0].vehicle_type_id, "bus12");
///     Ok(())
/// }
/// ```
pub trait CsvRecord: Sized {
    /// Dataset file the entity comes from (e.g. "agency.txt").
    const FILE_NAME: &'static str;

    /// Builds the entity from one CSV row.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] describing the offending field when
    /// the row violates the entity's format.
    fn from_row(row: &Row<'_>) -> Result<Self, ParseError>;
}
