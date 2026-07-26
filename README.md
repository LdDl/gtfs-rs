# gtfs-rs

Complete in-memory data model of the [GTFS Schedule
specification](https://gtfs.org/documentation/schedule/reference/) for
Rust. Every dataset file is represented by a typed struct, every
enumerated field by a typed enum, collected in the `GtfsReference`
container. Zero dependencies.

## Installation

With `cargo add`:

```sh
cargo add gtfs-rs
# with the CSV parser:
cargo add gtfs-rs --features parse
# with the CSV parser and the GTFS-Flex locations.geojson reader:
cargo add gtfs-rs --features geojson
```

Or manually in `Cargo.toml`:

```toml
[dependencies]
gtfs-rs = "0.1"
# or, with parsers enabled:
gtfs-rs = { version = "0.1", features = ["geojson"] }
```

Without features the crate is the pure data model with zero
dependencies; see [Optional features](#optional-features).

## Coverage

| Spec file | Type(s) |
|---|---|
| [`agency.txt`](https://gtfs.org/documentation/schedule/reference/#agencytxt) | `Agency`, `CemvSupport` |
| [`stops.txt`](https://gtfs.org/documentation/schedule/reference/#stopstxt) | `Stop`, `LocationType`, `WheelchairBoarding`, `StopAccess` |
| [`routes.txt`](https://gtfs.org/documentation/schedule/reference/#routestxt) | `Route`, `RouteType`, `ContinuousPickupDropOff`, `CemvSupport` |
| [`trips.txt`](https://gtfs.org/documentation/schedule/reference/#tripstxt) | `Trip`, `Direction`, `WheelchairAccessible`, `BikesAllowed`, `CarsAllowed` |
| [`stop_times.txt`](https://gtfs.org/documentation/schedule/reference/#stop_timestxt) | `StopTime`, `PickupDropOffType`, `Timepoint` |
| [`calendar.txt`](https://gtfs.org/documentation/schedule/reference/#calendartxt) | `Calendar` |
| [`calendar_dates.txt`](https://gtfs.org/documentation/schedule/reference/#calendar_datestxt) | `CalendarDate`, `ExceptionType` |
| [`fare_attributes.txt`](https://gtfs.org/documentation/schedule/reference/#fare_attributestxt) (legacy Fares v1) | `FareAttributeV1`, `PaymentMethod`, `FareTransfers` |
| [`fare_rules.txt`](https://gtfs.org/documentation/schedule/reference/#fare_rulestxt) (legacy Fares v1) | `FareRuleV1` |
| [`timeframes.txt`](https://gtfs.org/documentation/schedule/reference/#timeframestxt) | `Timeframe` |
| [`rider_categories.txt`](https://gtfs.org/documentation/schedule/reference/#rider_categoriestxt) | `RiderCategory` |
| [`fare_media.txt`](https://gtfs.org/documentation/schedule/reference/#fare_mediatxt) | `FareMedia`, `FareMediaType` |
| [`fare_products.txt`](https://gtfs.org/documentation/schedule/reference/#fare_productstxt) | `FareProduct` |
| [`fare_leg_rules.txt`](https://gtfs.org/documentation/schedule/reference/#fare_leg_rulestxt) | `FareLegRule` |
| [`fare_leg_join_rules.txt`](https://gtfs.org/documentation/schedule/reference/#fare_leg_join_rulestxt) | `FareLegJoinRule` |
| [`fare_transfer_rules.txt`](https://gtfs.org/documentation/schedule/reference/#fare_transfer_rulestxt) | `FareTransferRule`, `FareTransferType`, `DurationLimitType` |
| [`areas.txt`](https://gtfs.org/documentation/schedule/reference/#areastxt) / [`stop_areas.txt`](https://gtfs.org/documentation/schedule/reference/#stop_areastxt) | `Area`, `StopArea` |
| [`networks.txt`](https://gtfs.org/documentation/schedule/reference/#networkstxt) / [`route_networks.txt`](https://gtfs.org/documentation/schedule/reference/#route_networkstxt) | `Network`, `RouteNetwork` |
| [`shapes.txt`](https://gtfs.org/documentation/schedule/reference/#shapestxt) | `ShapePoint` |
| [`frequencies.txt`](https://gtfs.org/documentation/schedule/reference/#frequenciestxt) | `Frequency`, `ExactTimes` |
| [`transfers.txt`](https://gtfs.org/documentation/schedule/reference/#transferstxt) | `Transfer`, `TransferType` |
| [`pathways.txt`](https://gtfs.org/documentation/schedule/reference/#pathwaystxt) | `Pathway`, `PathwayMode` |
| [`levels.txt`](https://gtfs.org/documentation/schedule/reference/#levelstxt) | `Level` |
| [`location_groups.txt`](https://gtfs.org/documentation/schedule/reference/#location_groupstxt) / [`location_group_stops.txt`](https://gtfs.org/documentation/schedule/reference/#location_group_stopstxt) | `LocationGroup`, `LocationGroupStop` |
| [`locations.geojson`](https://gtfs.org/documentation/schedule/reference/#locationsgeojson) | `Location`, `LocationGeometry` |
| [`booking_rules.txt`](https://gtfs.org/documentation/schedule/reference/#booking_rulestxt) | `BookingRule`, `BookingType` |
| [`translations.txt`](https://gtfs.org/documentation/schedule/reference/#translationstxt) | `Translation`, `TableName` |
| [`feed_info.txt`](https://gtfs.org/documentation/schedule/reference/#feed_infotxt) | `FeedInfo` |
| [`attributions.txt`](https://gtfs.org/documentation/schedule/reference/#attributionstxt) | `Attribution` |

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
  use gtfs_rs::parsers;
  use gtfs_rs::parsers::csv;

  fn main() {
      // one table via its named function
      match csv::read_agencies("tests/data/sample_feed/agency.txt") {
          Ok(agencies) => println!("operated by {}", agencies[0].agency_name),
          Err(e) => eprintln!("failed to read agencies: {e}"),
      }

      // or a whole unpacked feed directory at once
      match parsers::read_dir("tests/data/sample_feed") {
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

- `geojson` (off by default; implies `parse`) - the
  `gtfs_rs::parsers::geojson` module reading the GTFS-Flex
  `locations.geojson` zones; adds the `serde_json` dependency. With
  the feature enabled, `parsers::read_dir` picks the file up
  automatically; a single file reads as:

  ```rust
  use gtfs_rs::parsers::{ParseError, geojson};

  fn main() -> Result<(), ParseError> {
      match geojson::read_locations("tests/data/flex_feed/locations.geojson") {
          Ok(zones) => println!("{} on-demand zones", zones.len()),
          Err(e) => eprintln!("failed to read zones: {e}"),
      }
      Ok(())
  }
  ```

## Scope

The crate models the dataset; writing files is not implemented yet.
Deliberately out of scope, but W.I.P.:

- serialization (writing feeds back to CSV/GeoJSON);
- feed validation beyond basic type safety;
- reading zipped feeds (a `zip` parser next to `csv`/`geojson`);

Don't think gonna do it, but maybe:
- GTFS Realtime.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
