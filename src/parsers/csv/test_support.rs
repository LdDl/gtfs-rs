//! Shared helpers for the CSV parser tests.

use crate::error::GtfsError;
use crate::parsers::{ParseError, ParseErrorKind};

/// Root of the official sample feed fixture (see the README in that
/// directory for attribution).
pub const FEED_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/sample_feed");

/// Returns the path of one sample feed file.
pub fn feed_file(name: &str) -> String {
    format!("{}/{}", FEED_DIR, name)
}

/// Wraps a model-level error for use in test assertions.
pub fn model(e: GtfsError) -> ParseError {
    ParseError {
        file: "test".to_string(),
        line: 0,
        field: None,
        kind: ParseErrorKind::Model(e),
    }
}
