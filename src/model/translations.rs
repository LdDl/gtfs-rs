//! `translations.txt` - translated values of rider-facing fields.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#translationstxt>

/// Table a translation applies to (`table_name`).
///
/// Defines the table that contains the field to be translated.
/// Required in `translations.txt`. Any file added to GTFS will have
/// a `table_name` value equivalent to the file name, as listed
/// below (i.e., not including the `.txt` file extension).
///
/// Unlike most GTFS enums this one is encoded as a string, so it
/// converts with [`TableName::from_name`] / [`TableName::name`]
/// instead of numeric codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableName {
    /// `agency.txt` (`agency`). Records are selected by
    /// `agency_id` in `record_id`; no `record_sub_id` is used
    Agency,
    /// `stops.txt` (`stops`). Records are selected by `stop_id` in
    /// `record_id`; no `record_sub_id` is used
    Stops,
    /// `routes.txt` (`routes`). Records are selected by `route_id`
    /// in `record_id`; no `record_sub_id` is used
    Routes,
    /// `trips.txt` (`trips`). Records are selected by `trip_id` in
    /// `record_id`; no `record_sub_id` is used
    Trips,
    /// `stop_times.txt` (`stop_times`). Records are selected by
    /// `trip_id` in `record_id` plus `stop_sequence` in
    /// `record_sub_id`, since the table has no unique single-field
    /// key
    StopTimes,
    /// `pathways.txt` (`pathways`). Records are selected by
    /// `pathway_id` in `record_id`; no `record_sub_id` is used
    Pathways,
    /// `levels.txt` (`levels`). Records are selected by `level_id`
    /// in `record_id`; no `record_sub_id` is used
    Levels,
    /// `feed_info.txt` (`feed_info`). The file contains a single
    /// record, so `record_id`, `record_sub_id` and `field_value`
    /// are all forbidden for this table
    FeedInfo,
    /// `attributions.txt` (`attributions`). Records are selected by
    /// `attribution_id` in `record_id`; no `record_sub_id` is used
    Attributions,
}

impl TableName {
    /// Parses the table name used in GTFS files (e.g. "stop_times").
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::TableName;
    ///
    /// let table = TableName::from_name("stop_times");
    /// assert_eq!(table, Some(TableName::StopTimes));
    /// assert_eq!(TableName::from_name("unknown"), None);
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "agency" => Some(TableName::Agency),
            "stops" => Some(TableName::Stops),
            "routes" => Some(TableName::Routes),
            "trips" => Some(TableName::Trips),
            "stop_times" => Some(TableName::StopTimes),
            "pathways" => Some(TableName::Pathways),
            "levels" => Some(TableName::Levels),
            "feed_info" => Some(TableName::FeedInfo),
            "attributions" => Some(TableName::Attributions),
            _ => None,
        }
    }

    /// Returns the table name used in GTFS files.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::TableName;
    ///
    /// assert_eq!(TableName::StopTimes.name(), "stop_times");
    /// assert_eq!(TableName::FeedInfo.name(), "feed_info");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            TableName::Agency => "agency",
            TableName::Stops => "stops",
            TableName::Routes => "routes",
            TableName::Trips => "trips",
            TableName::StopTimes => "stop_times",
            TableName::Pathways => "pathways",
            TableName::Levels => "levels",
            TableName::FeedInfo => "feed_info",
            TableName::Attributions => "attributions",
        }
    }
}

/// A translation from `translations.txt`.
///
/// The translated record is selected either by `record_id` (plus
/// `record_sub_id` for tables without a single unique key, such as
/// stop times) or by matching `field_value` against the original
/// value. If both referencing methods (`record_id`,
/// `record_sub_id`) and `field_value` are used to translate the
/// same value in 2 different rows, the translation provided with
/// (`record_id`, `record_sub_id`) takes precedence.
///
/// The optional file's primary key is (`table_name`, `field_name`,
/// `language`, `record_id`, `record_sub_id`, `field_value`).
///
/// # Examples
///
/// ```
/// use gtfs_rs::{TableName, Translation};
///
/// let t = Translation::new(TableName::Stops, "stop_name", "en", "Central Station")
///     .for_record("S1");
/// assert_eq!(t.table_name.name(), "stops");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Translation {
    /// Defines the table that contains the field to be translated;
    /// see [`TableName`] for the allowed values. Any file added to
    /// GTFS will have a `table_name` value equivalent to the file
    /// name, not including the `.txt` file extension. Required.
    pub table_name: TableName,
    /// Name of the field to be translated. Fields with type `Text`
    /// may be translated, fields with type `URL`, `Email` and
    /// `Phone number` may also be "translated" to provide resources
    /// in the correct language. Fields with other types should not
    /// be translated. Required.
    pub field_name: String,
    /// Language of translation (IETF BCP 47 language code).
    /// Required.
    ///
    /// If the language is the same as in `feed_info.feed_lang`, the
    /// original value of the field will be assumed to be the
    /// default value to use in languages without specific
    /// translations (if `default_lang` doesn't specify otherwise).
    ///
    /// Example: In Switzerland, a city in an officially bilingual
    /// canton is officially called "Biel/Bienne", but would simply
    /// be called "Bienne" in French and "Biel" in German.
    pub language: String,
    /// Translated value (a text, URL, email or phone number).
    /// Required.
    pub translation: String,
    /// Defines the record that corresponds to the field to be
    /// translated. The value in `record_id` must be the first or
    /// only field of a table's primary key, as defined in the
    /// primary key attribute for each table and below:
    ///
    /// - `agency_id` for `agency.txt`;
    /// - `stop_id` for `stops.txt`;
    /// - `route_id` for `routes.txt`;
    /// - `trip_id` for `trips.txt`;
    /// - `trip_id` for `stop_times.txt`;
    /// - `pathway_id` for `pathways.txt`;
    /// - `level_id` for `levels.txt`;
    /// - `attribution_id` for `attributions.txt`.
    ///
    /// Fields in tables not defined above should not be
    /// translated. However producers sometimes add extra fields
    /// that are outside the official specification and these
    /// unofficial fields may be translated. Below is the
    /// recommended way to use `record_id` for those tables:
    ///
    /// - `service_id` for `calendar.txt`;
    /// - `service_id` for `calendar_dates.txt`;
    /// - `fare_id` for `fare_attributes.txt`;
    /// - `fare_id` for `fare_rules.txt`;
    /// - `shape_id` for `shapes.txt`;
    /// - `trip_id` for `frequencies.txt`;
    /// - `from_stop_id` for `transfers.txt`.
    ///
    /// Conditionally Required:
    /// - Forbidden if `table_name` is `feed_info`.
    /// - Forbidden if `field_value` is defined.
    /// - Required if `field_value` is empty.
    ///
    /// `None` when the value is empty in the file.
    pub record_id: Option<String>,
    /// Helps the record that contains the field to be translated
    /// when the table doesn't have a unique ID. Therefore, the
    /// value in `record_sub_id` is the secondary ID of the table,
    /// as defined by the table below:
    ///
    /// - None for `agency.txt`;
    /// - None for `stops.txt`;
    /// - None for `routes.txt`;
    /// - None for `trips.txt`;
    /// - `stop_sequence` for `stop_times.txt`;
    /// - None for `pathways.txt`;
    /// - None for `levels.txt`;
    /// - None for `attributions.txt`.
    ///
    /// Fields in tables not defined above should not be
    /// translated. However producers sometimes add extra fields
    /// that are outside the official specification and these
    /// unofficial fields may be translated. Below is the
    /// recommended way to use `record_sub_id` for those tables:
    ///
    /// - None for `calendar.txt`;
    /// - `date` for `calendar_dates.txt`;
    /// - None for `fare_attributes.txt`;
    /// - `route_id` for `fare_rules.txt`;
    /// - None for `shapes.txt`;
    /// - `start_time` for `frequencies.txt`;
    /// - `to_stop_id` for `transfers.txt`.
    ///
    /// Conditionally Required:
    /// - Forbidden if `table_name` is `feed_info`.
    /// - Forbidden if `field_value` is defined.
    /// - Required if `table_name=stop_times` and `record_id` is
    ///   defined.
    ///
    /// `None` when the value is empty in the file.
    pub record_sub_id: Option<String>,
    /// Instead of defining which record should be translated by
    /// using `record_id` and `record_sub_id`, this field can be
    /// used to define the value which should be translated. When
    /// used, the translation will be applied when the fields
    /// identified by `table_name` and `field_name` contains the
    /// exact same value defined in `field_value`.
    ///
    /// The field must have exactly the value defined in
    /// `field_value`. If only a subset of the value matches
    /// `field_value`, the translation won't be applied.
    ///
    /// If two translation rules match the same record (one with
    /// `field_value`, and the other one with `record_id`), the
    /// rule with `record_id` takes precedence.
    ///
    /// Conditionally Required:
    /// - Forbidden if `table_name` is `feed_info`.
    /// - Forbidden if `record_id` is defined.
    /// - Required if `record_id` is empty.
    ///
    /// `None` when the value is empty in the file.
    pub field_value: Option<String>,
}

impl Translation {
    /// Creates a translation.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Table the translated field belongs to
    /// * `field_name` - Name of the translated field
    /// * `language` - Language of the translation
    /// * `translation` - Translated value
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{TableName, Translation};
    ///
    /// let t =
    ///     Translation::new(TableName::Stops, "stop_name", "en", "Central");
    /// assert_eq!(t.language, "en");
    /// assert!(t.record_id.is_none());
    /// ```
    pub fn new(table_name: TableName, field_name: &str, language: &str, translation: &str) -> Self {
        Translation {
            table_name,
            field_name: field_name.to_string(),
            language: language.to_string(),
            translation: translation.to_string(),
            record_id: None,
            record_sub_id: None,
            field_value: None,
        }
    }

    /// Selects the translated record by its key.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{TableName, Translation};
    ///
    /// let t =
    ///     Translation::new(TableName::Stops, "stop_name", "en", "Central")
    ///         .for_record("S1");
    /// assert_eq!(t.record_id.as_deref(), Some("S1"));
    /// ```
    pub fn for_record(mut self, record_id: &str) -> Self {
        self.record_id = Some(record_id.to_string());
        self
    }

    /// Selects translated records by matching the original value.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{TableName, Translation};
    ///
    /// let t =
    ///     Translation::new(TableName::Stops, "stop_name", "en", "Central")
    ///         .for_field_value("Центральная");
    /// assert_eq!(t.field_value.as_deref(), Some("Центральная"));
    /// ```
    pub fn for_field_value(mut self, field_value: &str) -> Self {
        self.field_value = Some(field_value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_name_roundtrip() {
        for table in [
            TableName::Agency,
            TableName::Stops,
            TableName::Routes,
            TableName::Trips,
            TableName::StopTimes,
            TableName::Pathways,
            TableName::Levels,
            TableName::FeedInfo,
            TableName::Attributions,
        ] {
            assert_eq!(TableName::from_name(table.name()), Some(table));
        }
        assert_eq!(TableName::from_name("fare_rules"), None);
    }
}
