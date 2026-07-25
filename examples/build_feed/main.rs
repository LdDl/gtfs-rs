//! Builds a small GTFS dataset programmatically and queries it.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo run --example build_feed
//! ```

use gtfs_rs::{
    Calendar, Direction, Frequency, GtfsDate, GtfsError, GtfsReference, Route, RouteType, Stop,
    StopTime, Trip,
};

fn build_feed() -> Result<GtfsReference, GtfsError> {
    let mut gtfs = GtfsReference::new();
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
    gtfs.trips
        .push(Trip::new("t0", "L1", "weekday").with_direction(Direction::Outbound));
    gtfs.stop_times.push(StopTime::new("t0", "A", 0, 8 * 3600));
    gtfs.stop_times
        .push(StopTime::new("t0", "B", 1, 8 * 3600 + 600));
    gtfs.frequencies
        .push(Frequency::new("t0", 7 * 3600, 10 * 3600, 300));
    Ok(gtfs)
}

fn main() {
    match build_feed() {
        Ok(gtfs) => {
            let pattern = gtfs.stop_times_of_trip("t0");
            println!("trip t0 serves {} stops", pattern.len());
        }
        Err(e) => eprintln!("failed to build feed: {e}"),
    }
}
