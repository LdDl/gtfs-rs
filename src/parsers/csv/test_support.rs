//! Shared helpers for the parser tests - the crate's "testutil".
//!
//! This module exists ONLY in test builds: it is declared as
//! `#[cfg(test)] pub(crate) mod test_support;` in `mod.rs`, so a
//! release build contains none of it - no code, no public API, no
//! rustdoc page. The Go analogy is a `testutil` package or a
//! `helpers_test.go` file.
//!
//! It lives under `csv` for historical reasons but serves the test
//! modules of `csv`, `feed` and `geojson` alike - hence the
//! `pub(crate)` visibility instead of private.
//!
//! What it provides:
//!
//! - absolute paths to the data fixtures under `tests/data/`, built
//!   from `CARGO_MANIFEST_DIR` (the crate root at compile time) so
//!   tests work regardless of the current working directory;
//! - small conversion helpers for test assertions.

use crate::error::GtfsError;
use crate::parsers::{ParseError, ParseErrorKind};

/// Root of the official Google sample feed fixture
/// (`tests/data/sample_feed/`): fixed-route data, 11 tables. See the
/// README in that directory for source and attribution.
pub const FEED_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/sample_feed");

/// Root of the hand-crafted GTFS-Flex fixture
/// (`tests/data/flex_feed/`): an on-demand feed with
/// `locations.geojson` zones and booking rules. See the README in
/// that directory.
pub const FLEX_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/flex_feed");

/// Returns the absolute path of one file inside the sample feed,
/// e.g. `feed_file("agency.txt")`.
pub fn feed_file(name: &str) -> String {
    format!("{}/{}", FEED_DIR, name)
}

/// Wraps a model-level [`GtfsError`] into a [`ParseError`] so test
/// code can use `?` on model constructors (e.g.
/// `GtfsDate::new(2026, 1, 1).map_err(model)?`) inside tests that
/// return `Result<(), ParseError>`.
pub fn model(e: GtfsError) -> ParseError {
    ParseError {
        file: "test".to_string(),
        line: 0,
        field: None,
        kind: ParseErrorKind::Model(e),
    }
}
