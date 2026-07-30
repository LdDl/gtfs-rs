//! `networks.txt` and `route_networks.txt` writers.

use crate::model::{Network, RouteNetwork};
use crate::writers::csv::CsvWrite;

impl CsvWrite for Network {
    const FILE_NAME: &'static str = "networks.txt";

    const HEADER: &'static [&'static str] = &["network_id", "network_name"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.network_id.clone(),
            self.network_name.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for RouteNetwork {
    const FILE_NAME: &'static str = "route_networks.txt";

    const HEADER: &'static [&'static str] = &["network_id", "route_id"];

    fn fields(&self) -> Vec<String> {
        vec![self.network_id.clone(), self.route_id.clone()]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Network, RouteNetwork};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_network_fields() {
        let metro = Network::new("metro").with_name("Metro");
        let fields = metro.fields();
        assert_eq!(fields.len(), Network::HEADER.len());
        assert_eq!(fields[0], "metro");
        assert_eq!(fields[1], "Metro");

        let unnamed = Network::new("bus");
        assert_eq!(unnamed.fields()[1], "");
    }

    #[test]
    fn test_route_network_fields() {
        let line = RouteNetwork::new("metro", "L1");
        let fields = line.fields();
        assert_eq!(fields.len(), RouteNetwork::HEADER.len());
        assert_eq!(fields[0], "metro");
        assert_eq!(fields[1], "L1");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let networks = vec![Network::new("metro").with_name("Metro")];
        let mut out = Vec::new();
        write(Network::FILE_NAME, &networks, &mut out)?;
        let parsed: Vec<Network> = read(Network::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].network_id, networks[0].network_id);
        assert_eq!(parsed[0].network_name, networks[0].network_name);

        let route_networks = vec![RouteNetwork::new("metro", "L1")];
        let mut out = Vec::new();
        write(RouteNetwork::FILE_NAME, &route_networks, &mut out)?;
        let parsed: Vec<RouteNetwork> = read(RouteNetwork::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].network_id, route_networks[0].network_id);
        assert_eq!(parsed[0].route_id, route_networks[0].route_id);
        Ok(())
    }
}
