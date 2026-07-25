//! `networks.txt` and `route_networks.txt` - route networks matched
//! by the fare leg rules (GTFS-Fares v2).
//!
//! Both files are forbidden when `routes.network_id` is used.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#networkstxt>

/// A route network from `networks.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Network, RouteNetwork};
///
/// let metro = Network::new("metro").with_name("Metro");
/// let line = RouteNetwork::new("metro", "L1");
/// assert_eq!(line.network_id, metro.network_id);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Network {
    /// Identifies a network. Must be unique in `networks.txt`.
    /// Unique ID. Required.
    pub network_id: String,
    /// The name of the network that applies for fare leg rules, as
    /// used by the local agency and its riders. Optional; `None`
    /// means the value is empty in the file.
    pub network_name: Option<String>,
}

impl Network {
    /// Creates a network.
    ///
    /// # Arguments
    ///
    /// * `network_id` - Unique network identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Network;
    ///
    /// let metro = Network::new("metro");
    /// assert_eq!(metro.network_id, "metro");
    /// assert_eq!(metro.network_name, None);
    /// ```
    pub fn new(network_id: &str) -> Self {
        Network {
            network_id: network_id.to_string(),
            network_name: None,
        }
    }

    /// Sets the rider-facing name.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Network;
    ///
    /// let metro = Network::new("metro").with_name("Metro");
    /// assert_eq!(metro.network_name.as_deref(), Some("Metro"));
    /// ```
    pub fn with_name(mut self, network_name: &str) -> Self {
        self.network_name = Some(network_name.to_string());
        self
    }
}

/// A route-to-network assignment from `route_networks.txt`.
///
/// A route may belong to at most one network.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RouteNetwork {
    /// Identifies a network to which one or multiple `route_id`s
    /// belong. A `route_id` can only be defined in one
    /// `network_id`. Foreign ID referencing
    /// `networks.network_id`. Required.
    pub network_id: String,
    /// Identifies a route. Foreign ID referencing
    /// `routes.route_id`. Required.
    pub route_id: String,
}

impl RouteNetwork {
    /// Creates a route-to-network assignment.
    ///
    /// # Arguments
    ///
    /// * `network_id` - Network the route belongs to
    /// * `route_id` - Assigned route
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::RouteNetwork;
    ///
    /// let line = RouteNetwork::new("metro", "L1");
    /// assert_eq!(line.network_id, "metro");
    /// assert_eq!(line.route_id, "L1");
    /// ```
    pub fn new(network_id: &str, route_id: &str) -> Self {
        RouteNetwork {
            network_id: network_id.to_string(),
            route_id: route_id.to_string(),
        }
    }
}
