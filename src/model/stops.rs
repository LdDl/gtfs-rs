//! `stops.txt` - stops, stations, entrances, generic nodes and
//! boarding areas.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#stopstxt>

gtfs_enum! {
    /// Type of location described by a `stops.txt` record
    /// (`location_type`).
    #[derive(Default)]
    LocationType {
        /// Stop (or Platform). A location where passengers board
        /// or disembark from a transit vehicle. Is called a
        /// platform when defined within a `parent_station` (`0`)
        #[default]
        StopOrPlatform = 0,
        /// Station. A physical structure or area that contains one
        /// or more platform (`1`)
        Station = 1,
        /// Entrance/Exit. A location where passengers can enter or
        /// exit a station from the street. If an entrance/exit
        /// belongs to multiple stations, it may be linked by
        /// pathways to both, but the data provider must pick one
        /// of them as parent (`2`)
        EntranceExit = 2,
        /// Generic Node. A location within a station, not matching
        /// any other `location_type`, that may be used to link
        /// together pathways defined in `pathways.txt` (`3`)
        GenericNode = 3,
        /// Boarding Area. A specific location on a platform, where
        /// passengers can board and/or alight vehicles (`4`)
        BoardingArea = 4,
    }
}

gtfs_enum! {
    /// Wheelchair accessibility of a location
    /// (`wheelchair_boarding`).
    #[derive(Default)]
    WheelchairBoarding {
        /// For parentless stops: no accessibility information for
        /// the stop. For child stops: the stop will inherit its
        /// `wheelchair_boarding` behavior from the parent station,
        /// if specified in the parent. For station entrances/exits:
        /// the station entrance will inherit its
        /// `wheelchair_boarding` behavior from the parent station,
        /// if specified for the parent (`0`)
        #[default]
        NoInformation = 0,
        /// For parentless stops: some vehicles at this stop can be
        /// boarded by a rider in a wheelchair. For child stops:
        /// there exists some accessible path from outside the
        /// station to the specific stop/platform. For station
        /// entrances/exits: the station entrance is wheelchair
        /// accessible (`1`)
        Accessible = 1,
        /// For parentless stops: wheelchair boarding is not
        /// possible at this stop. For child stops: there exists no
        /// accessible path from outside the station to the
        /// specific stop/platform. For station entrances/exits: no
        /// accessible path from station entrance to
        /// stops/platforms (`2`)
        NotAccessible = 2,
    }
}

gtfs_enum! {
    /// How a stop with a parent station is accessed from the street
    /// (`stop_access`).
    StopAccess {
        /// The stop/platform cannot be directly accessed from the
        /// street network. It must be accessed from a station
        /// entrance if there is one defined for the station,
        /// otherwise the station itself. If there are pathways
        /// defined for the station, they must be used to access
        /// the stop/platform (`0`)
        ViaStation = 0,
        /// Consuming applications should generate directions for
        /// access directly to the stop, independent of any
        /// entrances or pathways of the parent station (`1`)
        DirectStreetAccess = 1,
    }
}

/// A location from `stops.txt`.
///
/// # Examples
///
/// A station with one platform:
///
/// ```
/// use gtfs_rs::{LocationType, Stop};
///
/// let station = Stop::new("S1")
///     .with_name("Central")
///     .with_coordinates(55.751, 37.617)
///     .with_location_type(LocationType::Station);
/// let platform = Stop::new("S1_2")
///     .with_name("Central")
///     .with_coordinates(55.7511, 37.6172)
///     .with_parent_station("S1")
///     .with_platform_code("2");
///
/// assert_eq!(platform.location_type, LocationType::StopOrPlatform);
/// assert_eq!(platform.parent_station.as_deref(), Some(station.stop_id.as_str()));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Stop {
    /// Identifies a location: stop/platform, station,
    /// entrance/exit, generic node or boarding area (see
    /// `location_type`). ID must be unique across all
    /// `stops.stop_id`, locations.geojson `id`, and
    /// `location_groups.location_group_id` values.
    ///
    /// Multiple routes may use the same `stop_id`.
    ///
    /// Required.
    pub stop_id: String,
    /// Short text or a number that identifies the location for
    /// riders. These codes are often used in phone-based transit
    /// information systems or printed on signage to make it easier
    /// for riders to get information for a particular location.
    /// The `stop_code` may be the same as `stop_id` if it is
    /// public facing. This field should be left empty for
    /// locations without a code presented to riders.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub stop_code: Option<String>,
    /// Name of the location. The `stop_name` should match the
    /// agency's rider-facing name for the location as printed on a
    /// timetable, published online, or represented on signage. For
    /// translations into other languages, use `translations.txt`.
    ///
    /// When the location is a boarding area (`location_type=4`),
    /// the `stop_name` should contain the name of the boarding
    /// area as displayed by the agency. It could be just one
    /// letter (like on some European intercity railway stations),
    /// or text like "Wheelchair boarding area" (NYC's Subway) or
    /// "Head of short trains" (Paris' RER).
    ///
    /// Conditionally Required:
    /// - Required for locations which are stops
    ///   (`location_type=0`), stations (`location_type=1`) or
    ///   entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes
    ///   (`location_type=3`) or boarding areas
    ///   (`location_type=4`).
    ///
    /// `None` when the field is empty in the file.
    pub stop_name: Option<String>,
    /// Readable version of the `stop_name`. See "Text-to-speech
    /// field" in the spec's Term Definitions for more.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub tts_stop_name: Option<String>,
    /// Description of the location that provides useful, quality
    /// information. Should not be a duplicate of `stop_name`.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub stop_desc: Option<String>,
    /// Latitude of the location.
    ///
    /// For stops/platforms (`location_type=0`) and boarding areas
    /// (`location_type=4`), the coordinates must be the ones of
    /// the bus pole - if exists - and otherwise of where the
    /// travelers are boarding the vehicle (on the sidewalk or the
    /// platform, and not on the roadway or the track where the
    /// vehicle stops).
    ///
    /// Conditionally Required:
    /// - Required for locations which are stops
    ///   (`location_type=0`), stations (`location_type=1`) or
    ///   entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes
    ///   (`location_type=3`) or boarding areas
    ///   (`location_type=4`).
    ///
    /// WGS84 decimal degrees; `None` when the field is empty in
    /// the file.
    pub stop_lat: Option<f64>,
    /// Longitude of the location.
    ///
    /// For stops/platforms (`location_type=0`) and boarding areas
    /// (`location_type=4`), the coordinates must be the ones of
    /// the bus pole - if exists - and otherwise of where the
    /// travelers are boarding the vehicle (on the sidewalk or the
    /// platform, and not on the roadway or the track where the
    /// vehicle stops).
    ///
    /// Conditionally Required:
    /// - Required for locations which are stops
    ///   (`location_type=0`), stations (`location_type=1`) or
    ///   entrances/exits (`location_type=2`).
    /// - Optional for locations which are generic nodes
    ///   (`location_type=3`) or boarding areas
    ///   (`location_type=4`).
    ///
    /// WGS84 decimal degrees; `None` when the field is empty in
    /// the file.
    pub stop_lon: Option<f64>,
    /// Identifies the fare zone for a stop. If this record
    /// represents a station or station entrance, the `zone_id` is
    /// ignored.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub zone_id: Option<String>,
    /// URL of a web page about the location. This should be
    /// different from the `agency.agency_url` and the
    /// `routes.route_url` field values.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub stop_url: Option<String>,
    /// Type of the location, represented by [`LocationType`].
    ///
    /// Optional; defaults to [`LocationType::StopOrPlatform`]
    /// (`0`) via `Default` when the field is empty in the file.
    /// See the enum variants for the full per-value spec
    /// descriptions.
    pub location_type: LocationType,
    /// Defines hierarchy between the different locations defined
    /// in `stops.txt`. It contains the ID of the parent location,
    /// as followed:
    /// - Stop/platform (`location_type=0`): the `parent_station`
    ///   field contains the ID of a station.
    /// - Station (`location_type=1`): this field must be empty.
    /// - Entrance/exit (`location_type=2`) or generic node
    ///   (`location_type=3`): the `parent_station` field contains
    ///   the ID of a station (`location_type=1`).
    /// - Boarding area (`location_type=4`): the `parent_station`
    ///   field contains the ID of a platform.
    ///
    /// Foreign ID referencing `stops.stop_id`.
    ///
    /// Conditionally Required:
    /// - Required for locations which are entrances
    ///   (`location_type=2`), generic nodes (`location_type=3`)
    ///   or boarding areas (`location_type=4`).
    /// - Optional for stops/platforms (`location_type=0`).
    /// - Forbidden for stations (`location_type=1`).
    ///
    /// `None` when the field is empty in the file.
    pub parent_station: Option<String>,
    /// Indicates how the stop is accessed for a particular
    /// station, represented by [`StopAccess`]. When `stop_access`
    /// is empty (`None`), the access for the specified stop or
    /// platform is considered undefined. See the enum variants for
    /// the full per-value spec descriptions.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden for locations which are stations
    ///   (`location_type=1`), entrances (`location_type=2`),
    ///   generic nodes (`location_type=3`) or boarding areas
    ///   (`location_type=4`).
    /// - Forbidden if `parent_station` is empty.
    /// - Optional otherwise.
    pub stop_access: Option<StopAccess>,
    /// Timezone of the location, as an IANA timezone name. If the
    /// location has a parent station, it inherits the parent
    /// station's timezone instead of applying its own. Stations
    /// and parentless stops with empty `stop_timezone` inherit the
    /// timezone specified by `agency.agency_timezone`.
    ///
    /// The times provided in `stop_times.txt` are in the timezone
    /// specified by `agency.agency_timezone`, not `stop_timezone`.
    /// This ensures that the time values in a trip always increase
    /// over the course of a trip, regardless of which timezones
    /// the trip crosses.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub stop_timezone: Option<String>,
    /// Indicates whether wheelchair boardings are possible from
    /// the location, represented by [`WheelchairBoarding`]. The
    /// meaning of each value differs for parentless stops, child
    /// stops and station entrances/exits; see the enum variants
    /// for the full per-value spec descriptions.
    ///
    /// Optional; defaults to
    /// [`WheelchairBoarding::NoInformation`] (`0`) via `Default`
    /// when the field is empty in the file.
    pub wheelchair_boarding: WheelchairBoarding,
    /// Level of the location. The same level may be used by
    /// multiple unlinked stations.
    ///
    /// Foreign ID referencing `levels.level_id`.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub level_id: Option<String>,
    /// Platform identifier for a platform stop (a stop belonging
    /// to a station). This should be just the platform identifier
    /// (e.g. "G" or "3"). Words like "platform" or "track" (or the
    /// feed's language-specific equivalent) should not be
    /// included. This allows feed consumers to more easily
    /// internationalize and localize the platform identifier into
    /// other languages.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub platform_code: Option<String>,
}

impl Stop {
    /// Creates a stop with just an identifier.
    ///
    /// # Arguments
    ///
    /// * `stop_id` - Unique stop identifier
    pub fn new(stop_id: &str) -> Self {
        Stop {
            stop_id: stop_id.to_string(),
            stop_code: None,
            stop_name: None,
            tts_stop_name: None,
            stop_desc: None,
            stop_lat: None,
            stop_lon: None,
            zone_id: None,
            stop_url: None,
            location_type: LocationType::default(),
            parent_station: None,
            stop_access: None,
            stop_timezone: None,
            wheelchair_boarding: WheelchairBoarding::default(),
            level_id: None,
            platform_code: None,
        }
    }

    /// Sets the display name.
    pub fn with_name(mut self, stop_name: &str) -> Self {
        self.stop_name = Some(stop_name.to_string());
        self
    }

    /// Sets WGS84 coordinates.
    ///
    /// # Arguments
    ///
    /// * `lat` - WGS84 latitude
    /// * `lon` - WGS84 longitude
    pub fn with_coordinates(mut self, lat: f64, lon: f64) -> Self {
        self.stop_lat = Some(lat);
        self.stop_lon = Some(lon);
        self
    }

    /// Sets the location type.
    pub fn with_location_type(mut self, location_type: LocationType) -> Self {
        self.location_type = location_type;
        self
    }

    /// Sets the parent station.
    pub fn with_parent_station(mut self, parent_station: &str) -> Self {
        self.parent_station = Some(parent_station.to_string());
        self
    }

    /// Sets the fare zone identifier.
    pub fn with_zone_id(mut self, zone_id: &str) -> Self {
        self.zone_id = Some(zone_id.to_string());
        self
    }

    /// Sets the platform code.
    pub fn with_platform_code(mut self, platform_code: &str) -> Self {
        self.platform_code = Some(platform_code.to_string());
        self
    }
}
