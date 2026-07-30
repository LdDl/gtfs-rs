//! Builds a deliberately broken GTFS dataset and prints the full
//! validation report - errors and warnings, each with a trace to
//! the offending record. Needs no cargo features: validation is
//! part of the core model.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo run --example validate_feed
//! ```

use gtfs_rs::{
    Calendar, GtfsDate, GtfsError, GtfsReference, Route, RouteType, Stop, StopTime, Trip,
};

/// Assembles a dataset with several intentional mistakes.
fn build_broken_feed() -> Result<GtfsReference, GtfsError> {
    let mut gtfs = GtfsReference::new();

    // a route with no short and no long name
    gtfs.routes.push(Route::new("L1", RouteType::Bus));
    // a stop without a name and coordinates
    gtfs.stops.push(Stop::new("A"));

    gtfs.calendar.push(
        Calendar::new(
            "daily",
            GtfsDate::new(2026, 1, 1)?,
            GtfsDate::new(2026, 12, 31)?,
        )
        .with_all_days(),
    );

    // a trip referencing a route and a service that do not exist
    gtfs.trips.push(Trip::new("t0", "NO_SUCH_ROUTE", "NO_SVC"));
    // its stop time points at an unknown stop
    gtfs.stop_times
        .push(StopTime::new("t0", "NO_SUCH_STOP", 1, 8 * 3600));

    // a well-formed trip - but nobody gave it stop times (warning)
    gtfs.trips.push(Trip::new("t1", "L1", "daily"));

    Ok(gtfs)
}

fn main() {
    let gtfs = match build_broken_feed() {
        Ok(gtfs) => gtfs,
        Err(e) => {
            eprintln!("failed to build the dataset: {e}");
            return;
        }
    };

    let report = gtfs.validate();
    println!("validation: {report}");
    for issue in report.errors() {
        println!("  error:   {issue}");
    }
    for issue in report.warnings() {
        println!("  warning: {issue}");
    }
}
