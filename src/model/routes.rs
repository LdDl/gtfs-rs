//! `routes.txt` - transit routes: groups of trips displayed to riders
//! as a single service.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#routestxt>

use super::agency::CemvSupport;

/// Indicates the type of transportation used on a route
/// (`route_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteType {
    /// Tram, streetcar, light rail. Any light rail or street level
    /// system within a metropolitan area (`0`)
    Tram,
    /// Subway, metro. Any underground rail system within a
    /// metropolitan area (`1`)
    Subway,
    /// Rail. Used for intercity or long-distance travel (`2`)
    Rail,
    /// Bus. Used for short- and long-distance bus routes (`3`)
    Bus,
    /// Ferry. Used for short- and long-distance boat service (`4`)
    Ferry,
    /// Cable tram. Used for street-level rail cars where the cable
    /// runs beneath the vehicle (e.g., cable car in San Francisco)
    /// (`5`)
    CableTram,
    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial
    /// tramway). Cable transport where cabins, cars, gondolas or
    /// open chairs are suspended by means of one or more cables
    /// (`6`)
    AerialLift,
    /// Funicular. Any rail system designed for steep inclines (`7`)
    Funicular,
    /// Trolleybus. Electric buses that draw power from overhead
    /// wires using poles (`11`)
    Trolleybus,
    /// Monorail. Railway in which the track consists of a single
    /// rail or a beam (`12`)
    Monorail,
    /// Extended route type (Google extension, e.g. `700` for generic
    /// bus services). Also covers codes not named by the base spec.
    Extended(u16),
}

impl RouteType {
    /// Parses the numeric code used in GTFS files. Codes outside the
    /// base set but within `0..=u16::MAX` map to
    /// [`RouteType::Extended`]; negative codes are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::RouteType;
    ///
    /// assert_eq!(RouteType::from_code(3), Some(RouteType::Bus));
    /// assert_eq!(RouteType::from_code(800), Some(RouteType::Extended(800)));
    /// assert_eq!(RouteType::from_code(-1), None);
    /// ```
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(RouteType::Tram),
            1 => Some(RouteType::Subway),
            2 => Some(RouteType::Rail),
            3 => Some(RouteType::Bus),
            4 => Some(RouteType::Ferry),
            5 => Some(RouteType::CableTram),
            6 => Some(RouteType::AerialLift),
            7 => Some(RouteType::Funicular),
            11 => Some(RouteType::Trolleybus),
            12 => Some(RouteType::Monorail),
            c if (0..=i32::from(u16::MAX)).contains(&c) => Some(RouteType::Extended(c as u16)),
            _ => None,
        }
    }

    /// Returns the numeric code used in GTFS files.
    pub fn code(self) -> i32 {
        match self {
            RouteType::Tram => 0,
            RouteType::Subway => 1,
            RouteType::Rail => 2,
            RouteType::Bus => 3,
            RouteType::Ferry => 4,
            RouteType::CableTram => 5,
            RouteType::AerialLift => 6,
            RouteType::Funicular => 7,
            RouteType::Trolleybus => 11,
            RouteType::Monorail => 12,
            RouteType::Extended(c) => i32::from(c),
        }
    }
}

gtfs_enum! {
    /// Continuous stopping pickup or drop-off behavior along a route
    /// or trip segment (`continuous_pickup`, `continuous_drop_off`).
    ///
    /// Indicates that the rider can board (pickup) or alight
    /// (drop-off) from the transit vehicle at any point along the
    /// vehicle's travel path as described by `shapes.txt`.
    #[derive(Default)]
    ContinuousPickupDropOff {
        /// Continuous stopping pickup or drop-off: the rider can
        /// board or alight from the transit vehicle at any point
        /// along the vehicle's travel path as described by
        /// `shapes.txt` (`0`)
        Continuous = 0,
        /// No continuous stopping pickup or drop-off (`1` or empty,
        /// default)
        #[default]
        NotAvailable = 1,
        /// Must phone agency to arrange continuous stopping pickup
        /// or drop-off (`2`)
        PhoneAgency = 2,
        /// Must coordinate with driver to arrange continuous
        /// stopping pickup or drop-off (`3`)
        CoordinateWithDriver = 3,
    }
}

/// A route from `routes.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Route, RouteType};
///
/// let route = Route::new("L1", RouteType::Tram)
///     .with_short_name("1")
///     .with_colors("FFD700", "000000");
/// assert_eq!(route.route_color.as_deref(), Some("FFD700"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Route {
    /// Identifies a route.
    ///
    /// Required. Unique ID.
    pub route_id: String,
    /// Agency for the specified route.
    ///
    /// Foreign ID referencing `agency.agency_id`. Conditionally
    /// Required: required if multiple agencies are defined in
    /// `agency.txt`; recommended otherwise. `None` means the value
    /// is empty in the file.
    pub agency_id: Option<String>,
    /// Short name of a route. Often a short, abstract identifier
    /// (e.g., "32", "100X", "Green") that riders use to identify a
    /// route. Both `route_short_name` and `route_long_name` may be
    /// defined.
    ///
    /// Conditionally Required: required if `routes.route_long_name`
    /// is empty; recommended if there is a brief service
    /// designation. `None` means the value is empty in the file.
    pub route_short_name: Option<String>,
    /// Full name of a route. This name is generally more
    /// descriptive than the `route_short_name` and often includes
    /// the route's destination or stop. Both `route_short_name` and
    /// `route_long_name` may be defined.
    ///
    /// Conditionally Required: required if
    /// `routes.route_short_name` is empty; optional otherwise.
    /// `None` means the value is empty in the file.
    pub route_long_name: Option<String>,
    /// Description of a route that provides useful, quality
    /// information. Should not be a duplicate of `route_short_name`
    /// or `route_long_name`.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub route_desc: Option<String>,
    /// Indicates the type of transportation used on a route.
    ///
    /// Required. Represented by [`RouteType`]; see its variants for
    /// the full list of values. [`RouteType::Extended`] covers
    /// extended Google route types.
    pub route_type: RouteType,
    /// URL of a web page about the particular route. Should be
    /// different from the `agency.agency_url` value.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub route_url: Option<String>,
    /// Route color designation that matches public facing material.
    /// A six-digit hexadecimal value without the leading `#`
    /// (e.g. "FFD700"). Defaults to white (`FFFFFF`) when omitted
    /// or left empty. The color difference between `route_color`
    /// and `route_text_color` should provide sufficient contrast
    /// when viewed on a black and white screen.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub route_color: Option<String>,
    /// Legible color to use for text drawn against a background of
    /// `route_color`. A six-digit hexadecimal value without the
    /// leading `#` (e.g. "000000"). Defaults to black (`000000`)
    /// when omitted or left empty. The color difference between
    /// `route_color` and `route_text_color` should provide
    /// sufficient contrast when viewed on a black and white screen.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub route_text_color: Option<String>,
    /// Orders the routes in a way which is ideal for presentation
    /// to customers. Routes with smaller `route_sort_order` values
    /// should be displayed first.
    ///
    /// Optional. Non-negative integer. `None` means the value is
    /// empty in the file.
    pub route_sort_order: Option<u32>,
    /// Indicates that the rider can board the transit vehicle at
    /// any point along the vehicle's travel path as described by
    /// `shapes.txt`, on every trip of the route. Values for
    /// `routes.continuous_pickup` may be overridden by defining
    /// values in `stop_times.continuous_pickup` for specific stop
    /// times along the route.
    ///
    /// Conditionally Forbidden: any value other than
    /// [`ContinuousPickupDropOff::NotAvailable`] (`1` or empty) is
    /// forbidden if `stop_times.start_pickup_drop_off_window` or
    /// `stop_times.end_pickup_drop_off_window` are defined for any
    /// trip of this route; optional otherwise. Represented by
    /// [`ContinuousPickupDropOff`]; the default (`1` or empty, no
    /// continuous stopping pickup) is encoded via `Default`.
    pub continuous_pickup: ContinuousPickupDropOff,
    /// Indicates that the rider can alight from the transit vehicle
    /// at any point along the vehicle's travel path as described by
    /// `shapes.txt`, on every trip of the route. Values for
    /// `routes.continuous_drop_off` may be overridden by defining
    /// values in `stop_times.continuous_drop_off` for specific stop
    /// times along the route.
    ///
    /// Conditionally Forbidden: any value other than
    /// [`ContinuousPickupDropOff::NotAvailable`] (`1` or empty) is
    /// forbidden if `stop_times.start_pickup_drop_off_window` or
    /// `stop_times.end_pickup_drop_off_window` are defined for any
    /// trip of this route; optional otherwise. Represented by
    /// [`ContinuousPickupDropOff`]; the default (`1` or empty, no
    /// continuous stopping drop off) is encoded via `Default`.
    pub continuous_drop_off: ContinuousPickupDropOff,
    /// Identifies a group of routes for fare matching
    /// (GTFS-Fares v2). Multiple rows in `routes.txt` may have the
    /// same `network_id`.
    ///
    /// Conditionally Forbidden: forbidden if the
    /// `route_networks.txt` or `networks.txt` file exists; optional
    /// otherwise. `None` means the value is empty in the file.
    pub network_id: Option<String>,
    /// Indicates if riders can access a transit service (i.e.,
    /// trip) associated with this route by using a contactless EMV
    /// (Europay, Mastercard, and Visa) card or mobile device as
    /// fare media at a fare validator (such as in pay-as-you-go or
    /// open-loop systems). This field does not indicate that cEMV
    /// can be used to purchase other fare products or to add value
    /// to another fare media. Support for cEMVs should only be
    /// indicated if all services under this route are accessible
    /// with the use of cEMV cards or mobile devices as fare media.
    ///
    /// If both `agency.cemv_support` and `routes.cemv_support` are
    /// provided for the same service, the value in
    /// `routes.cemv_support` shall take precedence. This field is
    /// independent of all other fare-related files and may be used
    /// separately. If there is conflicting information between this
    /// field and any fare-related file (such as `fare_media.txt`,
    /// `fare_products.txt`, or `fare_leg_rules.txt`), the
    /// information in those files shall take precedence.
    ///
    /// Optional. Represented by [`CemvSupport`]; the default (`0`
    /// or empty, no cEMV information) is encoded via `Default`.
    pub cemv_support: CemvSupport,
}

impl Route {
    /// Creates a route from the required fields.
    ///
    /// # Arguments
    ///
    /// * `route_id` - Unique route identifier
    /// * `route_type` - Type of transportation used on the route
    pub fn new(route_id: &str, route_type: RouteType) -> Self {
        Route {
            route_id: route_id.to_string(),
            agency_id: None,
            route_short_name: None,
            route_long_name: None,
            route_desc: None,
            route_type,
            route_url: None,
            route_color: None,
            route_text_color: None,
            route_sort_order: None,
            continuous_pickup: ContinuousPickupDropOff::default(),
            continuous_drop_off: ContinuousPickupDropOff::default(),
            network_id: None,
            cemv_support: CemvSupport::default(),
        }
    }

    /// Sets the short display name.
    pub fn with_short_name(mut self, route_short_name: &str) -> Self {
        self.route_short_name = Some(route_short_name.to_string());
        self
    }

    /// Sets the full display name.
    pub fn with_long_name(mut self, route_long_name: &str) -> Self {
        self.route_long_name = Some(route_long_name.to_string());
        self
    }

    /// Sets the operating agency.
    pub fn with_agency_id(mut self, agency_id: &str) -> Self {
        self.agency_id = Some(agency_id.to_string());
        self
    }

    /// Sets the route color and matching text color (six-digit hex
    /// values without `#`).
    ///
    /// # Arguments
    ///
    /// * `route_color` - Background color (e.g. "FFD700")
    /// * `route_text_color` - Legible text color (e.g. "000000")
    pub fn with_colors(mut self, route_color: &str, route_text_color: &str) -> Self {
        self.route_color = Some(route_color.to_string());
        self.route_text_color = Some(route_text_color.to_string());
        self
    }
}
