//! Referential integrity: every foreign key must point at an
//! existing record.

use std::collections::HashSet;

use crate::model::LocationType;
use crate::reference::GtfsReference;
use crate::validate::report::{Rule, Severity, ValidationIssue};

/// The id sets of a dataset, collected once for all lookups.
struct Ids<'a> {
    agencies: HashSet<&'a str>,
    stops: HashSet<&'a str>,
    stations: HashSet<&'a str>,
    platforms: HashSet<&'a str>,
    zones: HashSet<&'a str>,
    routes: HashSet<&'a str>,
    trips: HashSet<&'a str>,
    services: HashSet<&'a str>,
    shapes: HashSet<&'a str>,
    levels: HashSet<&'a str>,
    areas: HashSet<&'a str>,
    networks: HashSet<&'a str>,
    location_groups: HashSet<&'a str>,
    locations: HashSet<&'a str>,
    booking_rules: HashSet<&'a str>,
    fares_v1: HashSet<&'a str>,
    fare_products: HashSet<&'a str>,
    fare_media: HashSet<&'a str>,
    rider_categories: HashSet<&'a str>,
    timeframe_groups: HashSet<&'a str>,
    leg_groups: HashSet<&'a str>,
}

impl<'a> Ids<'a> {
    fn collect(gtfs: &'a GtfsReference) -> Self {
        Ids {
            agencies: gtfs
                .agencies
                .iter()
                .filter_map(|a| a.agency_id.as_deref())
                .collect(),
            stops: gtfs.stops.iter().map(|s| s.stop_id.as_str()).collect(),
            stations: gtfs
                .stops
                .iter()
                .filter(|s| s.location_type == LocationType::Station)
                .map(|s| s.stop_id.as_str())
                .collect(),
            platforms: gtfs
                .stops
                .iter()
                .filter(|s| s.location_type == LocationType::StopOrPlatform)
                .map(|s| s.stop_id.as_str())
                .collect(),
            zones: gtfs
                .stops
                .iter()
                .filter_map(|s| s.zone_id.as_deref())
                .collect(),
            routes: gtfs.routes.iter().map(|r| r.route_id.as_str()).collect(),
            trips: gtfs.trips.iter().map(|t| t.trip_id.as_str()).collect(),
            services: gtfs
                .calendar
                .iter()
                .map(|c| c.service_id.as_str())
                .chain(gtfs.calendar_dates.iter().map(|d| d.service_id.as_str()))
                .collect(),
            shapes: gtfs.shapes.iter().map(|p| p.shape_id.as_str()).collect(),
            levels: gtfs.levels.iter().map(|l| l.level_id.as_str()).collect(),
            areas: gtfs.areas.iter().map(|a| a.area_id.as_str()).collect(),
            networks: gtfs
                .networks
                .iter()
                .map(|n| n.network_id.as_str())
                .chain(gtfs.routes.iter().filter_map(|r| r.network_id.as_deref()))
                .collect(),
            location_groups: gtfs
                .location_groups
                .iter()
                .map(|g| g.location_group_id.as_str())
                .collect(),
            locations: gtfs
                .locations
                .iter()
                .map(|l| l.location_id.as_str())
                .collect(),
            booking_rules: gtfs
                .booking_rules
                .iter()
                .map(|b| b.booking_rule_id.as_str())
                .collect(),
            fares_v1: gtfs
                .fare_attributes
                .iter()
                .map(|f| f.fare_id.as_str())
                .collect(),
            fare_products: gtfs
                .fare_products
                .iter()
                .map(|p| p.fare_product_id.as_str())
                .collect(),
            fare_media: gtfs
                .fare_media
                .iter()
                .map(|m| m.fare_media_id.as_str())
                .collect(),
            rider_categories: gtfs
                .rider_categories
                .iter()
                .map(|c| c.rider_category_id.as_str())
                .collect(),
            timeframe_groups: gtfs
                .timeframes
                .iter()
                .map(|t| t.timeframe_group_id.as_str())
                .collect(),
            leg_groups: gtfs
                .fare_leg_rules
                .iter()
                .filter_map(|r| r.leg_group_id.as_deref())
                .collect(),
        }
    }
}

/// Runs every referential-integrity rule.
pub fn check(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let ids = Ids::collect(gtfs);

    for trip in &gtfs.trips {
        req(
            issues,
            "trips.txt",
            &trip.trip_id,
            "route_id",
            &trip.route_id,
            &ids.routes,
        );
        req(
            issues,
            "trips.txt",
            &trip.trip_id,
            "service_id",
            &trip.service_id,
            &ids.services,
        );
        if let Some(shape_id) = &trip.shape_id {
            req(
                issues,
                "trips.txt",
                &trip.trip_id,
                "shape_id",
                shape_id,
                &ids.shapes,
            );
        }
    }

    for stop_time in &gtfs.stop_times {
        let entity = format!("{}#{}", stop_time.trip_id, stop_time.stop_sequence);
        req(
            issues,
            "stop_times.txt",
            &entity,
            "trip_id",
            &stop_time.trip_id,
            &ids.trips,
        );
        if let Some(stop_id) = &stop_time.stop_id {
            if !ids.stops.contains(stop_id.as_str()) {
                issues.push(unknown("stop_times.txt", &entity, "stop_id", stop_id));
            } else if !ids.platforms.contains(stop_id.as_str()) {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    file: "stop_times.txt",
                    entity_id: Some(entity.clone()),
                    field: Some("stop_id".to_string()),
                    rule: Rule::StopTimeStopNotPlatform,
                    message: format!(
                        "stop_id `{}` is not a stop/platform (location_type 0)",
                        stop_id
                    ),
                });
            }
        }
        if let Some(group) = &stop_time.location_group_id {
            req(
                issues,
                "stop_times.txt",
                &entity,
                "location_group_id",
                group,
                &ids.location_groups,
            );
        }
        if let Some(location) = &stop_time.location_id {
            req(
                issues,
                "stop_times.txt",
                &entity,
                "location_id",
                location,
                &ids.locations,
            );
        }
        for (field, value) in [
            ("pickup_booking_rule_id", &stop_time.pickup_booking_rule_id),
            (
                "drop_off_booking_rule_id",
                &stop_time.drop_off_booking_rule_id,
            ),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "stop_times.txt",
                    &entity,
                    field,
                    value,
                    &ids.booking_rules,
                );
            }
        }
    }

    for stop in &gtfs.stops {
        if let Some(parent) = &stop.parent_station {
            if !ids.stops.contains(parent.as_str()) {
                issues.push(unknown(
                    "stops.txt",
                    &stop.stop_id,
                    "parent_station",
                    parent,
                ));
            } else {
                match stop.location_type {
                    LocationType::BoardingArea => {
                        if !ids.platforms.contains(parent.as_str()) {
                            issues.push(ValidationIssue {
                                severity: Severity::Error,
                                file: "stops.txt",
                                entity_id: Some(stop.stop_id.clone()),
                                field: Some("parent_station".to_string()),
                                rule: Rule::ParentStationNotPlatform,
                                message: format!(
                                    "parent_station `{}` of a boarding area is not a platform",
                                    parent
                                ),
                            });
                        }
                    }
                    LocationType::StopOrPlatform
                    | LocationType::EntranceExit
                    | LocationType::GenericNode => {
                        if !ids.stations.contains(parent.as_str()) {
                            issues.push(ValidationIssue {
                                severity: Severity::Error,
                                file: "stops.txt",
                                entity_id: Some(stop.stop_id.clone()),
                                field: Some("parent_station".to_string()),
                                rule: Rule::ParentStationNotStation,
                                message: format!("parent_station `{}` is not a station", parent),
                            });
                        }
                    }
                    // a station must not have a parent at all,
                    // reported by ForbiddenParentStation
                    LocationType::Station => {}
                }
            }
        }
        if let Some(level_id) = &stop.level_id {
            req(
                issues,
                "stops.txt",
                &stop.stop_id,
                "level_id",
                level_id,
                &ids.levels,
            );
        }
    }

    if !ids.agencies.is_empty() {
        for route in &gtfs.routes {
            if let Some(agency_id) = &route.agency_id {
                req(
                    issues,
                    "routes.txt",
                    &route.route_id,
                    "agency_id",
                    agency_id,
                    &ids.agencies,
                );
            }
        }
        for fare in &gtfs.fare_attributes {
            if let Some(agency_id) = &fare.agency_id {
                req(
                    issues,
                    "fare_attributes.txt",
                    &fare.fare_id,
                    "agency_id",
                    agency_id,
                    &ids.agencies,
                );
            }
        }
    }

    for frequency in &gtfs.frequencies {
        req(
            issues,
            "frequencies.txt",
            &frequency.trip_id,
            "trip_id",
            &frequency.trip_id,
            &ids.trips,
        );
    }

    for timeframe in &gtfs.timeframes {
        req(
            issues,
            "timeframes.txt",
            &timeframe.timeframe_group_id,
            "service_id",
            &timeframe.service_id,
            &ids.services,
        );
    }

    for (index, transfer) in gtfs.transfers.iter().enumerate() {
        let entity = format!("row {}", index + 1);
        for (field, value, set) in [
            ("from_stop_id", &transfer.from_stop_id, &ids.stops),
            ("to_stop_id", &transfer.to_stop_id, &ids.stops),
            ("from_route_id", &transfer.from_route_id, &ids.routes),
            ("to_route_id", &transfer.to_route_id, &ids.routes),
            ("from_trip_id", &transfer.from_trip_id, &ids.trips),
            ("to_trip_id", &transfer.to_trip_id, &ids.trips),
        ] {
            if let Some(value) = value {
                req(issues, "transfers.txt", &entity, field, value, set);
            }
        }
    }

    for pathway in &gtfs.pathways {
        for (field, value) in [
            ("from_stop_id", &pathway.from_stop_id),
            ("to_stop_id", &pathway.to_stop_id),
        ] {
            if !ids.stops.contains(value.as_str()) {
                issues.push(unknown("pathways.txt", &pathway.pathway_id, field, value));
            } else if ids.stations.contains(value.as_str()) {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    file: "pathways.txt",
                    entity_id: Some(pathway.pathway_id.clone()),
                    field: Some(field.to_string()),
                    rule: Rule::PathwayEndpointIsStation,
                    message: format!(
                        "{} `{}` is a station (location_type 1); pathway endpoints must be \
                         platforms, entrances, generic nodes or boarding areas",
                        field, value
                    ),
                });
            }
        }
    }

    for rule in &gtfs.fare_rules {
        req(
            issues,
            "fare_rules.txt",
            &rule.fare_id,
            "fare_id",
            &rule.fare_id,
            &ids.fares_v1,
        );
        if let Some(route_id) = &rule.route_id {
            req(
                issues,
                "fare_rules.txt",
                &rule.fare_id,
                "route_id",
                route_id,
                &ids.routes,
            );
        }
        for (field, value) in [
            ("origin_id", &rule.origin_id),
            ("destination_id", &rule.destination_id),
            ("contains_id", &rule.contains_id),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "fare_rules.txt",
                    &rule.fare_id,
                    field,
                    value,
                    &ids.zones,
                );
            }
        }
    }

    for product in &gtfs.fare_products {
        if let Some(category) = &product.rider_category_id {
            req(
                issues,
                "fare_products.txt",
                &product.fare_product_id,
                "rider_category_id",
                category,
                &ids.rider_categories,
            );
        }
        if let Some(media) = &product.fare_media_id {
            req(
                issues,
                "fare_products.txt",
                &product.fare_product_id,
                "fare_media_id",
                media,
                &ids.fare_media,
            );
        }
    }

    for rule in &gtfs.fare_leg_rules {
        let entity = rule
            .leg_group_id
            .clone()
            .unwrap_or_else(|| rule.fare_product_id.clone());
        req(
            issues,
            "fare_leg_rules.txt",
            &entity,
            "fare_product_id",
            &rule.fare_product_id,
            &ids.fare_products,
        );
        if let Some(network) = &rule.network_id {
            req(
                issues,
                "fare_leg_rules.txt",
                &entity,
                "network_id",
                network,
                &ids.networks,
            );
        }
        for (field, value) in [
            ("from_area_id", &rule.from_area_id),
            ("to_area_id", &rule.to_area_id),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "fare_leg_rules.txt",
                    &entity,
                    field,
                    value,
                    &ids.areas,
                );
            }
        }
        for (field, value) in [
            ("from_timeframe_group_id", &rule.from_timeframe_group_id),
            ("to_timeframe_group_id", &rule.to_timeframe_group_id),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "fare_leg_rules.txt",
                    &entity,
                    field,
                    value,
                    &ids.timeframe_groups,
                );
            }
        }
    }

    for (index, rule) in gtfs.fare_transfer_rules.iter().enumerate() {
        let entity = format!("row {}", index + 1);
        for (field, value) in [
            ("from_leg_group_id", &rule.from_leg_group_id),
            ("to_leg_group_id", &rule.to_leg_group_id),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "fare_transfer_rules.txt",
                    &entity,
                    field,
                    value,
                    &ids.leg_groups,
                );
            }
        }
        if let Some(product) = &rule.fare_product_id {
            req(
                issues,
                "fare_transfer_rules.txt",
                &entity,
                "fare_product_id",
                product,
                &ids.fare_products,
            );
        }
    }

    for join in &gtfs.fare_leg_join_rules {
        let entity = format!("{}->{}", join.from_network_id, join.to_network_id);
        req(
            issues,
            "fare_leg_join_rules.txt",
            &entity,
            "from_network_id",
            &join.from_network_id,
            &ids.networks,
        );
        req(
            issues,
            "fare_leg_join_rules.txt",
            &entity,
            "to_network_id",
            &join.to_network_id,
            &ids.networks,
        );
        for (field, value) in [
            ("from_stop_id", &join.from_stop_id),
            ("to_stop_id", &join.to_stop_id),
        ] {
            if let Some(value) = value {
                req(
                    issues,
                    "fare_leg_join_rules.txt",
                    &entity,
                    field,
                    value,
                    &ids.stops,
                );
            }
        }
    }

    for stop_area in &gtfs.stop_areas {
        let entity = format!("{}:{}", stop_area.area_id, stop_area.stop_id);
        req(
            issues,
            "stop_areas.txt",
            &entity,
            "area_id",
            &stop_area.area_id,
            &ids.areas,
        );
        req(
            issues,
            "stop_areas.txt",
            &entity,
            "stop_id",
            &stop_area.stop_id,
            &ids.stops,
        );
    }

    for route_network in &gtfs.route_networks {
        let entity = format!("{}:{}", route_network.network_id, route_network.route_id);
        req(
            issues,
            "route_networks.txt",
            &entity,
            "network_id",
            &route_network.network_id,
            &ids.networks,
        );
        req(
            issues,
            "route_networks.txt",
            &entity,
            "route_id",
            &route_network.route_id,
            &ids.routes,
        );
    }

    for group_stop in &gtfs.location_group_stops {
        let entity = format!("{}:{}", group_stop.location_group_id, group_stop.stop_id);
        req(
            issues,
            "location_group_stops.txt",
            &entity,
            "location_group_id",
            &group_stop.location_group_id,
            &ids.location_groups,
        );
        req(
            issues,
            "location_group_stops.txt",
            &entity,
            "stop_id",
            &group_stop.stop_id,
            &ids.stops,
        );
    }

    for booking in &gtfs.booking_rules {
        if let Some(service) = &booking.prior_notice_service_id {
            req(
                issues,
                "booking_rules.txt",
                &booking.booking_rule_id,
                "prior_notice_service_id",
                service,
                &ids.services,
            );
        }
    }

    for attribution in &gtfs.attributions {
        let entity = attribution
            .attribution_id
            .clone()
            .unwrap_or_else(|| attribution.organization_name.clone());
        // gated like routes/fare_attributes: with a single id-less
        // agency the id set is empty and references are unverifiable
        if !ids.agencies.is_empty()
            && let Some(agency_id) = &attribution.agency_id
        {
            req(
                issues,
                "attributions.txt",
                &entity,
                "agency_id",
                agency_id,
                &ids.agencies,
            );
        }
        if let Some(route_id) = &attribution.route_id {
            req(
                issues,
                "attributions.txt",
                &entity,
                "route_id",
                route_id,
                &ids.routes,
            );
        }
        if let Some(trip_id) = &attribution.trip_id {
            req(
                issues,
                "attributions.txt",
                &entity,
                "trip_id",
                trip_id,
                &ids.trips,
            );
        }
    }
}

/// Pushes an unknown-reference issue when `value` is not in `set`.
fn req(
    issues: &mut Vec<ValidationIssue>,
    file: &'static str,
    entity: &str,
    field: &str,
    value: &str,
    set: &HashSet<&str>,
) {
    if !set.contains(value) {
        issues.push(unknown(file, entity, field, value));
    }
}

/// Builds an unknown-reference issue.
fn unknown(file: &'static str, entity_id: &str, field: &str, value: &str) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        file,
        entity_id: Some(entity_id.to_string()),
        field: Some(field.to_string()),
        rule: Rule::UnknownReference,
        message: format!("references unknown record `{}`", value),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        Agency, Attribution, FareRuleV1, LocationType, Pathway, PathwayMode, Route, RouteType,
        Stop, StopTime, Timeframe, Trip,
    };
    use crate::reference::GtfsReference;
    use crate::validate::report::Rule;

    #[test]
    fn test_broken_references_are_reported() {
        let mut gtfs = GtfsReference::new();
        gtfs.routes
            .push(Route::new("L1", RouteType::Bus).with_short_name("1"));
        gtfs.trips.push(Trip::new("t0", "NO_SUCH_ROUTE", "NO_SVC"));
        gtfs.stop_times
            .push(StopTime::new("t0", "NO_SUCH_STOP", 1, 3600));

        let report = gtfs.validate();
        let unknown_count = report
            .issues()
            .iter()
            .filter(|issue| issue.rule == Rule::UnknownReference)
            .count();
        // trip route, trip service, stop_time stop
        assert_eq!(unknown_count, 3);
    }

    #[test]
    fn test_stop_time_stop_must_be_platform() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("STA")
                .with_name("Station")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::Station),
        );
        gtfs.stop_times.push(StopTime::new("t0", "STA", 1, 3600));

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::StopTimeStopNotPlatform)
        );
    }

    #[test]
    fn test_agency_refs_skipped_for_single_idless_agency() {
        let mut gtfs = GtfsReference::new();
        gtfs.agencies
            .push(Agency::new("Demo", "https://demo.example", "UTC"));
        let mut attribution = Attribution::new("Org");
        attribution.agency_id = Some("demo".to_string());
        gtfs.attributions.push(attribution);

        let report = gtfs.validate();
        assert!(
            !report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::UnknownReference)
        );
    }

    #[test]
    fn test_parent_station_must_be_station() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("A")
                .with_name("A")
                .with_coordinates(0.0, 0.0)
                .with_parent_station("B"),
        );
        gtfs.stops
            .push(Stop::new("B").with_name("B").with_coordinates(0.0, 0.0));

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::ParentStationNotStation)
        );
    }

    #[test]
    fn test_station_platform_boarding_area_chain_is_valid() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("STA")
                .with_name("Station")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::Station),
        );
        gtfs.stops.push(
            Stop::new("PLAT")
                .with_name("Platform")
                .with_coordinates(0.0, 0.0)
                .with_parent_station("STA"),
        );
        gtfs.stops.push(
            Stop::new("BA")
                .with_location_type(LocationType::BoardingArea)
                .with_parent_station("PLAT"),
        );

        let report = gtfs.validate();
        assert!(report.issues().is_empty());
    }

    #[test]
    fn test_boarding_area_parent_must_be_platform() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("STA")
                .with_name("Station")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::Station),
        );
        gtfs.stops.push(
            Stop::new("BA")
                .with_location_type(LocationType::BoardingArea)
                .with_parent_station("STA"),
        );

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::ParentStationNotPlatform
                    && issue.entity_id.as_deref() == Some("BA"))
        );
    }

    #[test]
    fn test_entrance_parent_must_be_station() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("STA")
                .with_name("Station")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::Station),
        );
        gtfs.stops.push(
            Stop::new("PLAT")
                .with_name("Platform")
                .with_coordinates(0.0, 0.0)
                .with_parent_station("STA"),
        );
        gtfs.stops.push(
            Stop::new("E1")
                .with_name("Entrance")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::EntranceExit)
                .with_parent_station("PLAT"),
        );

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::ParentStationNotStation
                    && issue.entity_id.as_deref() == Some("E1"))
        );
    }

    #[test]
    fn test_pathway_endpoints_must_not_be_stations() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("STA")
                .with_name("Station")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::Station),
        );
        gtfs.stops.push(
            Stop::new("PLAT")
                .with_name("Platform")
                .with_coordinates(0.0, 0.0)
                .with_parent_station("STA"),
        );
        gtfs.stops.push(
            Stop::new("E1")
                .with_name("Entrance")
                .with_coordinates(0.0, 0.0)
                .with_location_type(LocationType::EntranceExit)
                .with_parent_station("STA"),
        );
        gtfs.pathways.push(Pathway::new(
            "pw1",
            "STA",
            "PLAT",
            PathwayMode::Walkway,
            true,
        ));
        gtfs.pathways.push(Pathway::new(
            "pw2",
            "E1",
            "PLAT",
            PathwayMode::Walkway,
            true,
        ));

        let flagged: Vec<(Option<String>, Option<String>)> = gtfs
            .validate()
            .into_iter()
            .filter(|issue| issue.rule == Rule::PathwayEndpointIsStation)
            .map(|issue| (issue.entity_id, issue.field))
            .collect();
        // only the station endpoint of pw1 is illegal
        assert_eq!(
            flagged,
            [(Some("pw1".to_string()), Some("from_stop_id".to_string()))]
        );
    }

    #[test]
    fn test_timeframe_service_and_fare_rule_zones() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops.push(
            Stop::new("A")
                .with_name("A")
                .with_coordinates(0.0, 0.0)
                .with_zone_id("Z1"),
        );
        gtfs.timeframes.push(Timeframe::new("peak", "NO_SVC"));
        let mut rule = FareRuleV1::new("base");
        rule.origin_id = Some("Z1".to_string());
        rule.destination_id = Some("NO_ZONE".to_string());
        rule.contains_id = Some("NO_ZONE".to_string());
        gtfs.fare_rules.push(rule);

        let report = gtfs.validate();
        let fields: Vec<&str> = report
            .issues()
            .iter()
            .filter(|issue| issue.rule == Rule::UnknownReference)
            .filter_map(|issue| issue.field.as_deref())
            .collect();
        assert!(fields.contains(&"service_id"));
        assert!(fields.contains(&"destination_id"));
        assert!(fields.contains(&"contains_id"));
        // Z1 exists on a stop, so origin_id resolves
        assert!(!fields.contains(&"origin_id"));
    }
}
