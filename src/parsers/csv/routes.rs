//! `routes.txt` reader.

use super::{CsvRecord, Row, opt_string};
use crate::model::{CemvSupport, ContinuousPickupDropOff, Route, RouteType};
use crate::parsers::ParseError;

impl CsvRecord for Route {
    const FILE_NAME: &'static str = "routes.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let route_type = row.req_code("route_type", RouteType::from_code, "a route type code")?;
        let mut route = Route::new(row.req("route_id")?, route_type);
        route.agency_id = opt_string(row, "agency_id");
        route.route_short_name = opt_string(row, "route_short_name");
        route.route_long_name = opt_string(row, "route_long_name");
        route.route_desc = opt_string(row, "route_desc");
        route.route_url = opt_string(row, "route_url");
        route.route_color = opt_string(row, "route_color");
        route.route_text_color = opt_string(row, "route_text_color");
        route.route_sort_order = row.opt_num("route_sort_order", "a non-negative integer")?;
        route.continuous_pickup = row
            .opt_code(
                "continuous_pickup",
                ContinuousPickupDropOff::from_code,
                "code 0-3",
            )?
            .unwrap_or_default();
        route.continuous_drop_off = row
            .opt_code(
                "continuous_drop_off",
                ContinuousPickupDropOff::from_code,
                "code 0-3",
            )?
            .unwrap_or_default();
        route.network_id = opt_string(row, "network_id");
        route.cemv_support = row
            .opt_code("cemv_support", CemvSupport::from_code, "code 0-2")?
            .unwrap_or_default();
        Ok(route)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{read, read_path, test_support::feed_file};
    use crate::model::{Route, RouteType};
    use crate::parsers::ParseError;

    #[test]
    fn test_sample_routes() -> Result<(), ParseError> {
        let routes: Vec<Route> = read_path(feed_file("routes.txt"))?;
        assert_eq!(routes.len(), 5);
        assert_eq!(routes[0].route_id, "AB");
        assert_eq!(routes[0].agency_id.as_deref(), Some("DTA"));
        assert_eq!(routes[0].route_type, RouteType::Bus);
        assert_eq!(routes[0].route_short_name.as_deref(), Some("10"));
        Ok(())
    }

    #[test]
    fn test_extended_route_type() -> Result<(), ParseError> {
        let data = "route_id,route_type\nIC,102\n";
        let routes: Vec<Route> = read("routes.txt", data.as_bytes())?;
        assert_eq!(routes[0].route_type, RouteType::Extended(102));
        Ok(())
    }
}
