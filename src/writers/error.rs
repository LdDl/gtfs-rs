//! # Write Errors
//!
//! Failures while serializing a feed, carrying the name of the file
//! being written.

use std::fmt;
use std::io;

/// Where and why writing failed.
#[derive(Debug)]
#[non_exhaustive]
pub struct WriteError {
    /// File being written when the error occurred
    /// (e.g. "stops.txt")
    pub file: String,
    /// What exactly went wrong
    pub kind: WriteErrorKind,
}

/// The failure kinds of [`WriteError`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WriteErrorKind {
    /// Writing to the underlying target failed
    Io(io::Error),
    /// The feed archive could not be assembled
    #[cfg(feature = "zip")]
    Zip(zip::result::ZipError),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        match &self.kind {
            WriteErrorKind::Io(e) => write!(f, ": {}", e),
            #[cfg(feature = "zip")]
            WriteErrorKind::Zip(e) => write!(f, ": invalid zip archive: {}", e),
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            WriteErrorKind::Io(e) => Some(e),
            #[cfg(feature = "zip")]
            WriteErrorKind::Zip(e) => Some(e),
        }
    }
}
