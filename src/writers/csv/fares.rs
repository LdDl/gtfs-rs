//! Fare table writers: `fare_attributes.txt` and `fare_rules.txt`
//! (GTFS-Fares v1), plus the Fares v2 tables `timeframes.txt`,
//! `rider_categories.txt`, `fare_media.txt`, `fare_products.txt`,
//! `fare_leg_rules.txt`, `fare_leg_join_rules.txt` and
//! `fare_transfer_rules.txt`.

use crate::misc::format_gtfs_time;
use crate::model::{
    FareAttributeV1, FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1,
    FareTransferRule, RiderCategory, Timeframe,
};
use crate::writers::csv::CsvWrite;

impl CsvWrite for FareAttributeV1 {
    const FILE_NAME: &'static str = "fare_attributes.txt";

    const HEADER: &'static [&'static str] = &[
        "fare_id",
        "price",
        "currency_type",
        "payment_method",
        "transfers",
        "agency_id",
        "transfer_duration",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.fare_id.clone(),
            self.price.to_string(),
            self.currency_type.clone(),
            self.payment_method.code().to_string(),
            self.transfers
                .code()
                .map(|c| c.to_string())
                .unwrap_or_default(),
            self.agency_id.clone().unwrap_or_default(),
            self.transfer_duration
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ]
    }
}

impl CsvWrite for FareRuleV1 {
    const FILE_NAME: &'static str = "fare_rules.txt";

    const HEADER: &'static [&'static str] = &[
        "fare_id",
        "route_id",
        "origin_id",
        "destination_id",
        "contains_id",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.fare_id.clone(),
            self.route_id.clone().unwrap_or_default(),
            self.origin_id.clone().unwrap_or_default(),
            self.destination_id.clone().unwrap_or_default(),
            self.contains_id.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for Timeframe {
    const FILE_NAME: &'static str = "timeframes.txt";

    const HEADER: &'static [&'static str] =
        &["timeframe_group_id", "start_time", "end_time", "service_id"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.timeframe_group_id.clone(),
            self.start_time.map(format_gtfs_time).unwrap_or_default(),
            self.end_time.map(format_gtfs_time).unwrap_or_default(),
            self.service_id.clone(),
        ]
    }
}

impl CsvWrite for RiderCategory {
    const FILE_NAME: &'static str = "rider_categories.txt";

    const HEADER: &'static [&'static str] = &[
        "rider_category_id",
        "rider_category_name",
        "is_default_fare_category",
        "eligibility_url",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.rider_category_id.clone(),
            self.rider_category_name.clone(),
            if self.is_default_fare_category {
                "1".to_string()
            } else {
                "0".to_string()
            },
            self.eligibility_url.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for FareMedia {
    const FILE_NAME: &'static str = "fare_media.txt";

    const HEADER: &'static [&'static str] =
        &["fare_media_id", "fare_media_name", "fare_media_type"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.fare_media_id.clone(),
            self.fare_media_name.clone().unwrap_or_default(),
            self.fare_media_type.code().to_string(),
        ]
    }
}

impl CsvWrite for FareProduct {
    const FILE_NAME: &'static str = "fare_products.txt";

    const HEADER: &'static [&'static str] = &[
        "fare_product_id",
        "fare_product_name",
        "rider_category_id",
        "fare_media_id",
        "amount",
        "currency",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.fare_product_id.clone(),
            self.fare_product_name.clone().unwrap_or_default(),
            self.rider_category_id.clone().unwrap_or_default(),
            self.fare_media_id.clone().unwrap_or_default(),
            self.amount.to_string(),
            self.currency.clone(),
        ]
    }
}

impl CsvWrite for FareLegRule {
    const FILE_NAME: &'static str = "fare_leg_rules.txt";

    const HEADER: &'static [&'static str] = &[
        "leg_group_id",
        "network_id",
        "from_area_id",
        "to_area_id",
        "from_timeframe_group_id",
        "to_timeframe_group_id",
        "fare_product_id",
        "rule_priority",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.leg_group_id.clone().unwrap_or_default(),
            self.network_id.clone().unwrap_or_default(),
            self.from_area_id.clone().unwrap_or_default(),
            self.to_area_id.clone().unwrap_or_default(),
            self.from_timeframe_group_id.clone().unwrap_or_default(),
            self.to_timeframe_group_id.clone().unwrap_or_default(),
            self.fare_product_id.clone(),
            self.rule_priority
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ]
    }
}

impl CsvWrite for FareLegJoinRule {
    const FILE_NAME: &'static str = "fare_leg_join_rules.txt";

    const HEADER: &'static [&'static str] = &[
        "from_network_id",
        "to_network_id",
        "from_stop_id",
        "to_stop_id",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.from_network_id.clone(),
            self.to_network_id.clone(),
            self.from_stop_id.clone().unwrap_or_default(),
            self.to_stop_id.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for FareTransferRule {
    const FILE_NAME: &'static str = "fare_transfer_rules.txt";

    const HEADER: &'static [&'static str] = &[
        "from_leg_group_id",
        "to_leg_group_id",
        "transfer_count",
        "duration_limit",
        "duration_limit_type",
        "fare_transfer_type",
        "fare_product_id",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.from_leg_group_id.clone().unwrap_or_default(),
            self.to_leg_group_id.clone().unwrap_or_default(),
            self.transfer_count
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.duration_limit
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.duration_limit_type
                .map(|v| v.code().to_string())
                .unwrap_or_default(),
            self.fare_transfer_type.code().to_string(),
            self.fare_product_id.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::CurrencyAmount;
    use crate::model::{
        DurationLimitType, FareAttributeV1, FareLegJoinRule, FareLegRule, FareMedia, FareMediaType,
        FareProduct, FareRuleV1, FareTransferRule, FareTransferType, FareTransfers, PaymentMethod,
        RiderCategory, Timeframe,
    };
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fare_attribute_fields() -> Result<(), Box<dyn std::error::Error>> {
        let fare = FareAttributeV1::new(
            "base",
            CurrencyAmount::parse("57.00")?,
            "RUB",
            PaymentMethod::BeforeBoarding,
        )
        .with_transfers(FareTransfers::Once);
        let fields = fare.fields();
        assert_eq!(fields.len(), FareAttributeV1::HEADER.len());
        assert_eq!(fields[1], "57.00");
        assert_eq!(fields[3], "1");
        assert_eq!(fields[4], "1");
        assert_eq!(fields[5], "");
        assert_eq!(fields[6], "");
        Ok(())
    }

    #[test]
    fn test_unlimited_transfers_renders_empty() -> Result<(), Box<dyn std::error::Error>> {
        let fare = FareAttributeV1::new(
            "base",
            CurrencyAmount::parse("57.00")?,
            "RUB",
            PaymentMethod::OnBoard,
        );
        assert_eq!(fare.transfers, FareTransfers::Unlimited);
        assert_eq!(fare.fields()[4], "");
        Ok(())
    }

    #[test]
    fn test_fare_rule_fields() {
        let rule = FareRuleV1::new("base").with_origin_destination("zone_a", "zone_b");
        let fields = rule.fields();
        assert_eq!(fields.len(), FareRuleV1::HEADER.len());
        assert_eq!(fields[0], "base");
        assert_eq!(fields[1], "");
        assert_eq!(fields[2], "zone_a");
        assert_eq!(fields[3], "zone_b");
    }

    #[test]
    fn test_timeframe_fields() {
        let frame = Timeframe::new("peak", "weekday").with_period(7 * 3600, 10 * 3600);
        let fields = frame.fields();
        assert_eq!(fields.len(), Timeframe::HEADER.len());
        assert_eq!(fields[1], "07:00:00");
        assert_eq!(fields[2], "10:00:00");

        let all_day = Timeframe::new("all", "weekday");
        assert_eq!(all_day.fields()[1], "");
        assert_eq!(all_day.fields()[2], "");
    }

    #[test]
    fn test_rider_category_fields() {
        let adult = RiderCategory::new("adult", "Adult").as_default();
        let fields = adult.fields();
        assert_eq!(fields.len(), RiderCategory::HEADER.len());
        assert_eq!(fields[2], "1");

        let child = RiderCategory::new("child", "Child");
        assert_eq!(child.fields()[2], "0");
    }

    #[test]
    fn test_fare_media_fields() {
        let card = FareMedia::new("card", FareMediaType::TransitCard).with_name("Troika");
        let fields = card.fields();
        assert_eq!(fields.len(), FareMedia::HEADER.len());
        assert_eq!(fields[1], "Troika");
        assert_eq!(fields[2], "2");
    }

    #[test]
    fn test_fare_product_fields() -> Result<(), Box<dyn std::error::Error>> {
        let single = FareProduct::new("single", CurrencyAmount::parse("57.00")?, "RUB")
            .with_rider_category("adult");
        let fields = single.fields();
        assert_eq!(fields.len(), FareProduct::HEADER.len());
        assert_eq!(fields[1], "");
        assert_eq!(fields[2], "adult");
        assert_eq!(fields[4], "57.00");
        assert_eq!(fields[5], "RUB");
        Ok(())
    }

    #[test]
    fn test_fare_leg_rule_fields() {
        let leg = FareLegRule::new("single")
            .with_leg_group("metro_leg")
            .with_network("metro");
        let fields = leg.fields();
        assert_eq!(fields.len(), FareLegRule::HEADER.len());
        assert_eq!(fields[0], "metro_leg");
        assert_eq!(fields[1], "metro");
        assert_eq!(fields[6], "single");
        assert_eq!(fields[7], "");
    }

    #[test]
    fn test_fare_leg_join_rule_fields() {
        let join = FareLegJoinRule::new("net_a", "net_b").between_stops("S1", "S2");
        let fields = join.fields();
        assert_eq!(fields.len(), FareLegJoinRule::HEADER.len());
        assert_eq!(fields[0], "net_a");
        assert_eq!(fields[3], "S2");
    }

    #[test]
    fn test_fare_transfer_rule_fields() {
        let rule = FareTransferRule::new(FareTransferType::FromLegPlusTransfer)
            .between_leg_groups("metro_leg", "metro_leg")
            .with_duration_limit(5400, DurationLimitType::DepartureToArrival);
        let fields = rule.fields();
        assert_eq!(fields.len(), FareTransferRule::HEADER.len());
        assert_eq!(fields[2], "");
        assert_eq!(fields[3], "5400");
        assert_eq!(fields[4], "0");
        assert_eq!(fields[5], "0");
        assert_eq!(fields[6], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let fares = vec![
            FareAttributeV1::new(
                "base",
                CurrencyAmount::parse("57.00")?,
                "RUB",
                PaymentMethod::OnBoard,
            )
            .with_transfers(FareTransfers::NotAllowed),
        ];
        let mut out = Vec::new();
        write(FareAttributeV1::FILE_NAME, &fares, &mut out)?;
        let parsed: Vec<FareAttributeV1> = read(FareAttributeV1::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].fare_id, fares[0].fare_id);
        assert_eq!(parsed[0].price, fares[0].price);
        assert_eq!(parsed[0].payment_method, fares[0].payment_method);
        assert_eq!(parsed[0].transfers, fares[0].transfers);

        let products = vec![
            FareProduct::new("single", CurrencyAmount::parse("57.00")?, "RUB")
                .with_name("Single ride"),
        ];
        let mut out = Vec::new();
        write(FareProduct::FILE_NAME, &products, &mut out)?;
        let parsed: Vec<FareProduct> = read(FareProduct::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].fare_product_id, products[0].fare_product_id);
        assert_eq!(parsed[0].fare_product_name, products[0].fare_product_name);
        assert_eq!(parsed[0].amount, products[0].amount);
        assert_eq!(parsed[0].currency, products[0].currency);

        let frames = vec![Timeframe::new("peak", "weekday").with_period(7 * 3600, 10 * 3600)];
        let mut out = Vec::new();
        write(Timeframe::FILE_NAME, &frames, &mut out)?;
        let parsed: Vec<Timeframe> = read(Timeframe::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].timeframe_group_id, frames[0].timeframe_group_id);
        assert_eq!(parsed[0].start_time, frames[0].start_time);
        assert_eq!(parsed[0].end_time, frames[0].end_time);
        assert_eq!(parsed[0].service_id, frames[0].service_id);
        Ok(())
    }
}
