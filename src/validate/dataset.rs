//! Dataset-wide rules: primary-key uniqueness, the shared GTFS-Flex
//! id space, and cross-file conditionals.

use std::collections::{HashMap, HashSet};

use crate::misc::format_gtfs_time;
use crate::reference::GtfsReference;
use crate::validate::report::{Rule, Severity, ValidationIssue};

/// Runs every dataset-wide rule.
pub fn check(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    check_duplicates(gtfs, issues);
    check_composite_duplicates(gtfs, issues);
    check_overlapping_frequencies(gtfs, issues);
    check_flex_id_space(gtfs, issues);
    check_multi_agency(gtfs, issues);
    check_network_conflict(gtfs, issues);
    check_feed_info(gtfs, issues);
    check_trips_without_stop_times(gtfs, issues);
}

/// Reports duplicate primary keys, one issue per extra occurrence.
fn check_duplicates(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let tables: [(&'static str, Vec<&str>); 13] = [
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
        (
            "locations.geojson",
            gtfs.locations
                .iter()
                .map(|l| l.location_id.as_str())
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

/// Reports duplicate composite primary keys, one issue per extra
/// occurrence.
fn check_composite_duplicates(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let mut seen = HashSet::new();
    for stop_time in &gtfs.stop_times {
        if !seen.insert((stop_time.trip_id.as_str(), stop_time.stop_sequence)) {
            issues.push(duplicate_key(
                "stop_times.txt",
                &stop_time.trip_id,
                format!(
                    "duplicate stop_sequence `{}` for trip `{}`",
                    stop_time.stop_sequence, stop_time.trip_id
                ),
            ));
        }
    }
    let mut seen = HashSet::new();
    for point in &gtfs.shapes {
        if !seen.insert((point.shape_id.as_str(), point.shape_pt_sequence)) {
            issues.push(duplicate_key(
                "shapes.txt",
                &point.shape_id,
                format!(
                    "duplicate shape_pt_sequence `{}` for shape `{}`",
                    point.shape_pt_sequence, point.shape_id
                ),
            ));
        }
    }
    let mut seen = HashSet::new();
    for date in &gtfs.calendar_dates {
        if !seen.insert((date.service_id.as_str(), date.date)) {
            issues.push(duplicate_key(
                "calendar_dates.txt",
                &date.service_id,
                format!(
                    "duplicate date `{}` for service `{}`",
                    date.date, date.service_id
                ),
            ));
        }
    }
    let mut seen = HashSet::new();
    for frequency in &gtfs.frequencies {
        if !seen.insert((frequency.trip_id.as_str(), frequency.start_time)) {
            issues.push(duplicate_key(
                "frequencies.txt",
                &frequency.trip_id,
                format!(
                    "duplicate start_time `{}` for trip `{}`",
                    format_gtfs_time(frequency.start_time),
                    frequency.trip_id
                ),
            ));
        }
    }
}

/// Builds a composite-duplicate issue.
fn duplicate_key(file: &'static str, entity_id: &str, message: String) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        file,
        entity_id: Some(entity_id.to_string()),
        field: None,
        rule: Rule::DuplicateId,
        message,
    }
}

/// Frequency windows of the same trip must not overlap; a window may
/// start at the exact time the previous one ends. Windows sharing a
/// start time are duplicates of the (`trip_id`, `start_time`)
/// primary key, reported by [`Rule::DuplicateId`] instead.
fn check_overlapping_frequencies(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let mut windows: HashMap<&str, Vec<(u32, u32)>> = HashMap::new();
    for frequency in &gtfs.frequencies {
        windows
            .entry(frequency.trip_id.as_str())
            .or_default()
            .push((frequency.start_time, frequency.end_time));
    }
    for (trip_id, mut trip_windows) in windows {
        trip_windows.sort_unstable();
        let mut reach: Option<(u32, u32)> = None;
        for (start, end) in trip_windows {
            if let Some((previous_start, max_end)) = reach {
                if start > previous_start && start < max_end {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        file: "frequencies.txt",
                        entity_id: Some(trip_id.to_string()),
                        field: None,
                        rule: Rule::OverlappingFrequency,
                        message: format!(
                            "window starting at `{}` overlaps an earlier window ending at `{}`",
                            format_gtfs_time(start),
                            format_gtfs_time(max_end)
                        ),
                    });
                }
                reach = Some((start, max_end.max(end)));
            } else {
                reach = Some((start, end));
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
    use crate::misc::GtfsDate;
    use crate::model::TableName;
    use crate::model::{
        Agency, CalendarDate, ExceptionType, Frequency, Location, LocationGeometry, LocationGroup,
        Route, RouteNetwork, RouteType, ShapePoint, Stop, StopTime, Translation,
    };
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
    fn test_duplicate_location_ids() {
        let mut gtfs = GtfsReference::new();
        let ring = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]];
        gtfs.locations.push(Location::new(
            "zone",
            LocationGeometry::Polygon(vec![ring.clone()]),
        ));
        gtfs.locations
            .push(Location::new("zone", LocationGeometry::Polygon(vec![ring])));

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::DuplicateId
                    && issue.file == "locations.geojson"
                    && issue.entity_id.as_deref() == Some("zone"))
        );
    }

    #[test]
    fn test_composite_key_duplicates() -> Result<(), crate::GtfsError> {
        let mut gtfs = GtfsReference::new();
        gtfs.stop_times.push(StopTime::new("t0", "A", 5, 3600));
        gtfs.stop_times.push(StopTime::new("t0", "B", 5, 3700));
        gtfs.shapes.push(ShapePoint::new("sh1", 0.0, 0.0, 3));
        gtfs.shapes.push(ShapePoint::new("sh1", 1.0, 1.0, 3));
        gtfs.calendar_dates.push(CalendarDate::new(
            "svc",
            GtfsDate::new(2026, 1, 1)?,
            ExceptionType::Added,
        ));
        gtfs.calendar_dates.push(CalendarDate::new(
            "svc",
            GtfsDate::new(2026, 1, 1)?,
            ExceptionType::Removed,
        ));
        gtfs.frequencies.push(Frequency::new("t0", 3600, 7200, 300));
        gtfs.frequencies.push(Frequency::new("t0", 3600, 9000, 600));

        let report = gtfs.validate();
        let duplicate_files: Vec<&str> = report
            .issues()
            .iter()
            .filter(|issue| issue.rule == Rule::DuplicateId)
            .map(|issue| issue.file)
            .collect();
        assert!(duplicate_files.contains(&"stop_times.txt"));
        assert!(duplicate_files.contains(&"shapes.txt"));
        assert!(duplicate_files.contains(&"calendar_dates.txt"));
        assert!(duplicate_files.contains(&"frequencies.txt"));
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.message == "duplicate stop_sequence `5` for trip `t0`")
        );
        // identical start times are duplicates, not overlaps
        assert!(
            !report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::OverlappingFrequency)
        );
        Ok(())
    }

    #[test]
    fn test_overlapping_frequencies() {
        let mut gtfs = GtfsReference::new();
        gtfs.frequencies
            .push(Frequency::new("t0", 6 * 3600, 8 * 3600, 300));
        gtfs.frequencies
            .push(Frequency::new("t0", 7 * 3600, 9 * 3600, 600));
        // touching windows are legal
        gtfs.frequencies
            .push(Frequency::new("t1", 6 * 3600, 8 * 3600, 300));
        gtfs.frequencies
            .push(Frequency::new("t1", 8 * 3600, 10 * 3600, 600));

        let report = gtfs.validate();
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::OverlappingFrequency
                    && issue.entity_id.as_deref() == Some("t0"))
        );
        assert!(
            !report
                .issues()
                .iter()
                .any(|issue| issue.rule == Rule::OverlappingFrequency
                    && issue.entity_id.as_deref() == Some("t1"))
        );
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
