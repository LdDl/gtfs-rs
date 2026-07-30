//! Builds a small GTFS dataset programmatically and serializes it:
//! the whole feed into a directory plus a single table through a
//! named shortcut.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo run --example write_feed
//! ```

use std::error::Error;
use std::path::Path;

use gtfs_rs::writers;
use gtfs_rs::writers::csv;
use gtfs_rs::{
    Agency, Calendar, GtfsDate, GtfsError, GtfsReference, Route, RouteType, Stop, StopTime, Trip,
};

fn build_feed() -> Result<GtfsReference, GtfsError> {
    let mut gtfs = GtfsReference::new();
    gtfs.agencies
        .push(Agency::new("Demo Transit", "https://demo.example", "Europe/Moscow").with_id("demo"));
    gtfs.stops.push(
        Stop::new("A")
            .with_name("Alpha")
            .with_coordinates(55.751, 37.618),
    );
    gtfs.stops.push(
        Stop::new("B")
            .with_name("Beta")
            .with_coordinates(55.760, 37.640),
    );
    gtfs.routes
        .push(Route::new("L1", RouteType::Tram).with_short_name("1"));
    gtfs.calendar.push(
        Calendar::new(
            "weekday",
            GtfsDate::new(2026, 1, 1)?,
            GtfsDate::new(2026, 12, 31)?,
        )
        .with_weekdays(),
    );
    gtfs.trips.push(Trip::new("t0", "L1", "weekday"));
    gtfs.stop_times.push(StopTime::new("t0", "A", 0, 8 * 3600));
    gtfs.stop_times
        .push(StopTime::new("t0", "B", 1, 8 * 3600 + 600));
    Ok(gtfs)
}

fn write_feed(dir: &Path) -> Result<(), Box<dyn Error>> {
    let gtfs = build_feed()?;
    // the whole dataset at once: required tables always, the rest
    // when non-empty
    writers::write_dir(&gtfs, dir)?;
    // or any single table through its named shortcut - to any path,
    // here next to the feed for comparison
    csv::write_agencies(&gtfs.agencies, dir.join("agency_only.txt"))?;
    Ok(())
}

fn main() {
    // out_feed/ is listed in .gitignore, so the output never lands
    // in version control
    let dir = Path::new("out_feed");
    match write_feed(dir) {
        Ok(()) => println!("dataset written to {}", dir.display()),
        Err(e) => eprintln!("failed to write feed: {e}"),
    }
}
