//! The [`CsvWrite`] trait: one GTFS entity serialized to one CSV
//! row, plus the RFC 4180 field escaping.

/// A GTFS entity writable as one CSV row.
///
/// Implemented by the crate for its `model` entities; implement it
/// for custom types to write GTFS extension tables with the same
/// machinery.
///
/// # Examples
///
/// ```
/// use gtfs_rs::writers::csv::{self, CsvWrite};
///
/// // a custom extension table: vehicle_types.txt
/// struct VehicleType {
///     vehicle_type_id: String,
///     description: Option<String>,
/// }
///
/// impl CsvWrite for VehicleType {
///     const FILE_NAME: &'static str = "vehicle_types.txt";
///     const HEADER: &'static [&'static str] = &["vehicle_type_id", "description"];
///
///     fn fields(&self) -> Vec<String> {
///         vec![
///             self.vehicle_type_id.clone(),
///             self.description.clone().unwrap_or_default(),
///         ]
///     }
/// }
///
/// fn main() -> Result<(), gtfs_rs::writers::WriteError> {
///     let types = vec![VehicleType {
///         vehicle_type_id: "bus12".to_string(),
///         description: None,
///     }];
///     let mut out = Vec::new();
///     csv::write("vehicle_types.txt", &types, &mut out)?;
///     let text = String::from_utf8_lossy(&out);
///     assert_eq!(text, "vehicle_type_id,description\nbus12,\n");
///     Ok(())
/// }
/// ```
pub trait CsvWrite {
    /// Dataset file the entity belongs to (e.g. "agency.txt").
    const FILE_NAME: &'static str;

    /// Every column of the table, in specification order.
    const HEADER: &'static [&'static str];

    /// Returns the field values in [`CsvWrite::HEADER`] order; empty
    /// strings encode absent optional values.
    fn fields(&self) -> Vec<String>;
}

/// Escapes one CSV field per RFC 4180: fields containing separators,
/// quotes or line breaks are quoted, with inner quotes doubled.
pub fn escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        let mut out = String::with_capacity(field.len() + 2);
        out.push('"');
        for ch in field.chars() {
            if ch == '"' {
                out.push('"');
            }
            out.push(ch);
        }
        out.push('"');
        out
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn test_escape() {
        assert_eq!(escape("plain"), "plain");
        assert_eq!(escape("Transit, Inc."), "\"Transit, Inc.\"");
        assert_eq!(escape("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(escape("two\nlines"), "\"two\nlines\"");
    }
}
