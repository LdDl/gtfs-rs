//! Fare files.
//!
//! GTFS has two fare frameworks that may coexist in one dataset:
//!
//! - the legacy zone/route based **GTFS-Fares v1**
//!   (`fare_attributes.txt`, `fare_rules.txt`) - its structs carry
//!   a `V1` name suffix: [`FareAttributeV1`], [`FareRuleV1`];
//! - the current product-based framework, informally "Fares v2"
//!   (`timeframes.txt`, `rider_categories.txt`, `fare_media.txt`,
//!   `fare_products.txt`, `fare_leg_rules.txt`,
//!   `fare_leg_join_rules.txt`, `fare_transfer_rules.txt`, plus
//!   `areas.txt` and `networks.txt` in their own modules) - its
//!   structs use plain, unsuffixed names.
//!
//! When a dataset provides both, consumers should prefer the v2
//! data.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#fare_attributestxt>
//!
//! # Examples
//!
//! A Fares v2 fragment: a single-ride product valid on the metro
//! network, with free transfers within 90 minutes:
//!
//! ```
//! fn main() -> Result<(), gtfs_rs::GtfsError> {
//!     use gtfs_rs::{
//!         CurrencyAmount, DurationLimitType, FareLegRule, FareProduct, FareTransferRule,
//!         FareTransferType,
//!     };
//!
//!     let single = FareProduct::new("single", CurrencyAmount::parse("57.00")?, "RUB")
//!         .with_name("Single ride");
//!     let leg = FareLegRule::new("single")
//!         .with_leg_group("metro_leg")
//!         .with_network("metro");
//!     let free_transfer = FareTransferRule::new(FareTransferType::FromLegPlusTransfer)
//!         .between_leg_groups("metro_leg", "metro_leg")
//!         .with_duration_limit(90 * 60, DurationLimitType::DepartureToArrival);
//!
//!     assert_eq!(leg.fare_product_id, single.fare_product_id);
//!     assert_eq!(free_transfer.duration_limit, Some(5400));
//!     Ok(())
//! }
//! ```

use crate::misc::CurrencyAmount;

gtfs_enum! {
    /// When a fare must be paid (`payment_method`).
    PaymentMethod {
        /// Fare is paid on board (`0`)
        OnBoard = 0,
        /// Fare must be paid before boarding the vehicle (`1`)
        BeforeBoarding = 1,
    }
}

/// Number of transfers permitted on a fare (`transfers`).
///
/// An empty value in `fare_attributes.txt` means unlimited transfers,
/// which is why this enum encodes to an `Option` rather than a plain
/// code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FareTransfers {
    /// No transfers permitted on this fare (`0`)
    NotAllowed,
    /// Riders may transfer once (`1`)
    Once,
    /// Riders may transfer twice (`2`)
    Twice,
    /// Unlimited transfers are permitted (empty value in the file,
    /// default)
    #[default]
    Unlimited,
}

impl FareTransfers {
    /// Parses the numeric code used in GTFS files; `None` (an empty
    /// field) means unlimited transfers.
    pub fn from_code(code: Option<i32>) -> Option<Self> {
        match code {
            Some(0) => Some(FareTransfers::NotAllowed),
            Some(1) => Some(FareTransfers::Once),
            Some(2) => Some(FareTransfers::Twice),
            None => Some(FareTransfers::Unlimited),
            Some(_) => None,
        }
    }

    /// Returns the numeric code used in GTFS files; `None` means the
    /// field is left empty (unlimited transfers).
    pub fn code(self) -> Option<i32> {
        match self {
            FareTransfers::NotAllowed => Some(0),
            FareTransfers::Once => Some(1),
            FareTransfers::Twice => Some(2),
            FareTransfers::Unlimited => None,
        }
    }
}

/// A fare class from `fare_attributes.txt` (GTFS-Fares v1).
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::{CurrencyAmount, FareAttributeV1, FareTransfers, PaymentMethod};
///
///     let fare = FareAttributeV1::new(
///         "base",
///         CurrencyAmount::parse("57.00")?,
///         "RUB",
///         PaymentMethod::OnBoard,
///     )
///     .with_transfers(FareTransfers::NotAllowed);
///     assert_eq!(fare.transfers.code(), Some(0));
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareAttributeV1 {
    /// Identifies a fare class. Unique ID. Required.
    pub fare_id: String,
    /// Fare price, in the unit specified by `currency_type`. The
    /// spec declares a non-negative float, but the crate stores it
    /// exactly as a [`CurrencyAmount`] decimal to avoid
    /// floating-point rounding on money. Required.
    pub price: CurrencyAmount,
    /// Currency used to pay the fare, as an ISO 4217 currency code
    /// (e.g. "RUB", "USD"). Required.
    pub currency_type: String,
    /// Indicates when the fare must be paid. Required. See
    /// [`PaymentMethod`] for the valid options.
    pub payment_method: PaymentMethod,
    /// Indicates the number of transfers permitted on this fare.
    /// Required; an empty value in the file means unlimited
    /// transfers are permitted, which [`FareTransfers`] encodes via
    /// `Option` codes rather than a plain numeric code.
    pub transfers: FareTransfers,
    /// Identifies the relevant agency for a fare. Foreign ID
    /// referencing `agency.agency_id`.
    ///
    /// Conditionally Required:
    /// - Required if multiple agencies are defined in `agency.txt`.
    /// - Recommended otherwise.
    ///
    /// `None` means the value may be empty in the file.
    pub agency_id: Option<String>,
    /// Length of time in seconds before a transfer expires. When
    /// `transfers` = `0` this field may be used to indicate how long
    /// a ticket is valid for or it may be left empty. Non-negative
    /// integer. Optional; `None` means the value is empty in the
    /// file.
    pub transfer_duration: Option<u32>,
}

impl FareAttributeV1 {
    /// Creates a fare class with unlimited transfers.
    ///
    /// # Arguments
    ///
    /// * `fare_id` - Unique fare identifier
    /// * `price` - Fare price
    /// * `currency_type` - ISO 4217 currency code
    /// * `payment_method` - When the fare must be paid
    pub fn new(
        fare_id: &str,
        price: CurrencyAmount,
        currency_type: &str,
        payment_method: PaymentMethod,
    ) -> Self {
        FareAttributeV1 {
            fare_id: fare_id.to_string(),
            price,
            currency_type: currency_type.to_string(),
            payment_method,
            transfers: FareTransfers::default(),
            agency_id: None,
            transfer_duration: None,
        }
    }

    /// Sets the permitted number of transfers.
    pub fn with_transfers(mut self, transfers: FareTransfers) -> Self {
        self.transfers = transfers;
        self
    }
}

/// A fare applicability rule from `fare_rules.txt` (GTFS-Fares v1).
///
/// A fare applies based on any combination of route, origin zone,
/// destination zone and traversed zones (`stops.zone_id`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareRuleV1 {
    /// Identifies a fare class. Foreign ID referencing
    /// `fare_attributes.fare_id`. Required.
    pub fare_id: String,
    /// Identifies a route associated with the fare class. If
    /// several routes with the same fare attributes exist, create a
    /// record in `fare_rules.txt` for each route. Foreign ID
    /// referencing `routes.route_id`. Optional; `None` means the
    /// value is empty in the file.
    pub route_id: Option<String>,
    /// Identifies an origin zone. If a fare class has multiple
    /// origin zones, create a record in `fare_rules.txt` for each
    /// `origin_id`. Foreign ID referencing `stops.zone_id`.
    /// Optional; `None` means the value is empty in the file.
    pub origin_id: Option<String>,
    /// Identifies a destination zone. If a fare class has multiple
    /// destination zones, create a record in `fare_rules.txt` for
    /// each `destination_id`. May be used together with `origin_id`
    /// to specify zone pairs the fare class is valid for. Foreign
    /// ID referencing `stops.zone_id`. Optional; `None` means the
    /// value is empty in the file.
    pub destination_id: Option<String>,
    /// Identifies the zones that a rider will enter while using a
    /// given fare class. Used in some systems to calculate correct
    /// fare class. Because all `contains_id` zones must be matched
    /// for the fare to apply, an itinerary that passes through only
    /// some of the listed zones does not match the fare class.
    /// Foreign ID referencing `stops.zone_id`. Optional; `None`
    /// means the value is empty in the file.
    pub contains_id: Option<String>,
}

impl FareRuleV1 {
    /// Creates a fare rule for a fare class.
    ///
    /// # Arguments
    ///
    /// * `fare_id` - Fare class the rule applies to
    pub fn new(fare_id: &str) -> Self {
        FareRuleV1 {
            fare_id: fare_id.to_string(),
            route_id: None,
            origin_id: None,
            destination_id: None,
            contains_id: None,
        }
    }

    /// Restricts the rule to a route.
    pub fn with_route_id(mut self, route_id: &str) -> Self {
        self.route_id = Some(route_id.to_string());
        self
    }

    /// Restricts the rule to an origin-destination zone pair.
    ///
    /// # Arguments
    ///
    /// * `origin_id` - Origin zone
    /// * `destination_id` - Destination zone
    pub fn with_origin_destination(mut self, origin_id: &str, destination_id: &str) -> Self {
        self.origin_id = Some(origin_id.to_string());
        self.destination_id = Some(destination_id.to_string());
        self
    }
}

/// A fare timeframe from `timeframes.txt` (GTFS-Fares v2).
///
/// Describes periods of time (e.g. peak hours) that fare rules can
/// depend on. `start_time` and `end_time` are set together or both
/// omitted; omitting both means the whole service day.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Timeframe {
    /// Identifies a timeframe or set of timeframes. Required.
    pub timeframe_group_id: String,
    /// Defines the beginning of a timeframe. The interval includes
    /// the start time. Values greater than `24:00:00` are
    /// forbidden. An empty value in `start_time` is considered
    /// `00:00:00`. Stored as seconds since midnight in a `u32`.
    ///
    /// Conditionally Required:
    /// - Required if `end_time` is defined.
    /// - Forbidden otherwise.
    pub start_time: Option<u32>,
    /// Defines the end of a timeframe. The interval does not
    /// include the end time. Values greater than `24:00:00` are
    /// forbidden. An empty value in `end_time` is considered
    /// `24:00:00`. Stored as seconds since midnight in a `u32`.
    ///
    /// Conditionally Required:
    /// - Required if `start_time` is defined.
    /// - Forbidden otherwise.
    pub end_time: Option<u32>,
    /// Identifies a set of dates that a timeframe is in effect.
    /// Foreign ID referencing `calendar.service_id` or
    /// `calendar_dates.service_id`. Required.
    pub service_id: String,
}

impl Timeframe {
    /// Creates a timeframe covering the whole service day.
    ///
    /// # Arguments
    ///
    /// * `timeframe_group_id` - Timeframe group identifier
    /// * `service_id` - Service dates the timeframe applies on
    pub fn new(timeframe_group_id: &str, service_id: &str) -> Self {
        Timeframe {
            timeframe_group_id: timeframe_group_id.to_string(),
            start_time: None,
            end_time: None,
            service_id: service_id.to_string(),
        }
    }

    /// Sets the period within the service day.
    ///
    /// # Arguments
    ///
    /// * `start_time` - Period start, seconds since midnight (inclusive)
    /// * `end_time` - Period end, seconds since midnight (exclusive)
    pub fn with_period(mut self, start_time: u32, end_time: u32) -> Self {
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }
}

/// A rider category from `rider_categories.txt` (GTFS-Fares v2),
/// e.g. adult, child, student.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RiderCategory {
    /// Identifies a rider category. Unique ID. Required.
    pub rider_category_id: String,
    /// Rider category name as displayed to the rider. Required.
    pub rider_category_name: String,
    /// Specifies if an entry in `rider_categories.txt` should be
    /// considered the default category (i.e. the main category that
    /// should be displayed to riders), for example an adult or
    /// regular fare. Required. In the file `0` or empty means the
    /// category is not considered the default and `1` means it is
    /// considered the default one; the crate represents this as a
    /// `bool`.
    ///
    /// When multiple rider categories are eligible for a single
    /// fare product specified by a `fare_product_id`, there must be
    /// exactly one of these eligible rider categories indicated as
    /// the default rider category
    /// (`is_default_fare_category = 1`).
    pub is_default_fare_category: bool,
    /// URL of a web page, usually from the operating agency, that
    /// provides detailed information about a specific rider
    /// category and/or describes its eligibility criteria.
    /// Optional; `None` means the value is empty in the file.
    pub eligibility_url: Option<String>,
}

impl RiderCategory {
    /// Creates a non-default rider category.
    ///
    /// # Arguments
    ///
    /// * `rider_category_id` - Unique rider category identifier
    /// * `rider_category_name` - Rider-facing name
    pub fn new(rider_category_id: &str, rider_category_name: &str) -> Self {
        RiderCategory {
            rider_category_id: rider_category_id.to_string(),
            rider_category_name: rider_category_name.to_string(),
            is_default_fare_category: false,
            eligibility_url: None,
        }
    }

    /// Marks the category as the default fare category.
    pub fn as_default(mut self) -> Self {
        self.is_default_fare_category = true;
        self
    }
}

gtfs_enum! {
    /// Type of fare media (`fare_media_type`).
    FareMediaType {
        /// None. Used when there is no fare media involved in
        /// purchasing or validating a fare product, such as paying
        /// cash to a driver or conductor with no physical ticket
        /// provided (`0`)
        None = 0,
        /// Physical paper ticket that allows a passenger to take
        /// either a certain number of pre-purchased trips or
        /// unlimited trips within a fixed period of time (`1`)
        PaperTicket = 1,
        /// Physical transit card that has stored tickets, passes or
        /// monetary value (`2`)
        TransitCard = 2,
        /// cEMV (contactless Europay, Mastercard and Visa) as an
        /// open-loop token container for account-based ticketing
        /// (`3`)
        ContactlessEmv = 3,
        /// Mobile app that has stored virtual transit cards,
        /// tickets, passes, or monetary value (`4`)
        MobileApp = 4,
    }
}

/// A fare medium from `fare_media.txt` (GTFS-Fares v2).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareMedia {
    /// Identifies a fare media. Unique ID. Required.
    pub fare_media_id: String,
    /// Name of the fare media. Optional; `None` means the value is
    /// empty in the file.
    ///
    /// For fare media which are transit cards
    /// (`fare_media_type = 2`) or mobile apps
    /// (`fare_media_type = 4`), the `fare_media_name` should be
    /// included and should match the rider-facing name used by the
    /// organizations delivering them.
    pub fare_media_name: Option<String>,
    /// The type of fare media. Required. See [`FareMediaType`] for
    /// the valid options.
    pub fare_media_type: FareMediaType,
}

impl FareMedia {
    /// Creates a fare medium.
    ///
    /// # Arguments
    ///
    /// * `fare_media_id` - Unique fare media identifier
    /// * `fare_media_type` - Type of the fare media
    pub fn new(fare_media_id: &str, fare_media_type: FareMediaType) -> Self {
        FareMedia {
            fare_media_id: fare_media_id.to_string(),
            fare_media_name: None,
            fare_media_type,
        }
    }

    /// Sets the rider-facing name.
    pub fn with_name(mut self, fare_media_name: &str) -> Self {
        self.fare_media_name = Some(fare_media_name.to_string());
        self
    }
}

/// A purchasable fare product from `fare_products.txt`
/// (GTFS-Fares v2).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareProduct {
    /// Identifies a fare product or set of fare products.
    /// Required.
    ///
    /// Multiple records sharing the same `fare_product_id` are
    /// permitted as long as they contain different
    /// `fare_media_id`s or `rider_category_id`s. Differing
    /// `fare_media_id`s would indicate various methods are
    /// available for employing the fare product, potentially at
    /// different prices. Differing `rider_category_id`s would
    /// indicate multiple rider categories are eligible for the fare
    /// product, potentially at different prices.
    pub fare_product_id: String,
    /// The name of the fare product as displayed to riders.
    /// Optional; `None` means the value is empty in the file.
    pub fare_product_name: Option<String>,
    /// Identifies a rider category eligible for the fare product.
    /// Foreign ID referencing
    /// `rider_categories.rider_category_id`. Optional.
    ///
    /// If `rider_category_id` is empty (`None`), the fare product
    /// is eligible for any `rider_category_id`.
    ///
    /// When multiple rider categories are eligible for a single
    /// fare product specified by a `fare_product_id`, there must be
    /// only one of these rider categories indicated as the default
    /// rider category (`is_default_fare_category = 1`).
    pub rider_category_id: Option<String>,
    /// Identifies a fare media that can be employed to use the fare
    /// product during the trip. Foreign ID referencing
    /// `fare_media.fare_media_id`. Optional. When `fare_media_id`
    /// is empty (`None`), it is considered that the fare media is
    /// unknown.
    pub fare_media_id: Option<String>,
    /// The cost of the fare product, in the units of `currency`.
    /// Required. May be negative to represent transfer discounts.
    /// May be zero to represent a fare product that is free. In the
    /// file the currency amount must contain the number of decimal
    /// places specified by the norm ISO 4217 for the accompanying
    /// currency code; the crate stores it exactly as a
    /// [`CurrencyAmount`] decimal, as the spec mandates for
    /// financial values.
    pub amount: CurrencyAmount,
    /// The currency of the cost of the fare product, as an ISO 4217
    /// currency code. Required.
    pub currency: String,
}

impl FareProduct {
    /// Creates a fare product.
    ///
    /// # Arguments
    ///
    /// * `fare_product_id` - Fare product identifier
    /// * `amount` - Cost of the product
    /// * `currency` - ISO 4217 currency code
    pub fn new(fare_product_id: &str, amount: CurrencyAmount, currency: &str) -> Self {
        FareProduct {
            fare_product_id: fare_product_id.to_string(),
            fare_product_name: None,
            rider_category_id: None,
            fare_media_id: None,
            amount,
            currency: currency.to_string(),
        }
    }

    /// Sets the rider-facing name.
    pub fn with_name(mut self, fare_product_name: &str) -> Self {
        self.fare_product_name = Some(fare_product_name.to_string());
        self
    }

    /// Restricts the product to a rider category.
    pub fn with_rider_category(mut self, rider_category_id: &str) -> Self {
        self.rider_category_id = Some(rider_category_id.to_string());
        self
    }

    /// Restricts the product to a fare media.
    pub fn with_fare_media(mut self, fare_media_id: &str) -> Self {
        self.fare_media_id = Some(fare_media_id.to_string());
        self
    }
}

/// A fare rule for a single leg of travel from
/// `fare_leg_rules.txt` (GTFS-Fares v2).
///
/// Legs are matched against rules by network, origin/destination
/// area and timeframes; the matched rule prices the leg with its
/// fare product.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareLegRule {
    /// Identifies a group of entries in `fare_leg_rules.txt`.
    /// Optional; `None` means the value is empty in the file.
    ///
    /// Used to describe fare transfer rules between
    /// `fare_transfer_rules.from_leg_group_id` and
    /// `fare_transfer_rules.to_leg_group_id`.
    ///
    /// Multiple entries in `fare_leg_rules.txt` may belong to the
    /// same `leg_group_id`. The same entry in `fare_leg_rules.txt`
    /// (not including `leg_group_id`) must not belong to multiple
    /// `leg_group_id`s.
    pub leg_group_id: Option<String>,
    /// Identifies a route network that applies for the fare leg
    /// rule. Foreign ID referencing `routes.network_id` or
    /// `networks.network_id`. Optional.
    ///
    /// If the `rule_priority` field does not exist AND there are no
    /// matching `fare_leg_rules.network_id` values to the
    /// `network_id` being filtered, empty `network_id` will be
    /// matched by default. An empty entry in `network_id` then
    /// corresponds to all networks defined in `routes.txt` or
    /// `networks.txt` excluding the ones listed under
    /// `fare_leg_rules.network_id`.
    ///
    /// If the `rule_priority` field exists in the file, an empty
    /// `network_id` indicates that the route network of the leg
    /// does not affect the matching of this rule.
    ///
    /// When matching against an effective fare leg of multiple legs
    /// (`fare_leg_join_rules.txt`), each leg must have the same
    /// `network_id`, which will be used for matching.
    pub network_id: Option<String>,
    /// Identifies a departure area. Foreign ID referencing
    /// `areas.area_id`. Optional.
    ///
    /// If the `rule_priority` field does not exist AND there are no
    /// matching `fare_leg_rules.from_area_id` values to the
    /// `area_id` being filtered, empty `from_area_id` will be
    /// matched by default. An empty entry in `from_area_id` then
    /// corresponds to all areas defined in `areas.area_id`
    /// excluding the ones listed under
    /// `fare_leg_rules.from_area_id`.
    ///
    /// If the `rule_priority` field exists in the file, an empty
    /// `from_area_id` indicates that the departure area of the leg
    /// does not affect the matching of this rule.
    ///
    /// When matching against an effective fare leg of multiple legs
    /// (`fare_leg_join_rules.txt`), the first leg of the effective
    /// fare leg is used for determining the departure area.
    pub from_area_id: Option<String>,
    /// Identifies an arrival area. Foreign ID referencing
    /// `areas.area_id`. Optional.
    ///
    /// If the `rule_priority` field does not exist AND there are no
    /// matching `fare_leg_rules.to_area_id` values to the `area_id`
    /// being filtered, empty `to_area_id` will be matched by
    /// default. An empty entry in `to_area_id` then corresponds to
    /// all areas defined in `areas.area_id` excluding the ones
    /// listed under `fare_leg_rules.to_area_id`.
    ///
    /// If the `rule_priority` field exists in the file, an empty
    /// `to_area_id` indicates that the arrival area of the leg does
    /// not affect the matching of this rule.
    ///
    /// When matching against an effective fare leg of multiple legs
    /// (`fare_leg_join_rules.txt`), the last leg of the effective
    /// fare leg is used for determining the arrival area.
    pub to_area_id: Option<String>,
    /// Defines the timeframe for the fare validation event at the
    /// start of the fare leg. Foreign ID referencing
    /// `timeframes.timeframe_group_id`. Optional.
    ///
    /// The "start time" of the fare leg is the time at which the
    /// event is scheduled to occur. For example, the time could be
    /// the scheduled departure time of a bus at the start of a fare
    /// leg where the rider boards and validates their fare. For the
    /// rule matching semantics below, the start time is computed in
    /// local time, as determined by the local time semantics of
    /// `timeframes.txt`. The stop or station of the fare leg's
    /// departure event should be used for timezone resolution,
    /// where appropriate.
    ///
    /// For a fare leg rule that specifies a
    /// `from_timeframe_group_id`, that rule will match a particular
    /// leg if there exists at least one record in `timeframes.txt`
    /// where all of the following conditions are true:
    /// - the value of `timeframe_group_id` is equal to the
    ///   `from_timeframe_group_id` value;
    /// - the set of days identified by the record's `service_id`
    ///   contains the "current day" of the fare leg's start time;
    /// - the "time-of-day" of the fare leg's start time is greater
    ///   than or equal to the record's `timeframes.start_time`
    ///   value and less than the `timeframes.end_time` value.
    ///
    /// An empty `from_timeframe_group_id` indicates that the start
    /// time of the leg does not affect the matching of this rule.
    ///
    /// When matching against an effective fare leg of multiple legs
    /// (`fare_leg_join_rules.txt`), the first leg of the effective
    /// fare leg is used for determining the starting fare
    /// validation event.
    pub from_timeframe_group_id: Option<String>,
    /// Defines the timeframe for the fare validation event at the
    /// end of the fare leg. Foreign ID referencing
    /// `timeframes.timeframe_group_id`. Optional.
    ///
    /// The "end time" of the fare leg is the time at which the
    /// event is scheduled to occur. For example, the time could be
    /// the scheduled arrival time of a bus at the end of a fare leg
    /// where the rider gets off and validates their fare. For the
    /// rule matching semantics below, the end time is computed in
    /// local time, as determined by the local time semantics of
    /// `timeframes.txt`. The stop or station of the fare leg's
    /// arrival event should be used for timezone resolution, where
    /// appropriate.
    ///
    /// For a fare leg rule that specifies a
    /// `to_timeframe_group_id`, that rule will match a particular
    /// leg if there exists at least one record in `timeframes.txt`
    /// where all of the following conditions are true:
    /// - the value of `timeframe_group_id` is equal to the
    ///   `to_timeframe_group_id` value;
    /// - the set of days identified by the record's `service_id`
    ///   contains the "current day" of the fare leg's end time;
    /// - the "time-of-day" of the fare leg's end time is greater
    ///   than or equal to the record's `timeframes.start_time`
    ///   value and less than the `timeframes.end_time` value.
    ///
    /// An empty `to_timeframe_group_id` indicates that the end time
    /// of the leg does not affect the matching of this rule.
    ///
    /// When matching against an effective fare leg of multiple legs
    /// (`fare_leg_join_rules.txt`), the last leg of the effective
    /// fare leg is used for determining the ending fare validation
    /// event.
    pub to_timeframe_group_id: Option<String>,
    /// The fare product required to travel the leg. Foreign ID
    /// referencing `fare_products.fare_product_id`. Required.
    pub fare_product_id: String,
    /// Defines the order of priority in which matching rules are
    /// applied to legs, allowing certain rules to take precedence
    /// over others. When multiple entries in `fare_leg_rules.txt`
    /// match, the rule or set of rules with the highest value for
    /// `rule_priority` will be selected. Non-negative integer.
    /// Optional.
    ///
    /// An empty value for `rule_priority` is treated as zero.
    pub rule_priority: Option<u32>,
}

impl FareLegRule {
    /// Creates a leg rule matching any leg.
    ///
    /// # Arguments
    ///
    /// * `fare_product_id` - Fare product that prices the leg
    pub fn new(fare_product_id: &str) -> Self {
        FareLegRule {
            leg_group_id: None,
            network_id: None,
            from_area_id: None,
            to_area_id: None,
            from_timeframe_group_id: None,
            to_timeframe_group_id: None,
            fare_product_id: fare_product_id.to_string(),
            rule_priority: None,
        }
    }

    /// Sets the leg group identifier.
    pub fn with_leg_group(mut self, leg_group_id: &str) -> Self {
        self.leg_group_id = Some(leg_group_id.to_string());
        self
    }

    /// Restricts the rule to a route network.
    pub fn with_network(mut self, network_id: &str) -> Self {
        self.network_id = Some(network_id.to_string());
        self
    }

    /// Restricts the rule to an origin-destination area pair.
    ///
    /// # Arguments
    ///
    /// * `from_area_id` - Area the leg must start in
    /// * `to_area_id` - Area the leg must end in
    pub fn with_areas(mut self, from_area_id: &str, to_area_id: &str) -> Self {
        self.from_area_id = Some(from_area_id.to_string());
        self.to_area_id = Some(to_area_id.to_string());
        self
    }
}

/// A rule for treating two consecutive legs as one from
/// `fare_leg_join_rules.txt` (GTFS-Fares v2).
///
/// A transfer between the matched networks (and optionally stops) is
/// considered a continuation of the same effective fare leg.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareLegJoinRule {
    /// Matches a pre-transfer leg that uses the specified route
    /// network. If specified, the same `to_network_id` must also be
    /// specified. Foreign ID referencing `routes.network_id` or
    /// `networks.network_id`. Required.
    pub from_network_id: String,
    /// Matches a post-transfer leg that uses the specified route
    /// network. If specified, the same `from_network_id` must also
    /// be specified. Foreign ID referencing `routes.network_id` or
    /// `networks.network_id`. Required.
    pub to_network_id: String,
    /// Matches a pre-transfer leg that ends at the specified stop
    /// (`location_type = 0` or empty) or station
    /// (`location_type = 1`). Foreign ID referencing
    /// `stops.stop_id`. `None` means the value is empty in the
    /// file.
    ///
    /// Conditionally Required:
    /// - Required if `to_stop_id` is defined.
    /// - Optional otherwise.
    pub from_stop_id: Option<String>,
    /// Matches a post-transfer leg that starts at the specified
    /// stop (`location_type = 0` or empty) or station
    /// (`location_type = 1`). Foreign ID referencing
    /// `stops.stop_id`. `None` means the value is empty in the
    /// file.
    ///
    /// Conditionally Required:
    /// - Required if `from_stop_id` is defined.
    /// - Optional otherwise.
    pub to_stop_id: Option<String>,
}

impl FareLegJoinRule {
    /// Creates a join rule between two networks.
    ///
    /// # Arguments
    ///
    /// * `from_network_id` - Network of the pre-transfer leg
    /// * `to_network_id` - Network of the post-transfer leg
    pub fn new(from_network_id: &str, to_network_id: &str) -> Self {
        FareLegJoinRule {
            from_network_id: from_network_id.to_string(),
            to_network_id: to_network_id.to_string(),
            from_stop_id: None,
            to_stop_id: None,
        }
    }

    /// Restricts the rule to a stop pair.
    ///
    /// # Arguments
    ///
    /// * `from_stop_id` - Stop the pre-transfer leg must end at
    /// * `to_stop_id` - Stop the post-transfer leg must start at
    pub fn between_stops(mut self, from_stop_id: &str, to_stop_id: &str) -> Self {
        self.from_stop_id = Some(from_stop_id.to_string());
        self.to_stop_id = Some(to_stop_id.to_string());
        self
    }
}

gtfs_enum! {
    /// How a transfer duration limit is measured
    /// (`duration_limit_type`).
    DurationLimitType {
        /// Between the departure fare validation of the first leg
        /// in the transfer sub-journey and the arrival fare
        /// validation of the last leg in the transfer sub-journey
        /// (`0`)
        DepartureToArrival = 0,
        /// Between the departure fare validation of the first leg
        /// in the transfer sub-journey and the departure fare
        /// validation of the last leg in the transfer sub-journey
        /// (`1`)
        DepartureToDeparture = 1,
        /// Between the arrival fare validation of the first leg in
        /// the transfer sub-journey and the departure fare
        /// validation of the last leg in the transfer sub-journey
        /// (`2`)
        ArrivalToDeparture = 2,
        /// Between the arrival fare validation of the first leg in
        /// the transfer sub-journey and the arrival fare validation
        /// of the last leg in the transfer sub-journey (`3`)
        ArrivalToArrival = 3,
    }
}

gtfs_enum! {
    /// How the cost of a transfer is computed
    /// (`fare_transfer_type`).
    FareTransferType {
        /// From-leg `fare_leg_rules.fare_product_id` plus
        /// `fare_transfer_rules.fare_product_id`; A + AB (`0`)
        FromLegPlusTransfer = 0,
        /// From-leg `fare_leg_rules.fare_product_id` plus
        /// `fare_transfer_rules.fare_product_id` plus to-leg
        /// `fare_leg_rules.fare_product_id`; A + AB + B (`1`)
        FromLegPlusTransferPlusToLeg = 1,
        /// `fare_transfer_rules.fare_product_id` only; AB (`2`)
        TransferOnly = 2,
    }
}

/// A fare transfer rule from `fare_transfer_rules.txt`
/// (GTFS-Fares v2).
///
/// Prices transfers between the legs matched by the referenced leg
/// groups.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FareTransferRule {
    /// Identifies a group of pre-transfer fare leg rules. Foreign
    /// ID referencing `fare_leg_rules.leg_group_id`. Optional.
    ///
    /// If there are no matching `from_leg_group_id` values to the
    /// `leg_group_id` being filtered, empty `from_leg_group_id`
    /// will be matched by default. An empty entry in
    /// `from_leg_group_id` corresponds to all leg groups defined
    /// under `fare_leg_rules.leg_group_id` excluding the ones
    /// listed under `fare_transfer_rules.from_leg_group_id`.
    pub from_leg_group_id: Option<String>,
    /// Identifies a group of post-transfer fare leg rules. Foreign
    /// ID referencing `fare_leg_rules.leg_group_id`. Optional.
    ///
    /// If there are no matching `to_leg_group_id` values to the
    /// `leg_group_id` being filtered, empty `to_leg_group_id` will
    /// be matched by default. An empty entry in `to_leg_group_id`
    /// corresponds to all leg groups defined under
    /// `fare_leg_rules.leg_group_id` excluding the ones listed
    /// under `fare_transfer_rules.to_leg_group_id`.
    pub to_leg_group_id: Option<String>,
    /// Defines how many consecutive transfers the transfer rule may
    /// be applied to. Non-zero integer. Valid options are `-1` for
    /// no limit, or `1` or more to define how many transfers the
    /// transfer rule may span.
    ///
    /// If a sub-journey matches multiple records with different
    /// `transfer_count`s, then the rule with the minimum
    /// `transfer_count` that is greater than or equal to the
    /// current transfer count of the sub-journey is to be selected.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden if `from_leg_group_id` does not equal
    ///   `to_leg_group_id`.
    /// - Required if `from_leg_group_id` equals `to_leg_group_id`.
    pub transfer_count: Option<i32>,
    /// Defines the duration limit of the transfer. Must be
    /// expressed in integer increments of seconds (positive
    /// integer). Optional. If there is no duration limit,
    /// `duration_limit` must be empty (`None`).
    pub duration_limit: Option<u32>,
    /// Defines the relative start and end of `duration_limit`. See
    /// [`DurationLimitType`] for the valid options.
    ///
    /// When a transfer rule with the same `from_leg_group_id` and
    /// `to_leg_group_id` is matched multiple times consecutively
    /// within a multi-leg journey, the `duration_limit` specified
    /// by the rule should be measured starting from the first
    /// matched leg.
    ///
    /// Conditionally Required:
    /// - Required if `duration_limit` is defined.
    /// - Forbidden if `duration_limit` is empty.
    pub duration_limit_type: Option<DurationLimitType>,
    /// Indicates the cost processing method of transferring between
    /// legs in a journey. Required. See [`FareTransferType`] for
    /// the valid options. When multiple transfers occur in a
    /// journey, the total processed cost of the preceding leg(s)
    /// and transfer(s) is carried into the processing of each
    /// subsequent transfer.
    pub fare_transfer_type: FareTransferType,
    /// The fare product required to transfer between two fare legs.
    /// Foreign ID referencing `fare_products.fare_product_id`.
    /// Optional. If empty (`None`), the cost of the transfer rule
    /// is 0.
    pub fare_product_id: Option<String>,
}

impl FareTransferRule {
    /// Creates a transfer rule matching any leg pair.
    ///
    /// # Arguments
    ///
    /// * `fare_transfer_type` - How the cost of the transfer is
    ///   computed
    pub fn new(fare_transfer_type: FareTransferType) -> Self {
        FareTransferRule {
            from_leg_group_id: None,
            to_leg_group_id: None,
            transfer_count: None,
            duration_limit: None,
            duration_limit_type: None,
            fare_transfer_type,
            fare_product_id: None,
        }
    }

    /// Restricts the rule to a pair of leg groups.
    ///
    /// # Arguments
    ///
    /// * `from_leg_group_id` - Leg group of the pre-transfer leg
    /// * `to_leg_group_id` - Leg group of the post-transfer leg
    pub fn between_leg_groups(mut self, from_leg_group_id: &str, to_leg_group_id: &str) -> Self {
        self.from_leg_group_id = Some(from_leg_group_id.to_string());
        self.to_leg_group_id = Some(to_leg_group_id.to_string());
        self
    }

    /// Sets the duration limit of the transfer.
    ///
    /// # Arguments
    ///
    /// * `duration_limit` - Limit in seconds
    /// * `duration_limit_type` - How the limit is measured
    pub fn with_duration_limit(
        mut self,
        duration_limit: u32,
        duration_limit_type: DurationLimitType,
    ) -> Self {
        self.duration_limit = Some(duration_limit);
        self.duration_limit_type = Some(duration_limit_type);
        self
    }

    /// Sets the fare product that prices the transfer.
    pub fn with_fare_product(mut self, fare_product_id: &str) -> Self {
        self.fare_product_id = Some(fare_product_id.to_string());
        self
    }
}
