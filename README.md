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
# everything: CSV tables, locations.geojson and zipped feeds:
cargo add gtfs-rs --features geojson,zip
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

## Validation

Structural validation is built into the core - no features needed.
`GtfsReference::validate()` checks the spec's conditionally required/forbidden field combinations, primary-key uniqueness and referential integrity across all tables, and returns a report with every found issue at once (errors and warnings), each carrying a machine-readable rule code and a trace to the offending record.
Runnable as [`examples/validate_feed`](examples/validate_feed/main.rs):
`cargo run --example validate_feed`.

```rust
use gtfs_rs::{GtfsReference, StopTime, Trip};

fn main() {
    let mut gtfs = GtfsReference::new();
    gtfs.trips.push(Trip::new("t0", "NO_SUCH_ROUTE", "NO_SVC"));
    gtfs.stop_times.push(StopTime::new("t0", "NO_SUCH_STOP", 1, 8 * 3600));

    let report = gtfs.validate();
    println!("{report}"); // "3 error(s), 0 warning(s)"
    for issue in report.errors() {
        // e.g. "trips.txt, record `t0`, field `route_id`:
        // references unknown record `NO_SUCH_ROUTE`"
        println!("{issue}");
    }
}
```

It is an embeddable pre-flight check for Rust pipelines, not a
replacement for the canonical
[MobilityData gtfs-validator](https://github.com/MobilityData/gtfs-validator),
which covers hundreds of rules including best practices.

## Serialization

Writing feeds is built into the core too - no features, no extra dependencies (hand-rolled RFC 4180 CSV and GeoJSON output).
`writers::write_dir` serializes a whole `GtfsReference` into an unpacked dataset directory: the five required tables are always written (header included even when empty), every other table only when it has records, `locations.geojson` when there are GTFS-Flex zones.
Every one of the 31 CSV tables also has its named shortcut (`write_agencies`, `write_fare_products`, ...) over the generic `csv::write_path::<T>`, which serves custom extension tables as well.
Runnable as [`examples/write_feed`](examples/write_feed/main.rs):
`cargo run --example write_feed`.

```rust
use gtfs_rs::writers;
use gtfs_rs::writers::csv;
use gtfs_rs::{GtfsReference, Stop};

fn main() {
    let mut gtfs = GtfsReference::new();
    gtfs.stops.push(Stop::new("A").with_name("Alpha").with_coordinates(55.751, 37.618));

    // the whole dataset into a directory
    match writers::write_dir(&gtfs, "out_feed") {
        Ok(()) => println!("dataset written to out_feed/"),
        Err(e) => eprintln!("failed to write feed: {e}"),
    }

    // or one table at a time via its named shortcut - to any path
    match csv::write_stops(&gtfs.stops, "out_feed/stops_only.txt") {
        Ok(()) => println!("stops_only.txt written"),
        Err(e) => eprintln!("failed to write stops: {e}"),
    }
}
```

With the `zip` feature enabled, `writers::zip::write_zip` packs the same table selection into a deflate-compressed archive, and `write_zip_bytes` builds it in memory (e.g. to upload without touching the disk).

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
          Ok(agencies) => match agencies.first() {
          Some(agency) => println!("operated by {}", agency.agency_name),
          None => println!("agency.txt has no records"),
      },
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
  use gtfs_rs::parsers::geojson;

  fn main() {
      match geojson::read_locations("tests/data/flex_feed/locations.geojson") {
          Ok(zones) => println!("{} on-demand zones", zones.len()),
          Err(e) => eprintln!("failed to read zones: {e}"),
      }
  }
  ```

- `zip` (off by default; implies `parse`) - the
  `gtfs_rs::parsers::zip` module reading whole zipped feeds - the
  form feeds are actually distributed in - and the
  `gtfs_rs::writers::zip` module packing them back; adds the `zip`
  dependency. Works from a path or from bytes already in memory
  (e.g. a fresh HTTP download), and picks `locations.geojson` up
  when `geojson` is enabled too. Runnable as
  [`examples/read_zip_feed`](examples/read_zip_feed/main.rs):
  `cargo run --example read_zip_feed --features zip` - it packs the
  bundled sample feed in memory first, so no archive file is stored
  in the repository.

  ```rust
  use gtfs_rs::parsers::zip;

  fn main() {
      match zip::read_zip("feed.zip") {
          Ok(gtfs) => println!("{} trips", gtfs.trips.len()),
          Err(e) => eprintln!("failed to read the archive: {e}"),
      }
  }
  ```

## Scope

The crate covers the full GTFS Schedule lifecycle, I believe: model, parsing,
validation and serialization.

Don't think gonna do it, but maybe in future:
- GTFS Realtime.

## Acknowledgements

This crate stands on the work of the GTFS community:

- [MobilityData](https://mobilitydata.org/) - stewardship of the
  GTFS specification and [gtfs.org](https://gtfs.org/), and the
  [gtfs-validator](https://github.com/MobilityData/gtfs-validator),
  whose rule catalogue inspired the built-in validation;
- [Google and the google/transit contributors](https://github.com/google/transit) -
  the original specification, its canonical reference text and the
  sample feed used as a test fixture here;
- everyone maintaining and evolving GTFS, Fares v2 and GTFS-Flex in
  the open.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
