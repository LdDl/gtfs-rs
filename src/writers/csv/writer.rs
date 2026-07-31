//! Writing functions: one CSV table into any writer or to a file
//! path.

use std::fs::File;
use std::io::{self, Write as _};
use std::path::Path;

use crate::writers::csv::table::{CsvWrite, escape};
use crate::writers::{WriteError, WriteErrorKind};

/// Writes all records of one GTFS table into any writer, header
/// included.
///
/// # Arguments
///
/// * `file_label` - Name used in error messages (e.g. "agency.txt")
/// * `rows` - The records to write
/// * `out` - Destination of the CSV bytes
///
/// # Errors
///
/// Returns a [`WriteError`] when the underlying writer fails.
///
/// # Examples
///
/// ```
/// use gtfs_rs::Agency;
/// use gtfs_rs::writers::{WriteError, csv};
///
/// fn main() -> Result<(), WriteError> {
///     let agencies = vec![
///         Agency::new("Demo", "https://demo.example", "Europe/Moscow"),
///     ];
///     let mut out = Vec::new();
///     csv::write("agency.txt", &agencies, &mut out)?;
///     let text = String::from_utf8_lossy(&out);
///     assert!(text.starts_with("agency_id,agency_name,"));
///     assert!(text.contains("Demo"));
///     Ok(())
/// }
/// ```
pub fn write<T: CsvWrite, W: io::Write>(
    file_label: &str,
    rows: &[T],
    out: W,
) -> Result<(), WriteError> {
    let mut out = io::BufWriter::new(out);
    let io_err = |e: io::Error| WriteError {
        file: file_label.to_string(),
        kind: WriteErrorKind::Io(e),
    };

    writeln!(out, "{}", T::HEADER.join(",")).map_err(io_err)?;
    for row in rows {
        let fields = row.fields();
        debug_assert_eq!(fields.len(), T::HEADER.len());
        let mut first = true;
        for field in &fields {
            if !first {
                write!(out, ",").map_err(io_err)?;
            }
            first = false;
            write!(out, "{}", escape(field)).map_err(io_err)?;
        }
        writeln!(out).map_err(io_err)?;
    }
    out.flush().map_err(io_err)
}

/// Writes one GTFS table to a file path, creating or overwriting the
/// file.
///
/// The table kind is decided by the element type of `rows`; the path
/// may have any file name (the name goes into error messages).
///
/// # Arguments
///
/// * `rows` - The records to write
/// * `path` - Destination file path
///
/// # Errors
///
/// Returns a [`WriteError`] if the file cannot be created or written.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::Agency;
/// use gtfs_rs::writers::{WriteError, csv};
///
/// fn main() -> Result<(), WriteError> {
///     let agencies = vec![
///         Agency::new("Demo", "https://demo.example", "Europe/Moscow"),
///     ];
///     csv::write_path(&agencies, "out/agency.txt")?;
///     Ok(())
/// }
/// ```
pub fn write_path<T: CsvWrite>(rows: &[T], path: impl AsRef<Path>) -> Result<(), WriteError> {
    let path = path.as_ref();
    let label = path.display().to_string();
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(WriteError {
                file: label,
                kind: WriteErrorKind::Io(e),
            });
        }
    };
    write(&label, rows, file)
}
