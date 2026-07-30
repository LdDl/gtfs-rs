//! `networks.txt` and `route_networks.txt` readers.

use crate::model::{Network, RouteNetwork};
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for Network {
    const FILE_NAME: &'static str = "networks.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut network = Network::new(row.req("network_id")?);
        network.network_name = opt_string(row, "network_name");
        Ok(network)
    }
}

impl CsvRecord for RouteNetwork {
    const FILE_NAME: &'static str = "route_networks.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        Ok(RouteNetwork::new(
            row.req("network_id")?,
            row.req("route_id")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Network, RouteNetwork};
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_networks_and_route_networks() -> Result<(), ParseError> {
        let networks: Vec<Network> = read(
            "networks.txt",
            "network_id,network_name\nmetro,Metro\n".as_bytes(),
        )?;
        assert_eq!(networks[0].network_name.as_deref(), Some("Metro"));

        let route_networks: Vec<RouteNetwork> = read(
            "route_networks.txt",
            "network_id,route_id\nmetro,L1\n".as_bytes(),
        )?;
        assert_eq!(route_networks[0].route_id, "L1");
        Ok(())
    }
}
