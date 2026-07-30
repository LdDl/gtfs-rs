//! `routes.txt` writer.

use crate::model::Route;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Route {
    const FILE_NAME: &'static str = "routes.txt";

    const HEADER: &'static [&'static str] = &[
        "route_id",
        "agency_id",
        "route_short_name",
        "route_long_name",
        "route_desc",
        "route_type",
        "route_url",
        "route_color",
        "route_text_color",
        "route_sort_order",
        "continuous_pickup",
        "continuous_drop_off",
        "network_id",
        "cemv_support",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.route_id.clone(),
            self.agency_id.clone().unwrap_or_default(),
            self.route_short_name.clone().unwrap_or_default(),
            self.route_long_name.clone().unwrap_or_default(),
            self.route_desc.clone().unwrap_or_default(),
            self.route_type.code().to_string(),
            self.route_url.clone().unwrap_or_default(),
            self.route_color.clone().unwrap_or_default(),
            self.route_text_color.clone().unwrap_or_default(),
            self.route_sort_order
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.continuous_pickup.code().to_string(),
            self.continuous_drop_off.code().to_string(),
            self.network_id.clone().unwrap_or_default(),
            self.cemv_support.code().to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Route, RouteType};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let route = Route::new("L1", RouteType::Bus).with_short_name("10");
        let fields = route.fields();
        assert_eq!(Route::HEADER.len(), fields.len());
        assert_eq!(fields[0], "L1");
        assert_eq!(fields[1], "");
        assert_eq!(fields[5], "3");
        assert_eq!(fields[10], "1");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::ContinuousPickupDropOff;

        let mut first = Route::new("L1", RouteType::Tram)
            .with_agency_id("DT")
            .with_short_name("1")
            .with_long_name("Circle Line")
            .with_colors("FFD700", "000000");
        first.route_sort_order = Some(5);
        first.continuous_pickup = ContinuousPickupDropOff::PhoneAgency;
        let routes = vec![first, Route::new("IC", RouteType::Extended(102))];
        let mut out = Vec::new();
        write("routes.txt", &routes, &mut out)?;
        let parsed: Vec<Route> = read("routes.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].route_id, "L1");
        assert_eq!(parsed[0].agency_id.as_deref(), Some("DT"));
        assert_eq!(parsed[0].route_short_name.as_deref(), Some("1"));
        assert_eq!(parsed[0].route_long_name.as_deref(), Some("Circle Line"));
        assert_eq!(parsed[0].route_type, RouteType::Tram);
        assert_eq!(parsed[0].route_color.as_deref(), Some("FFD700"));
        assert_eq!(parsed[0].route_sort_order, Some(5));
        assert_eq!(
            parsed[0].continuous_pickup,
            ContinuousPickupDropOff::PhoneAgency
        );
        assert_eq!(parsed[1].route_id, "IC");
        assert_eq!(parsed[1].route_type, RouteType::Extended(102));
        Ok(())
    }
}
