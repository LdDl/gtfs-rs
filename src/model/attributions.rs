//! `attributions.txt` - attributions applied to the dataset.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#attributionstxt>

/// An attribution from `attributions.txt`.
///
/// The optional file defines the attributions applied to the
/// dataset. An attribution applies to the whole dataset, or to a
/// single agency, route or trip when the corresponding identifier
/// is set (at most one of them may be defined per record).
///
/// # Examples
///
/// ```
/// use gtfs_rs::Attribution;
///
/// let a = Attribution::new("City Transit").as_producer().as_operator();
/// assert!(a.is_producer && a.is_operator && !a.is_authority);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Attribution {
    /// Identifies an attribution for the dataset or a subset of
    /// it. This is mostly useful for translations. Optional.
    pub attribution_id: Option<String>,
    /// Agency to which the attribution applies. Foreign ID
    /// referencing `agency.agency_id`.
    ///
    /// If one `agency_id`, `route_id`, or `trip_id` attribution is
    /// defined, the other ones must be empty. If none of them is
    /// specified, the attribution will apply to the whole dataset.
    ///
    /// Optional; `None` when the value is empty in the file.
    pub agency_id: Option<String>,
    /// Functions in the same way as `agency_id` except the
    /// attribution applies to a route. Multiple attributions may
    /// apply to the same route. Foreign ID referencing
    /// `routes.route_id`.
    ///
    /// Optional; `None` when the value is empty in the file.
    pub route_id: Option<String>,
    /// Functions in the same way as `agency_id` except the
    /// attribution applies to a trip. Multiple attributions may
    /// apply to the same trip. Foreign ID referencing
    /// `trips.trip_id`.
    ///
    /// Optional; `None` when the value is empty in the file.
    pub trip_id: Option<String>,
    /// Name of the organization that the dataset is attributed to.
    /// Required.
    pub organization_name: String,
    /// The role of the organization is producer. In the GTFS file
    /// this is an enum: `0` or empty - organization doesn't have
    /// this role; `1` - organization does have this role. Modeled
    /// as a `bool` in this crate. Optional.
    ///
    /// At least one of the fields `is_producer`, `is_operator`, or
    /// `is_authority` should be set at `1` (`true`).
    pub is_producer: bool,
    /// Functions in the same way as `is_producer` except the role
    /// of the organization is operator. Modeled as a `bool` in
    /// this crate. Optional.
    pub is_operator: bool,
    /// Functions in the same way as `is_producer` except the role
    /// of the organization is authority. Modeled as a `bool` in
    /// this crate. Optional.
    pub is_authority: bool,
    /// URL of the organization. Optional.
    pub attribution_url: Option<String>,
    /// Email of the organization. Optional.
    pub attribution_email: Option<String>,
    /// Phone number of the organization. Optional.
    pub attribution_phone: Option<String>,
}

impl Attribution {
    /// Creates a dataset-wide attribution with no roles set. At
    /// least one of the producer, operator or authority roles must
    /// be activated for a valid record.
    ///
    /// # Arguments
    ///
    /// * `organization_name` - Organization the dataset is
    ///   attributed to
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Attribution;
    ///
    /// let a = Attribution::new("Org");
    /// assert_eq!(a.organization_name, "Org");
    /// assert!(!a.is_producer && !a.is_operator && !a.is_authority);
    /// ```
    pub fn new(organization_name: &str) -> Self {
        Attribution {
            attribution_id: None,
            agency_id: None,
            route_id: None,
            trip_id: None,
            organization_name: organization_name.to_string(),
            is_producer: false,
            is_operator: false,
            is_authority: false,
            attribution_url: None,
            attribution_email: None,
            attribution_phone: None,
        }
    }

    /// Marks the organization as the producer of the dataset.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Attribution;
    ///
    /// let a = Attribution::new("Org").as_producer();
    /// assert!(a.is_producer);
    /// assert!(!a.is_operator);
    /// ```
    pub fn as_producer(mut self) -> Self {
        self.is_producer = true;
        self
    }

    /// Marks the organization as the operator of the service.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Attribution;
    ///
    /// let a = Attribution::new("Org").as_operator();
    /// assert!(a.is_operator);
    /// assert!(!a.is_producer);
    /// ```
    pub fn as_operator(mut self) -> Self {
        self.is_operator = true;
        self
    }

    /// Marks the organization as the authority over the service.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Attribution;
    ///
    /// let a = Attribution::new("Org").as_authority();
    /// assert!(a.is_authority);
    /// assert!(!a.is_producer);
    /// ```
    pub fn as_authority(mut self) -> Self {
        self.is_authority = true;
        self
    }
}
