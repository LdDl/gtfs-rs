//! Named per-table shortcuts: `write_agencies(&rows, "path")`
//! instead of the generic `write_path(&rows, "path")`, so downstream
//! code names the table it writes explicitly.

use std::path::Path;

use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route,
    RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};
use crate::writers::WriteError;
use crate::writers::csv::write_path;

/// Generates a named shortcut for writing one GTFS table to a path.
macro_rules! write_shortcut {
    ($name:ident, $entity:ident, $file:literal) => {
        #[doc = concat!(
                                    "Writes `", $file, "` to a path: a named shortcut for ",
                                    "[`write_path`] with the [`", stringify!($entity),
                                    "`](crate::model::", stringify!($entity), ") entity type."
                                )]
        ///
        /// # Errors
        ///
        /// See [`write_path`].
        ///
        /// # Examples
        ///
        #[doc = concat!(
                                    "```no_run\n",
                                    "use gtfs_rs::", stringify!($entity), ";\n",
                                    "use gtfs_rs::writers::{WriteError, csv};\n",
                                    "\n",
                                    "fn main() -> Result<(), WriteError> {\n",
                                    "    let rows: Vec<", stringify!($entity),
                                    "> = Vec::new();\n",
                                    "    csv::", stringify!($name),
                                    "(&rows, \"out/", $file, "\")?;\n",
                                    "    Ok(())\n",
                                    "}\n",
                                    "```"
                                )]
        pub fn $name(rows: &[$entity], path: impl AsRef<Path>) -> Result<(), WriteError> {
            write_path(rows, path)
        }
    };
}

write_shortcut!(write_agencies, Agency, "agency.txt");
write_shortcut!(write_stops, Stop, "stops.txt");
write_shortcut!(write_routes, Route, "routes.txt");
write_shortcut!(write_trips, Trip, "trips.txt");
write_shortcut!(write_stop_times, StopTime, "stop_times.txt");
write_shortcut!(write_calendar, Calendar, "calendar.txt");
write_shortcut!(write_calendar_dates, CalendarDate, "calendar_dates.txt");
write_shortcut!(
    write_fare_attributes,
    FareAttributeV1,
    "fare_attributes.txt"
);
write_shortcut!(write_fare_rules, FareRuleV1, "fare_rules.txt");
write_shortcut!(write_timeframes, Timeframe, "timeframes.txt");
write_shortcut!(
    write_rider_categories,
    RiderCategory,
    "rider_categories.txt"
);
write_shortcut!(write_fare_media, FareMedia, "fare_media.txt");
write_shortcut!(write_fare_products, FareProduct, "fare_products.txt");
write_shortcut!(write_fare_leg_rules, FareLegRule, "fare_leg_rules.txt");
write_shortcut!(
    write_fare_leg_join_rules,
    FareLegJoinRule,
    "fare_leg_join_rules.txt"
);
write_shortcut!(
    write_fare_transfer_rules,
    FareTransferRule,
    "fare_transfer_rules.txt"
);
write_shortcut!(write_areas, Area, "areas.txt");
write_shortcut!(write_stop_areas, StopArea, "stop_areas.txt");
write_shortcut!(write_networks, Network, "networks.txt");
write_shortcut!(write_route_networks, RouteNetwork, "route_networks.txt");
write_shortcut!(write_shapes, ShapePoint, "shapes.txt");
write_shortcut!(write_frequencies, Frequency, "frequencies.txt");
write_shortcut!(write_transfers, Transfer, "transfers.txt");
write_shortcut!(write_pathways, Pathway, "pathways.txt");
write_shortcut!(write_levels, Level, "levels.txt");
write_shortcut!(write_location_groups, LocationGroup, "location_groups.txt");
write_shortcut!(
    write_location_group_stops,
    LocationGroupStop,
    "location_group_stops.txt"
);
write_shortcut!(write_booking_rules, BookingRule, "booking_rules.txt");
write_shortcut!(write_translations, Translation, "translations.txt");
write_shortcut!(write_feed_info, FeedInfo, "feed_info.txt");
write_shortcut!(write_attributions, Attribution, "attributions.txt");

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::process;

    #[test]
    fn test_shortcut_writes_agency_file() -> Result<(), Box<dyn std::error::Error>> {
        let path = env::temp_dir().join(format!(
            "gtfs_rs_write_shortcut_agency_{}.txt",
            process::id()
        ));
        let agencies = vec![Agency::new("Demo", "https://demo.example", "UTC")];
        write_agencies(&agencies, &path)?;
        let text = fs::read_to_string(&path)?;
        fs::remove_file(&path)?;
        assert!(text.starts_with("agency_id,agency_name,"));
        assert!(text.contains("Demo"));
        Ok(())
    }
}
