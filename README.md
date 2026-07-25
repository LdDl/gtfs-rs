# gtfs-rs

Complete in-memory data model of the [GTFS Schedule
specification](https://gtfs.org/documentation/schedule/reference/) for
Rust. Every dataset file is represented by a typed struct, every
enumerated field by a typed enum, collected in the `GtfsReference`
container. Zero dependencies.

## Coverage

| Spec file | Type(s) |
|---|---|
| `agency.txt` | `Agency`, `CemvSupport` |
| `stops.txt` | `Stop`, `LocationType`, `WheelchairBoarding`, `StopAccess` |
| `routes.txt` | `Route`, `RouteType`, `ContinuousPickupDropOff`, `CemvSupport` |
| `trips.txt` | `Trip`, `Direction`, `WheelchairAccessible`, `BikesAllowed`, `CarsAllowed` |
| `stop_times.txt` | `StopTime`, `PickupDropOffType`, `Timepoint` |
| `calendar.txt` | `Calendar` |
| `calendar_dates.txt` | `CalendarDate`, `ExceptionType` |
| `fare_attributes.txt` (legacy Fares v1) | `FareAttributeV1`, `PaymentMethod`, `FareTransfers` |
| `fare_rules.txt` (legacy Fares v1) | `FareRuleV1` |
| `timeframes.txt` | `Timeframe` |
| `rider_categories.txt` | `RiderCategory` |
| `fare_media.txt` | `FareMedia`, `FareMediaType` |
| `fare_products.txt` | `FareProduct` |
| `fare_leg_rules.txt` | `FareLegRule` |
| `fare_leg_join_rules.txt` | `FareLegJoinRule` |
| `fare_transfer_rules.txt` | `FareTransferRule`, `FareTransferType`, `DurationLimitType` |
| `areas.txt` / `stop_areas.txt` | `Area`, `StopArea` |
| `networks.txt` / `route_networks.txt` | `Network`, `RouteNetwork` |
| `shapes.txt` | `ShapePoint` |
| `frequencies.txt` | `Frequency`, `ExactTimes` |
| `transfers.txt` | `Transfer`, `TransferType` |
| `pathways.txt` | `Pathway`, `PathwayMode` |
| `levels.txt` | `Level` |
| `location_groups.txt` / `location_group_stops.txt` | `LocationGroup`, `LocationGroupStop` |
| `locations.geojson` | `Location`, `LocationGeometry` |
| `booking_rules.txt` | `BookingRule`, `BookingType` |
| `translations.txt` | `Translation`, `TableName` |
| `feed_info.txt` | `FeedInfo` |
| `attributions.txt` | `Attribution` |

Field names follow the specification verbatim. Required fields are
plain values, optional fields are `Option`s, and enumerated fields
with a spec-defined default are plain enums implementing `Default`.
Every enum converts to and from its wire representation
(`from_code`/`code`, or `from_name`/`name` for `translations.table_name`).
Legacy Fares v1 structs carry a `V1` name suffix; unsuffixed fare
types belong to the current Fares v2 framework.

Supporting types: `GtfsDate` (`YYYYMMDD` dates with validation,
chronological ordering and weekday computation), `parse_gtfs_time` /
`format_gtfs_time` (`HH:MM:SS` values, including hours past midnight
such as `25:10:00`), and `CurrencyAmount` (exact decimal money for
fare prices, as the spec mandates - no floating-point rounding).

## Example

Runnable as [`examples/build_feed`](examples/build_feed/main.rs):
`cargo run --example build_feed`.

```rust
use gtfs_rs::{
    Calendar, Direction, Frequency, GtfsDate, GtfsError, GtfsReference, Route, RouteType, Stop,
    StopTime, Trip,
};

fn build_feed() -> Result<GtfsReference, GtfsError> {
    let mut gtfs = GtfsReference::new();
    gtfs.stops.push(Stop::new("A").with_name("Alpha").with_coordinates(55.751, 37.618));
    gtfs.stops.push(Stop::new("B").with_name("Beta").with_coordinates(55.760, 37.640));
    gtfs.routes.push(Route::new("L1", RouteType::Tram).with_short_name("1"));
    gtfs.calendar.push(
        Calendar::new(
            "weekday",
            GtfsDate::new(2026, 1, 1)?,
            GtfsDate::new(2026, 12, 31)?,
        )
        .with_weekdays(),
    );
    gtfs.trips.push(Trip::new("t0", "L1", "weekday").with_direction(Direction::Outbound));
    gtfs.stop_times.push(StopTime::new("t0", "A", 0, 8 * 3600));
    gtfs.stop_times.push(StopTime::new("t0", "B", 1, 8 * 3600 + 600));
    gtfs.frequencies.push(Frequency::new("t0", 7 * 3600, 10 * 3600, 300));
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
```

## Optional features

- `parse` (off by default; keeps the default build dependency-free) -
  the `gtfs_rs::parsers::csv` module: header-driven CSV readers with
  full error context (file, line, field). Enable it in
  `Cargo.toml`:

  ```toml
  [dependencies]
  gtfs-rs = { version = "0.1", features = ["parse"] }
  ```

  Tables can be read one at a time from any path, or a whole
  unpacked feed directory at once. Runnable as
  [`examples/read_feed`](examples/read_feed/main.rs) against the
  bundled sample feed:
  `cargo run --example read_feed --features parse`.

  ```rust
  use gtfs_rs::parsers::csv;

  fn main() {
      // one table via its named function
      match csv::read_agencies("tests/data/sample_feed/agency.txt") {
          Ok(agencies) => println!("operated by {}", agencies[0].agency_name),
          Err(e) => eprintln!("failed to read agencies: {e}"),
      }

      // or a whole unpacked feed directory at once
      match csv::read_dir("tests/data/sample_feed") {
          Ok(gtfs) => {
              println!(
                  "loaded {} routes, {} trips, {} stop times",
                  gtfs.routes.len(),
                  gtfs.trips.len(),
                  gtfs.stop_times.len(),
              );
              for trip in gtfs.trips_of_route("AB") {
                  println!("route AB trip: {}", trip.trip_id);
              }
          }
          // errors point at the exact record, e.g.:
          // "stop_times.txt, line 12401, field `arrival_time`:
          //  invalid GTFS time value: '8h00' (expected HH:MM:SS)"
          Err(e) => eprintln!("failed to load feed: {e}"),
      }
  }
  ```

  Every one of the 31 CSV tables has its named shortcut
  (`read_stops`, `read_fare_products`, ...); the generic
  `csv::read_path::<T>(path)` underlies them and serves custom
  extension tables.

## Scope

The crate models the dataset; it does not read or write files.
Deliberately out of scope, but W.I.P.:

- GeoJSON parsing and serialization - entity structs mirror the
  spec field-for-field so parsers can be layered on top;
- feed validation beyond basic type safety;

Don't think gonna do it, but maybe:
- GTFS Realtime.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
