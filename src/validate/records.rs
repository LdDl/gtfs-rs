//! Intra-record rules: the conditionally required/forbidden field
//! combinations of the specification, checked one record at a time.

use crate::model::LocationType;
use crate::reference::GtfsReference;
use crate::validate::report::{Rule, Severity, ValidationIssue};

/// Runs every intra-record rule.
pub fn check(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    for route in &gtfs.routes {
        if route.route_short_name.is_none() && route.route_long_name.is_none() {
            issues.push(error(
                "routes.txt",
                &route.route_id,
                Rule::MissingRouteName,
                "neither route_short_name nor route_long_name is set",
            ));
        }
    }

    for stop in &gtfs.stops {
        let named_kind = matches!(
            stop.location_type,
            LocationType::StopOrPlatform | LocationType::Station | LocationType::EntranceExit
        );
        if named_kind && stop.stop_name.is_none() {
            issues.push(error(
                "stops.txt",
                &stop.stop_id,
                Rule::MissingStopName,
                "stop_name is required for stops, stations and entrances",
            ));
        }
        if named_kind && (stop.stop_lat.is_none() || stop.stop_lon.is_none()) {
            issues.push(error(
                "stops.txt",
                &stop.stop_id,
                Rule::MissingStopCoordinates,
                "stop_lat/stop_lon are required for stops, stations and entrances",
            ));
        }
        match stop.location_type {
            LocationType::Station if stop.parent_station.is_some() => {
                issues.push(error(
                    "stops.txt",
                    &stop.stop_id,
                    Rule::ForbiddenParentStation,
                    "a station must not have a parent_station",
                ));
            }
            LocationType::EntranceExit | LocationType::GenericNode | LocationType::BoardingArea
                if stop.parent_station.is_none() =>
            {
                issues.push(error(
                    "stops.txt",
                    &stop.stop_id,
                    Rule::MissingParentStation,
                    "entrances, generic nodes and boarding areas require a parent_station",
                ));
            }
            _ => {}
        }
    }

    for stop_time in &gtfs.stop_times {
        let entity = format!("{}#{}", stop_time.trip_id, stop_time.stop_sequence);
        let references = [
            stop_time.stop_id.is_some(),
            stop_time.location_group_id.is_some(),
            stop_time.location_id.is_some(),
        ]
        .iter()
        .filter(|set| **set)
        .count();
        if references == 0 {
            issues.push(error(
                "stop_times.txt",
                &entity,
                Rule::MissingStopTimeLocation,
                "one of stop_id, location_group_id, location_id must be set",
            ));
        }
        if references > 1 {
            issues.push(error(
                "stop_times.txt",
                &entity,
                Rule::ConflictingStopTimeLocation,
                "only one of stop_id, location_group_id, location_id may be set",
            ));
        }
        let window_parts = (
            stop_time.start_pickup_drop_off_window.is_some(),
            stop_time.end_pickup_drop_off_window.is_some(),
        );
        match window_parts {
            (true, false) | (false, true) => {
                issues.push(error(
                    "stop_times.txt",
                    &entity,
                    Rule::IncompleteWindow,
                    "start and end of the pickup/drop-off window must be set together",
                ));
            }
            (true, true)
                if stop_time.arrival_time.is_some() || stop_time.departure_time.is_some() =>
            {
                issues.push(error(
                    "stop_times.txt",
                    &entity,
                    Rule::TimesWithPickupWindow,
                    "arrival/departure times are forbidden with a pickup/drop-off window",
                ));
            }
            _ => {}
        }
    }

    for timeframe in &gtfs.timeframes {
        if timeframe.start_time.is_some() != timeframe.end_time.is_some() {
            issues.push(error(
                "timeframes.txt",
                &timeframe.timeframe_group_id,
                Rule::IncompleteTimeframe,
                "start_time and end_time must be set together or both omitted",
            ));
        }
    }

    for calendar in &gtfs.calendar {
        if calendar.start_date > calendar.end_date {
            issues.push(error(
                "calendar.txt",
                &calendar.service_id,
                Rule::InvalidServicePeriod,
                "start_date is after end_date",
            ));
        }
    }

    for frequency in &gtfs.frequencies {
        if frequency.start_time >= frequency.end_time {
            issues.push(error(
                "frequencies.txt",
                &frequency.trip_id,
                Rule::InvalidFrequencyWindow,
                "start_time must be before end_time",
            ));
        }
    }

    for attribution in &gtfs.attributions {
        let entity = attribution
            .attribution_id
            .clone()
            .unwrap_or_else(|| attribution.organization_name.clone());
        if !(attribution.is_producer || attribution.is_operator || attribution.is_authority) {
            issues.push(error(
                "attributions.txt",
                &entity,
                Rule::AttributionRoleMissing,
                "at least one of is_producer, is_operator, is_authority must be set",
            ));
        }
        let targets = [
            attribution.agency_id.is_some(),
            attribution.route_id.is_some(),
            attribution.trip_id.is_some(),
        ]
        .iter()
        .filter(|set| **set)
        .count();
        if targets > 1 {
            issues.push(error(
                "attributions.txt",
                &entity,
                Rule::AttributionMultipleTargets,
                "at most one of agency_id, route_id, trip_id may be set",
            ));
        }
    }
}

/// Builds an error-severity issue for one record.
fn error(file: &'static str, entity_id: &str, rule: Rule, message: &str) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        file,
        entity_id: Some(entity_id.to_string()),
        field: None,
        rule,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::{
        Attribution, Calendar, Frequency, LocationType, Route, RouteType, Stop, StopTime, Timeframe,
    };
    use crate::reference::GtfsReference;
    use crate::validate::report::Rule;

    fn rules_of(gtfs: &GtfsReference) -> Vec<Rule> {
        gtfs.validate()
            .issues()
            .iter()
            .map(|issue| issue.rule)
            .collect()
    }

    #[test]
    fn test_route_and_stop_rules() {
        let mut gtfs = GtfsReference::new();
        gtfs.routes.push(Route::new("L1", RouteType::Bus));
        gtfs.stops.push(Stop::new("A"));
        gtfs.stops
            .push(Stop::new("E1").with_location_type(LocationType::EntranceExit));
        gtfs.stops.push(
            Stop::new("S1")
                .with_name("Central")
                .with_coordinates(55.0, 37.0)
                .with_location_type(LocationType::Station)
                .with_parent_station("S2"),
        );

        let rules = rules_of(&gtfs);
        assert!(rules.contains(&Rule::MissingRouteName));
        assert!(rules.contains(&Rule::MissingStopName));
        assert!(rules.contains(&Rule::MissingStopCoordinates));
        assert!(rules.contains(&Rule::MissingParentStation));
        assert!(rules.contains(&Rule::ForbiddenParentStation));
    }

    #[test]
    fn test_stop_time_rules() {
        let mut gtfs = GtfsReference::new();
        // no stop reference at all
        let mut orphan = StopTime::new("t0", "A", 1, 3600);
        orphan.stop_id = None;
        gtfs.stop_times.push(orphan);
        // both a stop and a flex location
        let mut double = StopTime::new("t0", "A", 2, 3600);
        double.location_id = Some("zone".to_string());
        gtfs.stop_times.push(double);
        // a window plus explicit times
        let mut windowed = StopTime::new("t0", "A", 3, 3600);
        windowed.start_pickup_drop_off_window = Some(8 * 3600);
        windowed.end_pickup_drop_off_window = Some(18 * 3600);
        gtfs.stop_times.push(windowed);
        // half a window
        let mut half = StopTime::new("t0", "A", 4, 3600);
        half.arrival_time = None;
        half.departure_time = None;
        half.start_pickup_drop_off_window = Some(8 * 3600);
        gtfs.stop_times.push(half);

        let rules = rules_of(&gtfs);
        assert!(rules.contains(&Rule::MissingStopTimeLocation));
        assert!(rules.contains(&Rule::ConflictingStopTimeLocation));
        assert!(rules.contains(&Rule::TimesWithPickupWindow));
        assert!(rules.contains(&Rule::IncompleteWindow));
    }

    #[test]
    fn test_period_window_and_attribution_rules() -> Result<(), crate::GtfsError> {
        let mut gtfs = GtfsReference::new();
        gtfs.calendar.push(Calendar::new(
            "svc",
            GtfsDate::new(2026, 12, 31)?,
            GtfsDate::new(2026, 1, 1)?,
        ));
        gtfs.frequencies.push(Frequency::new("t0", 3600, 3600, 300));
        gtfs.timeframes
            .push(Timeframe::new("peak", "svc").with_period(0, 3600));
        let mut half_open = Timeframe::new("peak", "svc");
        half_open.start_time = Some(0);
        gtfs.timeframes.push(half_open);
        gtfs.attributions.push(Attribution::new("Nobody"));
        let mut double_target = Attribution::new("Both").as_producer();
        double_target.agency_id = Some("A".to_string());
        double_target.route_id = Some("R".to_string());
        gtfs.attributions.push(double_target);

        let rules = rules_of(&gtfs);
        assert!(rules.contains(&Rule::InvalidServicePeriod));
        assert!(rules.contains(&Rule::InvalidFrequencyWindow));
        assert!(rules.contains(&Rule::IncompleteTimeframe));
        assert!(rules.contains(&Rule::AttributionRoleMissing));
        assert!(rules.contains(&Rule::AttributionMultipleTargets));
        Ok(())
    }
}
