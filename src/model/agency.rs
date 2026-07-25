//! `agency.txt` - transit agencies with service represented in the
//! dataset.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#agencytxt>

gtfs_enum! {
    /// Support for contactless EMV bank card payments
    /// (`cemv_support`).
    #[derive(Default)]
    CemvSupport {
        /// No cEMV information for trips associated with this
        /// agency (`0`, default)
        #[default]
        NoInformation = 0,
        /// Riders may use cEMVs (contactless EMV cards or mobile
        /// devices) as fare media at a fare validator for trips
        /// associated with this agency (`1`)
        Supported = 1,
        /// cEMVs are not supported as fare media for trips
        /// associated with this agency (`2`)
        NotSupported = 2,
    }
}

/// A transit agency from `agency.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::Agency;
///
/// let agency = Agency::new("City Transit", "https://transit.example", "Europe/Moscow")
///     .with_id("CT")
///     .with_lang("ru");
/// assert_eq!(agency.agency_id.as_deref(), Some("CT"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Agency {
    /// Identifies a transit brand which is often synonymous with a
    /// transit agency. Note that in some cases, such as when a
    /// single agency operates multiple separate services, agencies
    /// and brands are distinct. The spec uses the term "agency" in
    /// place of "brand". A dataset may contain data from multiple
    /// agencies.
    ///
    /// Conditionally Required:
    /// - Required when the dataset contains data for multiple
    ///   transit agencies.
    /// - Recommended otherwise.
    ///
    /// `None` when the field is empty in the file.
    pub agency_id: Option<String>,
    /// Full name of the transit agency.
    ///
    /// Required.
    pub agency_name: String,
    /// URL of the transit agency.
    ///
    /// Required.
    pub agency_url: String,
    /// Timezone where the transit agency is located, as an IANA
    /// timezone name (e.g. "Europe/Moscow"). If multiple agencies
    /// are specified in the dataset, each must have the same
    /// `agency_timezone`.
    ///
    /// Required.
    pub agency_timezone: String,
    /// Primary language used by this transit agency, as an IETF
    /// BCP 47 language code. Should be provided to help GTFS
    /// consumers choose capitalization rules and other
    /// language-specific settings for the dataset.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub agency_lang: Option<String>,
    /// A voice telephone number for the specified agency. This
    /// field is a string value that presents the telephone number
    /// as typical for the agency's service area. It may contain
    /// punctuation marks to group the digits of the number.
    /// Dialable text (for example, TriMet's "503-238-RIDE") is
    /// permitted, but the field must not contain any other
    /// descriptive text.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub agency_phone: Option<String>,
    /// URL of a web page where a rider can purchase tickets or
    /// other fare instruments for that agency, or a web page
    /// containing information about that agency's fares.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub agency_fare_url: Option<String>,
    /// Email address actively monitored by the agency's customer
    /// service department. This email address should be a direct
    /// contact point where transit riders can reach a customer
    /// service representative at the agency.
    ///
    /// Optional; `None` when the field is empty in the file.
    pub agency_email: Option<String>,
    /// Indicates if riders can access a transit service (i.e.,
    /// trip) associated with this agency by using a contactless
    /// EMV card or mobile device as fare media at a fare validator
    /// (such as in pay-as-you-go or open-loop systems).
    ///
    /// Optional; represented by [`CemvSupport`], defaulting to
    /// [`CemvSupport::NoInformation`] (`0`) via `Default` when the
    /// field is empty in the file. See the enum variants for the
    /// full per-value spec descriptions.
    ///
    /// If both `agency.cemv_support` and `routes.cemv_support` are
    /// provided for the same service, the value in
    /// `routes.cemv_support` takes precedence. If conflicting
    /// information exists between this field and fare-related
    /// files (such as `fare_media.txt`, `fare_products.txt` or
    /// `fare_leg_rules.txt`), the information in those files takes
    /// precedence.
    pub cemv_support: CemvSupport,
}

impl Agency {
    /// Creates an agency from the required fields.
    ///
    /// # Arguments
    ///
    /// * `agency_name` - Full name of the agency
    /// * `agency_url` - URL of the agency website
    /// * `agency_timezone` - IANA timezone of the agency
    pub fn new(agency_name: &str, agency_url: &str, agency_timezone: &str) -> Self {
        Agency {
            agency_id: None,
            agency_name: agency_name.to_string(),
            agency_url: agency_url.to_string(),
            agency_timezone: agency_timezone.to_string(),
            agency_lang: None,
            agency_phone: None,
            agency_fare_url: None,
            agency_email: None,
            cemv_support: CemvSupport::default(),
        }
    }

    /// Sets the agency identifier.
    pub fn with_id(mut self, agency_id: &str) -> Self {
        self.agency_id = Some(agency_id.to_string());
        self
    }

    /// Sets the primary language.
    pub fn with_lang(mut self, agency_lang: &str) -> Self {
        self.agency_lang = Some(agency_lang.to_string());
        self
    }

    /// Sets the voice telephone number.
    pub fn with_phone(mut self, agency_phone: &str) -> Self {
        self.agency_phone = Some(agency_phone.to_string());
        self
    }
}
