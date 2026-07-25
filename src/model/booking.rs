//! `booking_rules.txt` - how riders book on-demand service
//! (GTFS-Flex).
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#booking_rulestxt>

gtfs_enum! {
    /// Indicates how far in advance booking can be made
    /// (`booking_type`). Required in `booking_rules.txt`.
    BookingType {
        /// Real time booking (`0`)
        RealTime = 0,
        /// Up to same-day booking with advance notice (`1`)
        SameDay = 1,
        /// Up to prior day(s) booking (`2`)
        PriorDays = 2,
    }
}

/// A booking rule from `booking_rules.txt`.
///
/// Defines the booking rules for rider-requested services. The file
/// is optional; its primary key is (`booking_rule_id`).
///
/// # Examples
///
/// ```
/// use gtfs_rs::{BookingRule, BookingType};
///
/// let mut rule = BookingRule::new("br1", BookingType::SameDay)
///     .with_phone_number("+7 495 000-00-00");
/// // remaining optional fields are set directly
/// rule.prior_notice_duration_min = Some(60);
/// assert_eq!(rule.booking_type.code(), 1);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BookingRule {
    /// Identifies a rule. Required.
    pub booking_rule_id: String,
    /// Indicates how far in advance booking can be made; see
    /// [`BookingType`] for the valid options. Required.
    pub booking_type: BookingType,
    /// Minimum number of minutes before travel to make the request.
    ///
    /// Conditionally Required:
    /// - Required for `booking_type=1` (up to same-day booking).
    /// - Forbidden otherwise.
    ///
    /// `None` when the value is empty in the file.
    pub prior_notice_duration_min: Option<u32>,
    /// Maximum number of minutes before travel to make the booking
    /// request.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden for `booking_type=0` and `booking_type=2`.
    /// - Optional for `booking_type=1`.
    ///
    /// `None` when the value is empty in the file.
    pub prior_notice_duration_max: Option<u32>,
    /// Last day before travel to make the booking request.
    ///
    /// Example: "Ride must be booked 1 day in advance before 5PM"
    /// will be encoded as `prior_notice_last_day=1`.
    ///
    /// Conditionally Required:
    /// - Required for `booking_type=2`.
    /// - Forbidden otherwise.
    ///
    /// `None` when the value is empty in the file.
    pub prior_notice_last_day: Option<u32>,
    /// Last time on the last day before travel to make the booking
    /// request, as seconds since midnight in this crate (the GTFS
    /// field is a `HH:MM:SS` time).
    ///
    /// Example: "Ride must be booked 1 day in advance before 5PM"
    /// will be encoded as `prior_notice_last_time=17:00:00`.
    ///
    /// Conditionally Required:
    /// - Required if `prior_notice_last_day` is defined.
    /// - Forbidden otherwise.
    pub prior_notice_last_time: Option<u32>,
    /// Earliest day before travel to make the booking request.
    ///
    /// Example: "Ride can be booked at the earliest one week in
    /// advance at midnight" will be encoded as
    /// `prior_notice_start_day=7`.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden for `booking_type=0`.
    /// - Forbidden for `booking_type=1` if
    ///   `prior_notice_duration_max` is defined.
    /// - Optional otherwise.
    pub prior_notice_start_day: Option<u32>,
    /// Earliest time on the earliest day before travel to make the
    /// booking request, as seconds since midnight in this crate
    /// (the GTFS field is a `HH:MM:SS` time).
    ///
    /// Example: "Ride can be booked at the earliest one week in
    /// advance at midnight" will be encoded as
    /// `prior_notice_start_time=00:00:00`.
    ///
    /// Conditionally Required:
    /// - Required if `prior_notice_start_day` is defined.
    /// - Forbidden otherwise.
    pub prior_notice_start_time: Option<u32>,
    /// Indicates the service days on which `prior_notice_last_day`
    /// or `prior_notice_start_day` are counted. Foreign ID
    /// referencing `calendar.service_id`.
    ///
    /// Example: If empty, `prior_notice_start_day=2` will be two
    /// calendar days in advance. If defined as a `service_id`
    /// containing only business days (weekdays without holidays),
    /// `prior_notice_start_day=2` will be two business days in
    /// advance.
    ///
    /// Conditionally Forbidden:
    /// - Optional if `booking_type=2`.
    /// - Forbidden otherwise.
    pub prior_notice_service_id: Option<String>,
    /// Message to riders utilizing service at a `stop_time` when
    /// booking on-demand pickup and drop off. Meant to provide
    /// minimal information to be transmitted within a user
    /// interface about the action a rider must take in order to
    /// utilize the service. Optional.
    pub message: Option<String>,
    /// Functions in the same way as `message` but used when riders
    /// have on-demand pickup only. Optional.
    pub pickup_message: Option<String>,
    /// Functions in the same way as `message` but used when riders
    /// have on-demand drop off only. Optional.
    pub drop_off_message: Option<String>,
    /// Phone number to call to make the booking request. Optional.
    pub phone_number: Option<String>,
    /// URL providing information about the booking rule. Optional.
    pub info_url: Option<String>,
    /// URL to an online interface or app where the booking request
    /// can be made. Optional.
    pub booking_url: Option<String>,
}

impl BookingRule {
    /// Creates a booking rule.
    ///
    /// # Arguments
    ///
    /// * `booking_rule_id` - Unique booking rule identifier
    /// * `booking_type` - How far in advance booking is required
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{BookingRule, BookingType};
    ///
    /// let rule = BookingRule::new("br1", BookingType::SameDay);
    /// assert_eq!(rule.booking_rule_id, "br1");
    /// assert!(rule.phone_number.is_none());
    /// ```
    pub fn new(booking_rule_id: &str, booking_type: BookingType) -> Self {
        BookingRule {
            booking_rule_id: booking_rule_id.to_string(),
            booking_type,
            prior_notice_duration_min: None,
            prior_notice_duration_max: None,
            prior_notice_last_day: None,
            prior_notice_last_time: None,
            prior_notice_start_day: None,
            prior_notice_start_time: None,
            prior_notice_service_id: None,
            message: None,
            pickup_message: None,
            drop_off_message: None,
            phone_number: None,
            info_url: None,
            booking_url: None,
        }
    }

    /// Sets the phone number to call to make the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{BookingRule, BookingType};
    ///
    /// let rule = BookingRule::new("br1", BookingType::SameDay)
    ///     .with_phone_number("+7 495 000-00-00");
    /// assert_eq!(rule.phone_number.as_deref(), Some("+7 495 000-00-00"));
    /// ```
    pub fn with_phone_number(mut self, phone_number: &str) -> Self {
        self.phone_number = Some(phone_number.to_string());
        self
    }

    /// Sets the URL of the online booking interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{BookingRule, BookingType};
    ///
    /// let rule = BookingRule::new("br1", BookingType::SameDay)
    ///     .with_booking_url("https://book.example");
    /// assert_eq!(rule.booking_url.as_deref(), Some("https://book.example"));
    /// ```
    pub fn with_booking_url(mut self, booking_url: &str) -> Self {
        self.booking_url = Some(booking_url.to_string());
        self
    }
}
