//! The [`Row`] accessor handed to record implementations:
//! by-name column lookups with typed helpers that attach full
//! error context (file, line, field).

use std::collections::HashMap;

use crate::misc::{CurrencyAmount, GtfsDate, parse_gtfs_time};
use crate::parsers::{ParseError, ParseErrorKind};

/// Reads an optional string column into an owned value.
pub(crate) fn opt_string(row: &Row<'_>, name: &str) -> Option<String> {
    row.opt(name).map(str::to_string)
}

/// One CSV row with header context, handed to
/// [`CsvRecord::from_row`](crate::parsers::csv::CsvRecord::from_row)
/// implementations.
///
/// Lookups are by column name; positions in the file are irrelevant.
/// The accessors build [`ParseError`]s that already carry the file
/// name, line number and field name.
///
/// # Examples
///
/// ```
/// use gtfs_rs::parsers::ParseError;
/// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
///
/// struct StopName {
///     name: Option<String>,
/// }
///
/// impl CsvRecord for StopName {
///     const FILE_NAME: &'static str = "stops.txt";
///
///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
///         Ok(StopName {
///             name: row.opt("stop_name").map(str::to_string),
///         })
///     }
/// }
///
/// fn main() -> Result<(), ParseError> {
///     let data = "stop_id,stop_name\nA,Central\nB,\n";
///     let rows: Vec<StopName> = csv::read("stops.txt", data.as_bytes())?;
///     assert_eq!(rows[0].name.as_deref(), Some("Central"));
///     assert!(rows[1].name.is_none()); // empty value -> None
///     Ok(())
/// }
/// ```
pub struct Row<'a> {
    pub(super) file: &'a str,
    pub(super) line: u64,
    pub(super) header: &'a HashMap<String, usize>,
    pub(super) record: &'a csv::StringRecord,
}

impl Row<'_> {
    /// Returns the trimmed value of an optional column, or `None`
    /// when the column is absent or the value is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Desc {
    ///     desc: Option<String>,
    /// }
    ///
    /// impl CsvRecord for Desc {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Desc {
    ///             desc: row.opt("stop_desc").map(str::to_string),
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     // the stop_desc column does not exist at all
    ///     let data = "stop_id\nA\n";
    ///     let rows: Vec<Desc> = csv::read("stops.txt", data.as_bytes())?;
    ///     assert!(rows[0].desc.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt(&self, name: &str) -> Option<&str> {
        let index = *self.header.get(name)?;
        match self.record.get(index).map(str::trim) {
            None | Some("") => None,
            Some(value) => Some(value),
        }
    }

    /// Returns the trimmed value of a required column.
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`] if the header lacks
    /// the column, or [`ParseErrorKind::EmptyValue`] if the value is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Id {
    ///     id: String,
    /// }
    ///
    /// impl CsvRecord for Id {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Id {
    ///             id: row.req("stop_id")?.to_string(),
    ///         })
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_name\nCentral\n";
    ///     match csv::read::<Id, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected a missing-column error"),
    ///         Err(e) => {
    ///             assert_eq!(e.field.as_deref(), Some("stop_id"));
    ///             assert!(matches!(e.kind, ParseErrorKind::MissingColumn));
    ///         }
    ///     }
    /// }
    /// ```
    pub fn req(&self, name: &str) -> Result<&str, ParseError> {
        if !self.header.contains_key(name) {
            return Err(self.err(name, ParseErrorKind::MissingColumn));
        }
        match self.opt(name) {
            Some(value) => Ok(value),
            None => Err(self.err(name, ParseErrorKind::EmptyValue)),
        }
    }

    /// Parses an optional integer-coded enum column with the given
    /// `from_code` converter (every crate enum provides one).
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    /// * `from_code` - Converter, e.g. `LocationType::from_code`
    /// * `expected` - Description for error messages, e.g. "code 0-4"
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Invalid`] if the value is not an
    /// integer or the code is rejected by the converter.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::LocationType;
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Kind {
    ///     location_type: Option<LocationType>,
    /// }
    ///
    /// impl CsvRecord for Kind {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Kind {
    ///             location_type: row.opt_code(
    ///                 "location_type",
    ///                 LocationType::from_code,
    ///                 "code 0-4",
    ///             )?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "stop_id,location_type\nA,1\nB,\n";
    ///     let rows: Vec<Kind> = csv::read("stops.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].location_type, Some(LocationType::Station));
    ///     assert!(rows[1].location_type.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_code<T>(
        &self,
        name: &str,
        from_code: fn(i32) -> Option<T>,
        expected: &str,
    ) -> Result<Option<T>, ParseError> {
        let raw = match self.opt(name) {
            None => return Ok(None),
            Some(raw) => raw,
        };
        let code: i32 = match raw.parse() {
            Ok(code) => code,
            Err(_) => return Err(self.invalid(name, raw, expected)),
        };
        match from_code(code) {
            Some(value) => Ok(Some(value)),
            None => Err(self.invalid(name, raw, expected)),
        }
    }

    /// Parses a required integer-coded enum column with the given
    /// `from_code` converter.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    /// * `from_code` - Converter, e.g. `RouteType::from_code`
    /// * `expected` - Description for error messages
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Invalid`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::RouteType;
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Kind {
    ///     route_type: RouteType,
    /// }
    ///
    /// impl CsvRecord for Kind {
    ///     const FILE_NAME: &'static str = "routes.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Kind {
    ///             route_type: row.req_code(
    ///                 "route_type",
    ///                 RouteType::from_code,
    ///                 "a route type code",
    ///             )?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "route_id,route_type\nL1,3\n";
    ///     let rows: Vec<Kind> = csv::read("routes.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].route_type, RouteType::Bus);
    ///     Ok(())
    /// }
    /// ```
    pub fn req_code<T>(
        &self,
        name: &str,
        from_code: fn(i32) -> Option<T>,
        expected: &str,
    ) -> Result<T, ParseError> {
        let raw = self.req(name)?;
        let code: i32 = match raw.parse() {
            Ok(code) => code,
            Err(_) => return Err(self.invalid(name, raw, expected)),
        };
        match from_code(code) {
            Some(value) => Ok(value),
            None => Err(self.invalid(name, raw, expected)),
        }
    }

    /// Parses an optional numeric column into any [`std::str::FromStr`]
    /// number type (`u32`, `i32`, `f64`, ...).
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    /// * `expected` - Description for error messages
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Invalid`] if the value fails to
    /// parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Dist {
    ///     traveled: Option<f64>,
    /// }
    ///
    /// impl CsvRecord for Dist {
    ///     const FILE_NAME: &'static str = "shapes.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Dist {
    ///             traveled: row.opt_num("shape_dist_traveled", "a distance")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "shape_id,shape_dist_traveled\nsh,5.25\nsh,\n";
    ///     let rows: Vec<Dist> = csv::read("shapes.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].traveled, Some(5.25));
    ///     assert!(rows[1].traveled.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_num<T: std::str::FromStr>(
        &self,
        name: &str,
        expected: &str,
    ) -> Result<Option<T>, ParseError> {
        let raw = match self.opt(name) {
            None => return Ok(None),
            Some(raw) => raw,
        };
        match raw.parse() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(self.invalid(name, raw, expected)),
        }
    }

    /// Parses a required numeric column into any
    /// [`std::str::FromStr`] number type.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    /// * `expected` - Description for error messages
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Invalid`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Seq {
    ///     stop_sequence: u32,
    /// }
    ///
    /// impl CsvRecord for Seq {
    ///     const FILE_NAME: &'static str = "stop_times.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Seq {
    ///             stop_sequence: row.req_num("stop_sequence", "a sequence")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "trip_id,stop_sequence\nt0,5\n";
    ///     let rows: Vec<Seq> = csv::read("stop_times.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].stop_sequence, 5);
    ///     Ok(())
    /// }
    /// ```
    pub fn req_num<T: std::str::FromStr>(
        &self,
        name: &str,
        expected: &str,
    ) -> Result<T, ParseError> {
        let raw = self.req(name)?;
        match raw.parse() {
            Ok(value) => Ok(value),
            Err(_) => Err(self.invalid(name, raw, expected)),
        }
    }

    /// Parses an optional `HH:MM:SS` time column into seconds since
    /// midnight of the service day (hours may exceed 23).
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Model`] if the value is not a valid
    /// GTFS time.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Arrival {
    ///     arrival_time: Option<u32>,
    /// }
    ///
    /// impl CsvRecord for Arrival {
    ///     const FILE_NAME: &'static str = "stop_times.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Arrival {
    ///             arrival_time: row.opt_time("arrival_time")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "trip_id,arrival_time\nt0,6:00:00\nt0,\n";
    ///     let rows: Vec<Arrival> = csv::read("stop_times.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].arrival_time, Some(6 * 3600));
    ///     assert!(rows[1].arrival_time.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_time(&self, name: &str) -> Result<Option<u32>, ParseError> {
        let raw = match self.opt(name) {
            None => return Ok(None),
            Some(raw) => raw,
        };
        match parse_gtfs_time(raw) {
            Ok(seconds) => Ok(Some(seconds)),
            Err(e) => Err(self.err(name, ParseErrorKind::Model(e))),
        }
    }

    /// Parses a required `HH:MM:SS` time column into seconds since
    /// midnight of the service day.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Model`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Window {
    ///     start_time: u32,
    /// }
    ///
    /// impl CsvRecord for Window {
    ///     const FILE_NAME: &'static str = "frequencies.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Window {
    ///             start_time: row.req_time("start_time")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "trip_id,start_time\nt0,25:10:00\n";
    ///     let rows: Vec<Window> = csv::read("frequencies.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].start_time, 25 * 3600 + 600);
    ///     Ok(())
    /// }
    /// ```
    pub fn req_time(&self, name: &str) -> Result<u32, ParseError> {
        let raw = self.req(name)?;
        match parse_gtfs_time(raw) {
            Ok(seconds) => Ok(seconds),
            Err(e) => Err(self.err(name, ParseErrorKind::Model(e))),
        }
    }

    /// Parses an optional `YYYYMMDD` date column.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Model`] if the value is not a valid
    /// GTFS date.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::GtfsDate;
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Validity {
    ///     feed_start_date: Option<GtfsDate>,
    /// }
    ///
    /// impl CsvRecord for Validity {
    ///     const FILE_NAME: &'static str = "feed_info.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Validity {
    ///             feed_start_date: row.opt_date("feed_start_date")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "feed_publisher_name,feed_start_date\nDemo,20260101\n";
    ///     let rows: Vec<Validity> = csv::read("feed_info.txt", data.as_bytes())?;
    ///     assert!(rows[0].feed_start_date.is_some());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_date(&self, name: &str) -> Result<Option<GtfsDate>, ParseError> {
        let raw = match self.opt(name) {
            None => return Ok(None),
            Some(raw) => raw,
        };
        match GtfsDate::parse(raw) {
            Ok(date) => Ok(Some(date)),
            Err(e) => Err(self.err(name, ParseErrorKind::Model(e))),
        }
    }

    /// Parses a required `YYYYMMDD` date column.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Model`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsDate, Weekday};
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Start {
    ///     start_date: GtfsDate,
    /// }
    ///
    /// impl CsvRecord for Start {
    ///     const FILE_NAME: &'static str = "calendar.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Start {
    ///             start_date: row.req_date("start_date")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "service_id,start_date\nwd,20260724\n";
    ///     let rows: Vec<Start> = csv::read("calendar.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].start_date.weekday(), Weekday::Friday);
    ///     Ok(())
    /// }
    /// ```
    pub fn req_date(&self, name: &str) -> Result<GtfsDate, ParseError> {
        let raw = self.req(name)?;
        match GtfsDate::parse(raw) {
            Ok(date) => Ok(date),
            Err(e) => Err(self.err(name, ParseErrorKind::Model(e))),
        }
    }

    /// Parses a required decimal currency amount column.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Model`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::CurrencyAmount;
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Price {
    ///     amount: CurrencyAmount,
    /// }
    ///
    /// impl CsvRecord for Price {
    ///     const FILE_NAME: &'static str = "fare_products.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Price {
    ///             amount: row.req_currency("amount")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "fare_product_id,amount\nsingle,57.00\n";
    ///     let rows: Vec<Price> = csv::read("fare_products.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].amount.to_string(), "57.00");
    ///     Ok(())
    /// }
    /// ```
    pub fn req_currency(&self, name: &str) -> Result<CurrencyAmount, ParseError> {
        let raw = self.req(name)?;
        match CurrencyAmount::parse(raw) {
            Ok(amount) => Ok(amount),
            Err(e) => Err(self.err(name, ParseErrorKind::Model(e))),
        }
    }

    /// Parses an optional `0`/`1` flag column.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Invalid`] for values other than `0`
    /// and `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Producer {
    ///     is_producer: bool,
    /// }
    ///
    /// impl CsvRecord for Producer {
    ///     const FILE_NAME: &'static str = "attributions.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Producer {
    ///             is_producer: row.opt_bool01("is_producer")?.unwrap_or(false),
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "organization_name,is_producer\nDemo,1\nOther,\n";
    ///     let rows: Vec<Producer> = csv::read("attributions.txt", data.as_bytes())?;
    ///     assert!(rows[0].is_producer);
    ///     assert!(!rows[1].is_producer);
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_bool01(&self, name: &str) -> Result<Option<bool>, ParseError> {
        match self.opt(name) {
            None => Ok(None),
            Some("0") => Ok(Some(false)),
            Some("1") => Ok(Some(true)),
            Some(raw) => Err(self.invalid(name, raw, "0 or 1")),
        }
    }

    /// Parses a required `0`/`1` flag column.
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`],
    /// [`ParseErrorKind::EmptyValue`] or [`ParseErrorKind::Invalid`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Monday {
    ///     monday: bool,
    /// }
    ///
    /// impl CsvRecord for Monday {
    ///     const FILE_NAME: &'static str = "calendar.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Monday {
    ///             monday: row.req_bool01("monday")?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "service_id,monday\nwd,1\n";
    ///     let rows: Vec<Monday> = csv::read("calendar.txt", data.as_bytes())?;
    ///     assert!(rows[0].monday);
    ///     Ok(())
    /// }
    /// ```
    pub fn req_bool01(&self, name: &str) -> Result<bool, ParseError> {
        let raw = self.req(name)?;
        match raw {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(self.invalid(name, raw, "0 or 1")),
        }
    }

    /// Builds a field-level error at this row's position.
    ///
    /// # Arguments
    ///
    /// * `field` - Column name the error is about
    /// * `kind` - The failure kind
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Strict;
    ///
    /// impl CsvRecord for Strict {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Err(row.err("stop_id", ParseErrorKind::EmptyValue))
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_id\nA\n";
    ///     match csv::read::<Strict, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected an error"),
    ///         Err(e) => {
    ///             assert_eq!(e.file, "stops.txt");
    ///             assert_eq!(e.line, 2);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn err(&self, field: &str, kind: ParseErrorKind) -> ParseError {
        ParseError {
            file: self.file.to_string(),
            line: self.line,
            field: Some(field.to_string()),
            kind,
        }
    }

    /// Builds an invalid-value error at this row's position.
    ///
    /// # Arguments
    ///
    /// * `field` - Column name the error is about
    /// * `value` - The rejected raw value
    /// * `expected` - What the parser expected
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Lat;
    ///
    /// impl CsvRecord for Lat {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         let raw = row.req("stop_lat")?;
    ///         match raw.parse::<f64>() {
    ///             Ok(_) => Ok(Lat),
    ///             Err(_) => Err(row.invalid("stop_lat", raw, "latitude")),
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_lat\nnorth\n";
    ///     match csv::read::<Lat, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected an error"),
    ///         Err(e) => assert!(e.to_string().contains("'north'")),
    ///     }
    /// }
    /// ```
    pub fn invalid(&self, field: &str, value: &str, expected: &str) -> ParseError {
        self.err(
            field,
            ParseErrorKind::Invalid {
                value: value.to_string(),
                expected: expected.to_string(),
            },
        )
    }
}
