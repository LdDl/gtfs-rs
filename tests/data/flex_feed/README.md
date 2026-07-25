# Demo flex feed

A minimal fictional GTFS-Flex feed ("Demo Flex Rides") authored from
scratch for this project as a parser test fixture - in particular for
`locations.geojson`. It is original data, licensed under the same
Apache-2.0 license as the crate; no external feed was copied.

The structure follows the official specification and examples:

- `locations.geojson` format:
  https://gtfs.org/documentation/schedule/reference/#locationsgeojson
- GTFS-Flex data examples (demand-responsive services):
  https://gtfs.org/documentation/schedule/examples/flex/
- The GTFS-Flex extension page:
  https://gtfs.org/community/extensions/flex/

Contents: one on-demand route (`FLEX1`) serving two fictional zones
near Death Valley; `zone_a` is a `Polygon`, `zone_b` a
`MultiPolygon`. Booking requires a 30-minute advance call
(`booking_rules.txt`), and the `stop_times.txt` rows use
pickup/drop-off windows instead of fixed times.
