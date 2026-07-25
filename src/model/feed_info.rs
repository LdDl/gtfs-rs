//! `feed_info.txt` - metadata about the dataset itself.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#feed_infotxt>

use crate::misc::GtfsDate;

/// Dataset metadata from `feed_info.txt`.
///
/// The file contains information about the dataset itself, rather
/// than the services that the dataset describes. In some cases, the
/// publisher of the dataset is a different entity than any of the
/// agencies.
///
/// The file contains a single record and is conditionally required:
/// required if `translations.txt` is provided, recommended
/// otherwise.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::{FeedInfo, GtfsDate};
///
///     let info = FeedInfo::new("City Transit", "https://transit.example", "ru")
///         .with_period(GtfsDate::new(2026, 1, 1)?, GtfsDate::new(2026, 12, 31)?)
///         .with_version("2026-07");
///     assert_eq!(info.feed_version.as_deref(), Some("2026-07"));
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FeedInfo {
    /// Full name of the organization that publishes the dataset.
    /// This may be the same as one of the `agency.agency_name`
    /// values. Required.
    pub feed_publisher_name: String,
    /// URL of the dataset publishing organization's website. This
    /// may be the same as one of the `agency.agency_url` values.
    /// Required.
    pub feed_publisher_url: String,
    /// Default language used for the text in this dataset (an IETF
    /// BCP 47 language code). This setting helps GTFS consumers
    /// choose capitalization rules and other language-specific
    /// settings for the dataset. The file `translations.txt` can be
    /// used if the text needs to be translated into languages other
    /// than the default one. Required.
    ///
    /// The default language may be multilingual for datasets with
    /// the original text in multiple languages. In such cases, the
    /// `feed_lang` field should contain the language code `mul`
    /// defined by the norm ISO 639-2, and a translation for each
    /// language used in the dataset should be provided in
    /// `translations.txt`. If all the original text in the dataset
    /// is in the same language, then `mul` should not be used.
    ///
    /// Example: Consider a dataset from a multilingual country like
    /// Switzerland, with the original `stops.stop_name` field
    /// populated with stop names in different languages. Each stop
    /// name is written according to the dominant language in that
    /// stop's geographic location, e.g. `Genève` for the
    /// French-speaking city of Geneva, `Zürich` for the
    /// German-speaking city of Zurich, and `Biel/Bienne` for the
    /// bilingual city of Biel/Bienne. The dataset `feed_lang`
    /// should be `mul` and translations would be provided in
    /// `translations.txt`, in German: `Genf`, `Zürich` and `Biel`;
    /// in French: `Genève`, `Zurich` and `Bienne`; in Italian:
    /// `Ginevra`, `Zurigo` and `Bienna`; and in English: `Geneva`,
    /// `Zurich` and `Biel/Bienne`.
    pub feed_lang: String,
    /// Defines the language that should be used when the data
    /// consumer doesn't know the language of the rider. It will
    /// often be `en` (English).
    ///
    /// Optional; `None` when the value is empty in the file.
    pub default_lang: Option<String>,
    /// The dataset provides complete and reliable schedule
    /// information for service in the period from the beginning of
    /// the `feed_start_date` day to the end of the `feed_end_date`
    /// day. Both days may be left empty if unavailable. The
    /// `feed_end_date` date must not precede the `feed_start_date`
    /// date if both are given. It is recommended that dataset
    /// providers give schedule data outside this period to advise
    /// of likely future service, but dataset consumers should treat
    /// it mindful of its non-authoritative status. If
    /// `feed_start_date` or `feed_end_date` extend beyond the
    /// active calendar dates defined in `calendar.txt` and
    /// `calendar_dates.txt`, the dataset is making an explicit
    /// assertion that there is no service for dates within the
    /// `feed_start_date` or `feed_end_date` range but not included
    /// in the active calendar dates.
    ///
    /// Recommended. Stored as a [`GtfsDate`]; `None` when the value
    /// is empty in the file.
    pub feed_start_date: Option<GtfsDate>,
    /// End of the period for which the dataset provides complete
    /// and reliable schedule information; see `feed_start_date` for
    /// the full dataset validity semantics.
    ///
    /// Recommended. Stored as a [`GtfsDate`]; `None` when the value
    /// is empty in the file.
    pub feed_end_date: Option<GtfsDate>,
    /// String that indicates the current version of their GTFS
    /// dataset. GTFS-consuming applications can display this value
    /// to help dataset publishers determine whether the latest
    /// dataset has been incorporated. Recommended.
    pub feed_version: Option<String>,
    /// Email address for communication regarding the GTFS dataset
    /// and data publishing practices. `feed_contact_email` is a
    /// technical contact for GTFS-consuming applications. Provide
    /// customer service contact information through `agency.txt`.
    /// It's recommended that at least one of `feed_contact_email`
    /// or `feed_contact_url` are provided. Optional.
    pub feed_contact_email: Option<String>,
    /// URL for contact information, a web-form, support desk, or
    /// other tools for communication regarding the GTFS dataset and
    /// data publishing practices. `feed_contact_url` is a technical
    /// contact for GTFS-consuming applications. Provide customer
    /// service contact information through `agency.txt`. It's
    /// recommended that at least one of `feed_contact_url` or
    /// `feed_contact_email` are provided. Optional.
    pub feed_contact_url: Option<String>,
}

impl FeedInfo {
    /// Creates feed metadata from the required fields.
    ///
    /// # Arguments
    ///
    /// * `feed_publisher_name` - Organization publishing the dataset
    /// * `feed_publisher_url` - URL of the publisher's website
    /// * `feed_lang` - Default language of the dataset
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::FeedInfo;
    ///
    /// let info = FeedInfo::new("Pub", "https://x.example", "ru");
    /// assert_eq!(info.feed_lang, "ru");
    /// assert!(info.feed_version.is_none());
    /// ```
    pub fn new(feed_publisher_name: &str, feed_publisher_url: &str, feed_lang: &str) -> Self {
        FeedInfo {
            feed_publisher_name: feed_publisher_name.to_string(),
            feed_publisher_url: feed_publisher_url.to_string(),
            feed_lang: feed_lang.to_string(),
            default_lang: None,
            feed_start_date: None,
            feed_end_date: None,
            feed_version: None,
            feed_contact_email: None,
            feed_contact_url: None,
        }
    }

    /// Sets the period the dataset is complete and reliable for.
    ///
    /// # Arguments
    ///
    /// * `start` - First covered date
    /// * `end` - Last covered date
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::{FeedInfo, GtfsDate};
    ///
    ///     let start = GtfsDate::new(2026, 1, 1)?;
    ///     let end = GtfsDate::new(2026, 12, 31)?;
    ///     let info = FeedInfo::new("Pub", "https://x.example", "ru")
    ///         .with_period(start, end);
    ///     assert!(info.feed_start_date.is_some());
    ///     assert!(info.feed_end_date.is_some());
    ///     Ok(())
    /// }
    /// ```
    pub fn with_period(mut self, start: GtfsDate, end: GtfsDate) -> Self {
        self.feed_start_date = Some(start);
        self.feed_end_date = Some(end);
        self
    }

    /// Sets the dataset version string.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::FeedInfo;
    ///
    /// let info = FeedInfo::new("Pub", "https://x.example", "ru")
    ///     .with_version("2026-07");
    /// assert_eq!(info.feed_version.as_deref(), Some("2026-07"));
    /// ```
    pub fn with_version(mut self, feed_version: &str) -> Self {
        self.feed_version = Some(feed_version.to_string());
        self
    }
}
