//! # Feed Validation
//!
//! Structural validation of an assembled [`GtfsReference`]: intra-record
//! conditional rules from the specification, primary-key uniqueness
//! and referential integrity across tables. Run it with
//! [`GtfsReference::validate`](crate::GtfsReference::validate), which
//! returns a [`ValidationReport`] listing every found
//! [`ValidationIssue`] - validation never stops at the first
//! problem.
//!
//! This is an embeddable pre-flight check for Rust pipelines, not a
//! replacement for the canonical
//! [MobilityData gtfs-validator](https://github.com/MobilityData/gtfs-validator),
//! which covers hundreds of rules including best practices.
//!
//! Module layout: the report types live in `report.rs`, the entry
//! point in `run.rs`, intra-record rules in `records.rs`,
//! referential integrity in `refs.rs` and dataset-wide rules
//! (uniqueness, cross-file conditionals) in `dataset.rs`.
//!
//! [`GtfsReference`]: crate::GtfsReference

mod dataset;
mod records;
mod refs;
mod report;
mod run;

pub use report::{Rule, Severity, ValidationIssue, ValidationReport};
