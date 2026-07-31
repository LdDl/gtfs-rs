//! Intra-record rules: the conditionally required/forbidden field
//! combinations of the specification, checked one record at a time.

use std::collections::HashMap;

use crate::model::{BookingType, LocationType, PathwayMode, StopTime, TransferType};
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
        if (stop_time.location_group_id.is_some() || stop_time.location_id.is_some())
            && window_parts == (false, false)
        {
            issues.push(error(
                "stop_times.txt",
                &entity,
                Rule::MissingPickupWindow,
                "a pickup/drop-off window is required with location_group_id or location_id",
            ));
        }
    }

    check_first_last_arrival(gtfs, issues);

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

    for (index, transfer) in gtfs.transfers.iter().enumerate() {
        let entity = format!("row {}", index + 1);
        match transfer.transfer_type {
            TransferType::InSeat | TransferType::InSeatNotAllowed => {
                for (field, value) in [
                    ("from_trip_id", &transfer.from_trip_id),
                    ("to_trip_id", &transfer.to_trip_id),
                ] {
                    if value.is_none() {
                        issues.push(field_error(
                            "transfers.txt",
                            &entity,
                            field,
                            Rule::MissingTransferTrip,
                            "required for in-seat transfer types 4 and 5",
                        ));
                    }
                }
            }
            _ => {
                for (field, value) in [
                    ("from_stop_id", &transfer.from_stop_id),
                    ("to_stop_id", &transfer.to_stop_id),
                ] {
                    if value.is_none() {
                        issues.push(field_error(
                            "transfers.txt",
                            &entity,
                            field,
                            Rule::MissingTransferStop,
                            "required for transfer types 0-3",
                        ));
                    }
                }
            }
        }
    }

    for pathway in &gtfs.pathways {
        if pathway.pathway_mode == PathwayMode::ExitGate && pathway.is_bidirectional {
            issues.push(error(
                "pathways.txt",
                &pathway.pathway_id,
                Rule::BidirectionalExitGate,
                "an exit gate (pathway_mode 7) must not be bidirectional",
            ));
        }
    }

    check_booking_rules(gtfs, issues);

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

/// `arrival_time` is required on the first and last stop time of a
/// trip (by `stop_sequence`); on-demand rows (a location group, a
/// GeoJSON location or a pickup/drop-off window) are exempt. The
/// spec also requires times for `timepoint = 1`, but an omitted
/// `timepoint` parses to the default [`Timepoint::Exact`], so an
/// explicit `1` cannot be told apart from an empty value; checking
/// it would false-flag interpolated-times feeds.
///
/// [`Timepoint::Exact`]: crate::model::Timepoint::Exact
fn check_first_last_arrival(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    let mut edges: HashMap<&str, (&StopTime, &StopTime)> = HashMap::new();
    for stop_time in &gtfs.stop_times {
        edges
            .entry(stop_time.trip_id.as_str())
            .and_modify(|(first, last)| {
                if stop_time.stop_sequence < first.stop_sequence {
                    *first = stop_time;
                }
                if stop_time.stop_sequence > last.stop_sequence {
                    *last = stop_time;
                }
            })
            .or_insert((stop_time, stop_time));
    }
    for (trip_id, (first, last)) in edges {
        let trip_edges = if first.stop_sequence == last.stop_sequence {
            vec![first]
        } else {
            vec![first, last]
        };
        for stop_time in trip_edges {
            let on_demand = stop_time.location_group_id.is_some()
                || stop_time.location_id.is_some()
                || stop_time.start_pickup_drop_off_window.is_some()
                || stop_time.end_pickup_drop_off_window.is_some();
            if !on_demand && stop_time.arrival_time.is_none() {
                issues.push(error(
                    "stop_times.txt",
                    &format!("{}#{}", trip_id, stop_time.stop_sequence),
                    Rule::MissingFirstLastArrivalTime,
                    "arrival_time is required for the first and last stop time of a trip",
                ));
            }
        }
    }
}

/// The conditionally required/forbidden matrix of
/// `booking_rules.txt` around `booking_type`: same-day booking (1)
/// requires `prior_notice_duration_min` and alone may carry
/// `prior_notice_duration_max`; prior-days booking (2) requires
/// `prior_notice_last_day` and alone may carry
/// `prior_notice_service_id`; `prior_notice_last_time` and
/// `prior_notice_start_time` go strictly together with their `_day`
/// counterparts; `prior_notice_start_day` is forbidden for real-time
/// booking (0) and for same-day booking combined with
/// `prior_notice_duration_max`.
fn check_booking_rules(gtfs: &GtfsReference, issues: &mut Vec<ValidationIssue>) {
    for booking in &gtfs.booking_rules {
        let id = &booking.booking_rule_id;
        let same_day = booking.booking_type == BookingType::SameDay;
        let prior_days = booking.booking_type == BookingType::PriorDays;
        if same_day && booking.prior_notice_duration_min.is_none() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_duration_min",
                Rule::MissingBookingField,
                "required for same-day booking (booking_type 1)",
            ));
        }
        if !same_day && booking.prior_notice_duration_min.is_some() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_duration_min",
                Rule::ForbiddenBookingField,
                "forbidden unless booking_type is 1",
            ));
        }
        if !same_day && booking.prior_notice_duration_max.is_some() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_duration_max",
                Rule::ForbiddenBookingField,
                "forbidden for booking_type 0 and 2",
            ));
        }
        if prior_days && booking.prior_notice_last_day.is_none() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_last_day",
                Rule::MissingBookingField,
                "required for prior-days booking (booking_type 2)",
            ));
        }
        if !prior_days && booking.prior_notice_last_day.is_some() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_last_day",
                Rule::ForbiddenBookingField,
                "forbidden unless booking_type is 2",
            ));
        }
        match (
            booking.prior_notice_last_day.is_some(),
            booking.prior_notice_last_time.is_some(),
        ) {
            (true, false) => issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_last_time",
                Rule::MissingBookingField,
                "required when prior_notice_last_day is defined",
            )),
            (false, true) => issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_last_time",
                Rule::ForbiddenBookingField,
                "forbidden without prior_notice_last_day",
            )),
            _ => {}
        }
        if booking.prior_notice_start_day.is_some() {
            if booking.booking_type == BookingType::RealTime {
                issues.push(field_error(
                    "booking_rules.txt",
                    id,
                    "prior_notice_start_day",
                    Rule::ForbiddenBookingField,
                    "forbidden for real-time booking (booking_type 0)",
                ));
            }
            if same_day && booking.prior_notice_duration_max.is_some() {
                issues.push(field_error(
                    "booking_rules.txt",
                    id,
                    "prior_notice_start_day",
                    Rule::ForbiddenBookingField,
                    "forbidden for booking_type 1 when prior_notice_duration_max is defined",
                ));
            }
        }
        match (
            booking.prior_notice_start_day.is_some(),
            booking.prior_notice_start_time.is_some(),
        ) {
            (true, false) => issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_start_time",
                Rule::MissingBookingField,
                "required when prior_notice_start_day is defined",
            )),
            (false, true) => issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_start_time",
                Rule::ForbiddenBookingField,
                "forbidden without prior_notice_start_day",
            )),
            _ => {}
        }
        if !prior_days && booking.prior_notice_service_id.is_some() {
            issues.push(field_error(
                "booking_rules.txt",
                id,
                "prior_notice_service_id",
                Rule::ForbiddenBookingField,
                "forbidden unless booking_type is 2",
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

/// Builds an error-severity issue for one field of a record.
fn field_error(
    file: &'static str,
    entity_id: &str,
    field: &str,
    rule: Rule,
    message: &str,
) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        file,
        entity_id: Some(entity_id.to_string()),
        field: Some(field.to_string()),
        rule,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::{
        Attribution, BookingRule, BookingType, Calendar, Frequency, LocationType, Pathway,
        PathwayMode, Route, RouteType, Stop, StopTime, Timeframe, Transfer, TransferType,
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
    fn test_flex_row_requires_window() {
        let mut gtfs = GtfsReference::new();
        let mut flex = StopTime::new("t0", "A", 1, 3600);
        flex.stop_id = None;
        flex.arrival_time = None;
        flex.departure_time = None;
        flex.location_id = Some("zone".to_string());
        gtfs.stop_times.push(flex);

        let rules = rules_of(&gtfs);
        assert!(rules.contains(&Rule::MissingPickupWindow));
        // on-demand rows are exempt from the first/last requirement
        assert!(!rules.contains(&Rule::MissingFirstLastArrivalTime));
    }

    #[test]
    fn test_first_and_last_stop_times_require_arrival() {
        let mut gtfs = GtfsReference::new();
        let mut first = StopTime::new("t0", "A", 1, 3600);
        first.arrival_time = None;
        first.departure_time = None;
        gtfs.stop_times.push(first);
        let mut middle = StopTime::new("t0", "B", 2, 3700);
        middle.arrival_time = None;
        middle.departure_time = None;
        gtfs.stop_times.push(middle);
        gtfs.stop_times.push(StopTime::new("t0", "C", 3, 3800));

        let flagged: Vec<_> = gtfs
            .validate()
            .issues()
            .iter()
            .filter(|issue| issue.rule == Rule::MissingFirstLastArrivalTime)
            .map(|issue| issue.entity_id.clone())
            .collect();
        // only the first row lacks a required arrival_time; the
        // interpolated middle row is legal
        assert_eq!(flagged, [Some("t0#1".to_string())]);
    }

    #[test]
    fn test_transfer_stop_pair_required_for_types_0_to_3() {
        let mut gtfs = GtfsReference::new();
        gtfs.transfers
            .push(Transfer::new(TransferType::MinimumTime));

        let fields: Vec<Option<String>> = gtfs
            .validate()
            .into_iter()
            .filter(|issue| issue.rule == Rule::MissingTransferStop)
            .map(|issue| issue.field)
            .collect();
        assert_eq!(
            fields,
            [
                Some("from_stop_id".to_string()),
                Some("to_stop_id".to_string())
            ]
        );
    }

    #[test]
    fn test_transfer_trip_pair_required_for_in_seat() {
        let mut gtfs = GtfsReference::new();
        gtfs.transfers.push(Transfer::new(TransferType::InSeat));

        let rules = rules_of(&gtfs);
        assert!(rules.contains(&Rule::MissingTransferTrip));
        // the stop pair is optional for in-seat transfer types
        assert!(!rules.contains(&Rule::MissingTransferStop));
    }

    #[test]
    fn test_complete_transfers_pass() {
        let mut gtfs = GtfsReference::new();
        gtfs.transfers
            .push(Transfer::new(TransferType::Recommended).between_stops("A", "B"));
        gtfs.transfers
            .push(Transfer::new(TransferType::InSeatNotAllowed).between_trips("t1", "t2"));

        let rules = rules_of(&gtfs);
        assert!(!rules.contains(&Rule::MissingTransferStop));
        assert!(!rules.contains(&Rule::MissingTransferTrip));
    }

    #[test]
    fn test_exit_gate_must_not_be_bidirectional() {
        let mut gtfs = GtfsReference::new();
        gtfs.pathways
            .push(Pathway::new("pw1", "A", "B", PathwayMode::ExitGate, true));
        gtfs.pathways
            .push(Pathway::new("pw2", "A", "B", PathwayMode::FareGate, true));
        gtfs.pathways
            .push(Pathway::new("pw3", "A", "B", PathwayMode::ExitGate, false));

        let flagged: Vec<Option<String>> = gtfs
            .validate()
            .into_iter()
            .filter(|issue| issue.rule == Rule::BidirectionalExitGate)
            .map(|issue| issue.entity_id)
            .collect();
        assert_eq!(flagged, [Some("pw1".to_string())]);
    }

    #[test]
    fn test_booking_required_fields() {
        let mut gtfs = GtfsReference::new();
        gtfs.booking_rules
            .push(BookingRule::new("same_day", BookingType::SameDay));
        gtfs.booking_rules
            .push(BookingRule::new("prior_days", BookingType::PriorDays));
        let mut half_last = BookingRule::new("half_last", BookingType::PriorDays);
        half_last.prior_notice_last_day = Some(1);
        gtfs.booking_rules.push(half_last);

        let missing: Vec<(Option<String>, Option<String>)> = gtfs
            .validate()
            .into_iter()
            .filter(|issue| issue.rule == Rule::MissingBookingField)
            .map(|issue| (issue.entity_id, issue.field))
            .collect();
        assert_eq!(
            missing,
            [
                (
                    Some("same_day".to_string()),
                    Some("prior_notice_duration_min".to_string())
                ),
                (
                    Some("prior_days".to_string()),
                    Some("prior_notice_last_day".to_string())
                ),
                (
                    Some("half_last".to_string()),
                    Some("prior_notice_last_time".to_string())
                ),
            ]
        );
    }

    #[test]
    fn test_booking_forbidden_fields() {
        let mut gtfs = GtfsReference::new();
        let mut real_time = BookingRule::new("real_time", BookingType::RealTime);
        real_time.prior_notice_duration_min = Some(30);
        real_time.prior_notice_duration_max = Some(120);
        real_time.prior_notice_start_day = Some(2);
        real_time.prior_notice_start_time = Some(8 * 3600);
        real_time.prior_notice_service_id = Some("svc".to_string());
        gtfs.booking_rules.push(real_time);
        let mut same_day = BookingRule::new("same_day", BookingType::SameDay);
        same_day.prior_notice_duration_min = Some(30);
        same_day.prior_notice_duration_max = Some(120);
        same_day.prior_notice_start_day = Some(2);
        same_day.prior_notice_start_time = Some(8 * 3600);
        same_day.prior_notice_last_time = Some(17 * 3600);
        gtfs.booking_rules.push(same_day);

        let forbidden: Vec<(Option<String>, Option<String>)> = gtfs
            .validate()
            .into_iter()
            .filter(|issue| issue.rule == Rule::ForbiddenBookingField)
            .map(|issue| (issue.entity_id, issue.field))
            .collect();
        assert_eq!(
            forbidden,
            [
                (
                    Some("real_time".to_string()),
                    Some("prior_notice_duration_min".to_string())
                ),
                (
                    Some("real_time".to_string()),
                    Some("prior_notice_duration_max".to_string())
                ),
                (
                    Some("real_time".to_string()),
                    Some("prior_notice_start_day".to_string())
                ),
                (
                    Some("real_time".to_string()),
                    Some("prior_notice_service_id".to_string())
                ),
                (
                    Some("same_day".to_string()),
                    Some("prior_notice_last_time".to_string())
                ),
                (
                    Some("same_day".to_string()),
                    Some("prior_notice_start_day".to_string())
                ),
            ]
        );
    }

    #[test]
    fn test_booking_fields_allowed_for_their_type() -> Result<(), crate::GtfsError> {
        let mut gtfs = GtfsReference::new();
        gtfs.calendar.push(Calendar::new(
            "svc",
            GtfsDate::new(2026, 1, 1)?,
            GtfsDate::new(2026, 12, 31)?,
        ));
        let mut same_day = BookingRule::new("same_day", BookingType::SameDay);
        same_day.prior_notice_duration_min = Some(30);
        same_day.prior_notice_duration_max = Some(120);
        gtfs.booking_rules.push(same_day);
        let mut prior_days = BookingRule::new("prior_days", BookingType::PriorDays);
        prior_days.prior_notice_last_day = Some(1);
        prior_days.prior_notice_last_time = Some(17 * 3600);
        prior_days.prior_notice_start_day = Some(7);
        prior_days.prior_notice_start_time = Some(0);
        prior_days.prior_notice_service_id = Some("svc".to_string());
        gtfs.booking_rules.push(prior_days);

        let rules = rules_of(&gtfs);
        assert!(!rules.contains(&Rule::MissingBookingField));
        assert!(!rules.contains(&Rule::ForbiddenBookingField));
        Ok(())
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
