//! # GTFS Errors

use std::fmt;

/// GTFS data errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GtfsError {
    /// A time value is not in the HH:MM:SS format.
    InvalidTime {
        /// The rejected input string
        value: String,
    },
    /// A date value is not a valid YYYYMMDD calendar date.
    InvalidDate {
        /// The rejected input string
        value: String,
    },
    /// A currency amount is not a valid decimal number.
    InvalidCurrencyAmount {
        /// The rejected input string
        value: String,
    },
}

impl fmt::Display for GtfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GtfsError::InvalidTime { value } => {
                write!(
                    f,
                    "invalid GTFS time value: '{}' (expected HH:MM:SS)",
                    value
                )
            }
            GtfsError::InvalidDate { value } => {
                write!(
                    f,
                    "invalid GTFS date value: '{}' (expected YYYYMMDD)",
                    value
                )
            }
            GtfsError::InvalidCurrencyAmount { value } => {
                write!(
                    f,
                    "invalid GTFS currency amount: '{}' (expected a decimal number)",
                    value
                )
            }
        }
    }
}

impl std::error::Error for GtfsError {}
