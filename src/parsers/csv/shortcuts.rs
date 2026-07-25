//! Named per-table shortcuts: `read_agencies("path")` instead of
//! the generic `read_path::<Agency>("path")`, so downstream code
//! never has to pick the table through a type annotation.

use std::path::Path;

use super::read_path;
use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route,
    RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};
use crate::parsers::ParseError;

/// Generates a named shortcut for reading one GTFS table by path.
macro_rules! table_shortcut {
    ($name:ident, $entity:ident, $file:literal) => {
        #[doc = concat!(
                                    "Reads `", $file, "` from a path: a named shortcut for ",
                                    "[`read_path`] with the [`", stringify!($entity),
                                    "`](crate::model::", stringify!($entity), ") entity type."
                                )]
        ///
        /// The path may have any file name; see [`read_path`].
        ///
        /// # Errors
        ///
        /// See [`read_path`].
        ///
        /// # Examples
        ///
        #[doc = concat!(
                                    "```no_run\n",
                                    "use gtfs_rs::parsers::{ParseError, csv};\n",
                                    "\n",
                                    "fn main() -> Result<(), ParseError> {\n",
                                    "    let records = csv::", stringify!($name),
                                    "(\"feed/", $file, "\")?;\n",
                                    "    println!(\"{} records\", records.len());\n",
                                    "    Ok(())\n",
                                    "}\n",
                                    "```"
                                )]
        pub fn $name(path: impl AsRef<Path>) -> Result<Vec<$entity>, ParseError> {
            read_path(path)
        }
    };
}

table_shortcut!(read_agencies, Agency, "agency.txt");
table_shortcut!(read_stops, Stop, "stops.txt");
table_shortcut!(read_routes, Route, "routes.txt");
table_shortcut!(read_trips, Trip, "trips.txt");
table_shortcut!(read_stop_times, StopTime, "stop_times.txt");
table_shortcut!(read_calendar, Calendar, "calendar.txt");
table_shortcut!(read_calendar_dates, CalendarDate, "calendar_dates.txt");
table_shortcut!(read_fare_attributes, FareAttributeV1, "fare_attributes.txt");
table_shortcut!(read_fare_rules, FareRuleV1, "fare_rules.txt");
table_shortcut!(read_timeframes, Timeframe, "timeframes.txt");
table_shortcut!(read_rider_categories, RiderCategory, "rider_categories.txt");
table_shortcut!(read_fare_media, FareMedia, "fare_media.txt");
table_shortcut!(read_fare_products, FareProduct, "fare_products.txt");
table_shortcut!(read_fare_leg_rules, FareLegRule, "fare_leg_rules.txt");
table_shortcut!(
    read_fare_leg_join_rules,
    FareLegJoinRule,
    "fare_leg_join_rules.txt"
);
table_shortcut!(
    read_fare_transfer_rules,
    FareTransferRule,
    "fare_transfer_rules.txt"
);
table_shortcut!(read_areas, Area, "areas.txt");
table_shortcut!(read_stop_areas, StopArea, "stop_areas.txt");
table_shortcut!(read_networks, Network, "networks.txt");
table_shortcut!(read_route_networks, RouteNetwork, "route_networks.txt");
table_shortcut!(read_shapes, ShapePoint, "shapes.txt");
table_shortcut!(read_frequencies, Frequency, "frequencies.txt");
table_shortcut!(read_transfers, Transfer, "transfers.txt");
table_shortcut!(read_pathways, Pathway, "pathways.txt");
table_shortcut!(read_levels, Level, "levels.txt");
table_shortcut!(read_location_groups, LocationGroup, "location_groups.txt");
table_shortcut!(
    read_location_group_stops,
    LocationGroupStop,
    "location_group_stops.txt"
);
table_shortcut!(read_booking_rules, BookingRule, "booking_rules.txt");
table_shortcut!(read_translations, Translation, "translations.txt");
table_shortcut!(read_feed_info, FeedInfo, "feed_info.txt");
table_shortcut!(read_attributions, Attribution, "attributions.txt");

#[cfg(test)]
mod tests {
    use super::super::test_support::feed_file;
    use super::*;

    #[test]
    fn test_shortcut_reads_sample_agency() -> Result<(), ParseError> {
        let agencies = read_agencies(feed_file("agency.txt"))?;
        assert_eq!(agencies[0].agency_name, "Demo Transit Authority");
        Ok(())
    }

    #[test]
    fn test_shortcut_reads_sample_stops_and_shapes() -> Result<(), ParseError> {
        assert_eq!(read_stops(feed_file("stops.txt"))?.len(), 9);
        assert!(read_shapes(feed_file("shapes.txt"))?.is_empty());
        Ok(())
    }
}
