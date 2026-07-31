//! The validation entry point: [`GtfsReference::validate`].

use crate::reference::GtfsReference;
use crate::validate::report::ValidationReport;
use crate::validate::{dataset, records, refs};

impl GtfsReference {
    /// Validates the dataset and returns every found issue - intra-record
    /// conditional rules, primary-key uniqueness and referential
    /// integrity. Validation never stops at the first problem: the
    /// report lists everything, split into errors (hard spec
    /// violations) and warnings (suspicious but legal data).
    ///
    /// Note for GTFS-Flex datasets read without the `geojson` cargo
    /// feature: `locations` stays empty, so stop times referencing
    /// GeoJSON zones will report unknown references. Enable the
    /// feature to load the zones.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Rule, StopTime, Trip};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// // references a route and a service that do not exist
    /// gtfs.trips.push(Trip::new("t0", "NO_ROUTE", "NO_SERVICE"));
    /// gtfs.stop_times.push(StopTime::new("t0", "NO_STOP", 1, 8 * 3600));
    /// gtfs.stop_times.push(StopTime::new("t0", "NO_STOP", 2, 9 * 3600));
    ///
    /// let report = gtfs.validate();
    /// assert!(!report.is_valid());
    /// for issue in report.errors() {
    ///     // "trips.txt, record `t0`, field `route_id`:
    ///     //  references unknown record `NO_ROUTE`"
    ///     println!("{issue}");
    /// }
    /// assert!(
    ///     report
    ///         .issues()
    ///         .iter()
    ///         .all(|issue| issue.rule == Rule::UnknownReference)
    /// );
    /// ```
    pub fn validate(&self) -> ValidationReport {
        let mut issues = Vec::new();
        records::check(self, &mut issues);
        refs::check(self, &mut issues);
        dataset::check(self, &mut issues);
        ValidationReport::new(issues)
    }
}

#[cfg(all(test, feature = "parse"))]
mod fixture_tests {
    use crate::parsers;
    use crate::parsers::ParseError;
    use crate::parsers::csv::test_support::FEED_DIR;

    #[test]
    fn test_sample_feed_is_valid() -> Result<(), ParseError> {
        let gtfs = parsers::read_dir(FEED_DIR)?;
        let report = gtfs.validate();
        for issue in report.issues() {
            println!("unexpected: {issue}");
        }
        assert!(report.is_valid());
        assert_eq!(report.warnings().count(), 0);
        Ok(())
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_flex_feed_is_valid() -> Result<(), ParseError> {
        use crate::parsers::csv::test_support::FLEX_DIR;

        let gtfs = parsers::read_dir(FLEX_DIR)?;
        let report = gtfs.validate();
        for issue in report.issues() {
            println!("unexpected: {issue}");
        }
        assert!(report.is_valid());
        Ok(())
    }
}
