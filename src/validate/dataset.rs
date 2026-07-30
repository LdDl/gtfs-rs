//! Dataset-wide rules: primary-key uniqueness, the shared GTFS-Flex
//! id space, and cross-file conditionals.

use std::collections::{HashMap, HashSet};

use crate::reference::GtfsReference;
use crate::validate::report::{Rule, Severity, ValidationIssue};

/// Runs every dataset-wide rule.
pub fn check(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    check_duplicates(gtfs, issues);
    check_flex_id_space(gtfs, issues);
    check_multi_agency(gtfs, issues);
    check_network_conflict(gtfs, issues);
    check_feed_info(gtfs, issues);
    check_trips_without_stop_times(gtfs, issues);
}

/// Reports duplicate primary keys, one issue per extra occurrence.
fn check_duplicates(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let tables: [(&'static str, Vec<&str>); 12] = [
        (
            "agency.txt",
            gtfs.agencies
                .iter()
                .filter_map(|a| a.agency_id.as_deref())
                .collect(),
        ),
        (
            "stops.txt",
            gtfs.stops.iter().map(|s| s.stop_id.as_str()).collect(),
        ),
        (
            "routes.txt",
            gtfs.routes.iter().map(|r| r.route_id.as_str()).collect(),
        ),
        (
            "trips.txt",
            gtfs.trips.iter().map(|t| t.trip_id.as_str()).collect(),
        ),
        (
            "calendar.txt",
            gtfs.calendar
                .iter()
                .map(|c| c.service_id.as_str())
                .collect(),
        ),
        (
            "fare_attributes.txt",
            gtfs.fare_attributes
                .iter()
                .map(|f| f.fare_id.as_str())
                .collect(),
        ),
        (
            "areas.txt",
            gtfs.areas.iter().map(|a| a.area_id.as_str()).collect(),
        ),
        (
            "networks.txt",
            gtfs.networks
                .iter()
                .map(|n| n.network_id.as_str())
                .collect(),
        ),
        (
            "levels.txt",
            gtfs.levels.iter().map(|l| l.level_id.as_str()).collect(),
        ),
        (
            "pathways.txt",
            gtfs.pathways
                .iter()
                .map(|p| p.pathway_id.as_str())
                .collect(),
        ),
        (
            "location_groups.txt",
            gtfs.location_groups
                .iter()
                .map(|g| g.location_group_id.as_str())
                .collect(),
        ),
        (
            "booking_rules.txt",
            gtfs.booking_rules
                .iter()
                .map(|b| b.booking_rule_id.as_str())
                .collect(),
        ),
    ];
    for (file, ids) in tables {
        let mut seen = HashSet::new();
        for id in ids {
            if !seen.insert(id) {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    file,
                    entity_id: Some(id.to_string()),
                    field: None,
                    rule: Rule::DuplicateId,
                    message: "primary key appears more than once".to_string(),
                });
            }
        }
    }
}

/// `stop_id`, `location_group_id` and `locations.geojson` ids share
/// one id space and must not collide.
fn check_flex_id_space(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let mut space: HashMap<&str, &'static str> = HashMap::new();
    for stop in &gtfs.stops {
        space.insert(stop.stop_id.as_str(), "stops.txt");
    }
    for (id, file) in gtfs
        .location_groups
        .iter()
        .map(|g| (g.location_group_id.as_str(), "location_groups.txt"))
        .chain(
            gtfs.locations
                .iter()
                .map(|l| (l.location_id.as_str(), "locations.geojson")),
        )
    {
        if let Some(other) = space.insert(id, file) {
            // ignore in-table duplicates, reported by DuplicateId
            if other != file {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    file,
                    entity_id: Some(id.to_string()),
                    field: None,
                    rule: Rule::IdSpaceCollision,
                    message: format!("id also used in {}", other),
                });
            }
        }
    }
}

/// With more than one agency, `agency_id` becomes required on
/// agencies, routes and fare attributes.
fn check_multi_agency(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    if gtfs.agencies.len() <= 1 {
        return;
    }
    for agency in &gtfs.agencies {
        if agency.agency_id.is_none() {
            issues.push(missing_agency_id("agency.txt", &agency.agency_name));
        }
    }
    for route in &gtfs.routes {
        if route.agency_id.is_none() {
            issues.push(missing_agency_id("routes.txt", &route.route_id));
        }
    }
    for fare in &gtfs.fare_attributes {
        if fare.agency_id.is_none() {
            issues.push(missing_agency_id("fare_attributes.txt", &fare.fare_id));
        }
    }
}

/// Builds a missing-agency-id issue.
fn missing_agency_id(file: &'static str, entity_id: &str) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        file,
        entity_id: Some(entity_id.to_string()),
        field: Some("agency_id".to_string()),
        rule: Rule::MissingAgencyId,
        message: "agency_id is required when the dataset has multiple agencies".to_string(),
    }
}

/// `routes.network_id` and `route_networks.txt` are mutually
/// exclusive.
fn check_network_conflict(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let uses_inline = gtfs.routes.iter().any(|r| r.network_id.is_some());
    if uses_inline && !gtfs.route_networks.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            file: "route_networks.txt",
            entity_id: None,
            field: None,
            rule: Rule::NetworkIdConflict,
            message: "routes.network_id must not be used together with route_networks.txt"
                .to_string(),
        });
    }
}

/// `feed_info.txt` is required when translations are present.
fn check_feed_info(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    if !gtfs.translations.is_empty() && gtfs.feed_info.is_none() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            file: "feed_info.txt",
            entity_id: None,
            field: None,
            rule: Rule::MissingFeedInfo,
            message: "feed_info.txt is required when translations.txt is present".to_string(),
        });
    }
}

/// A trip without any stop time cannot be ridden - suspicious, but
/// only a warning.
fn check_trips_without_stop_times(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let served: HashSet<&str> = gtfs
        .stop_times
        .iter()
        .map(|st| st.trip_id.as_str())
        .collect();
    for trip in &gtfs.trips {
        if !served.contains(trip.trip_id.as_str()) {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                file: "trips.txt",
                entity_id: Some(trip.trip_id.clone()),
                field: None,
                rule: Rule::TripWithoutStopTimes,
                message: "trip has no stop times".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::TableName;
    use crate::model::{Agency, LocationGroup, Route, RouteNetwork, RouteType, Stop, Translation};
    use crate::reference::GtfsReference;
    use crate::validate::report::{Rule, Severity};

    #[test]
    fn test_duplicates_and_id_space() {
        let mut gtfs = GtfsReference::new();
        gtfs.stops
            .push(Stop::new("A").with_name("A").with_coordinates(0.0, 0.0));
        gtfs.stops.push(
            Stop::new("A")
                .with_name("A twin")
                .with_coordinates(0.0, 0.0),
        );
        gtfs.location_groups.push(LocationGroup::new("A"));

        let report = gtfs.validate();
        let rules: Vec<_> = report.issues().iter().map(|i| i.rule).collect();
        assert!(rules.contains(&Rule::DuplicateId));
        assert!(rules.contains(&Rule::IdSpaceCollision));
    }

    #[test]
    fn test_multi_agency_and_network_conflict() {
        let mut gtfs = GtfsReference::new();
        gtfs.agencies
            .push(Agency::new("First", "https://a.example", "UTC"));
        gtfs.agencies
            .push(Agency::new("Second", "https://b.example", "UTC"));
        let mut route = Route::new("L1", RouteType::Bus).with_short_name("1");
        route.network_id = Some("metro".to_string());
        gtfs.routes.push(route);
        gtfs.route_networks.push(RouteNetwork::new("metro", "L1"));

        let report = gtfs.validate();
        let rules: Vec<_> = report.issues().iter().map(|i| i.rule).collect();
        assert!(rules.contains(&Rule::MissingAgencyId));
        assert!(rules.contains(&Rule::NetworkIdConflict));
    }

    #[test]
    fn test_feed_info_required_with_translations() {
        let mut gtfs = GtfsReference::new();
        gtfs.translations.push(
            Translation::new(TableName::Stops, "stop_name", "en", "Central").for_record("S1"),
        );

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::MissingFeedInfo
                    && issue.severity == Severity::Error)
        );
    }
}
