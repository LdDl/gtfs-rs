//! Fare table readers: `fare_attributes.txt` and `fare_rules.txt`
//! (GTFS-Fares v1), plus the Fares v2 tables `timeframes.txt`,
//! `rider_categories.txt`, `fare_media.txt`, `fare_products.txt`,
//! `fare_leg_rules.txt`, `fare_leg_join_rules.txt` and
//! `fare_transfer_rules.txt`.

use crate::model::{
    DurationLimitType, FareAttributeV1, FareLegJoinRule, FareLegRule, FareMedia, FareMediaType,
    FareProduct, FareRuleV1, FareTransferRule, FareTransferType, FareTransfers, PaymentMethod,
    RiderCategory, Timeframe,
};
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for FareAttributeV1 {
    const FILE_NAME: &'static str = "fare_attributes.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut fare = FareAttributeV1::new(
            row.req("fare_id")?,
            row.req_currency("price")?,
            row.req("currency_type")?,
            row.req_code("payment_method", PaymentMethod::from_code, "code 0-1")?,
        );
        fare.transfers = match row.opt("transfers") {
            None => FareTransfers::Unlimited,
            Some(raw) => {
                let code: i32 = match raw.parse() {
                    Ok(code) => code,
                    Err(_) => return Err(row.invalid("transfers", raw, "code 0-2 or empty")),
                };
                match FareTransfers::from_code(Some(code)) {
                    Some(transfers) => transfers,
                    None => return Err(row.invalid("transfers", raw, "code 0-2 or empty")),
                }
            }
        };
        fare.agency_id = opt_string(row, "agency_id");
        fare.transfer_duration = row.opt_num("transfer_duration", "seconds")?;
        Ok(fare)
    }
}

impl CsvRecord for FareRuleV1 {
    const FILE_NAME: &'static str = "fare_rules.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut rule = FareRuleV1::new(row.req("fare_id")?);
        rule.route_id = opt_string(row, "route_id");
        rule.origin_id = opt_string(row, "origin_id");
        rule.destination_id = opt_string(row, "destination_id");
        rule.contains_id = opt_string(row, "contains_id");
        Ok(rule)
    }
}

impl CsvRecord for Timeframe {
    const FILE_NAME: &'static str = "timeframes.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut timeframe = Timeframe::new(row.req("timeframe_group_id")?, row.req("service_id")?);
        timeframe.start_time = row.opt_time("start_time")?;
        timeframe.end_time = row.opt_time("end_time")?;
        Ok(timeframe)
    }
}

impl CsvRecord for RiderCategory {
    const FILE_NAME: &'static str = "rider_categories.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut category = RiderCategory::new(
            row.req("rider_category_id")?,
            row.req("rider_category_name")?,
        );
        category.is_default_fare_category = row.req_bool01("is_default_fare_category")?;
        category.eligibility_url = opt_string(row, "eligibility_url");
        Ok(category)
    }
}

impl CsvRecord for FareMedia {
    const FILE_NAME: &'static str = "fare_media.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut media = FareMedia::new(
            row.req("fare_media_id")?,
            row.req_code("fare_media_type", FareMediaType::from_code, "code 0-4")?,
        );
        media.fare_media_name = opt_string(row, "fare_media_name");
        Ok(media)
    }
}

impl CsvRecord for FareProduct {
    const FILE_NAME: &'static str = "fare_products.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut product = FareProduct::new(
            row.req("fare_product_id")?,
            row.req_currency("amount")?,
            row.req("currency")?,
        );
        product.fare_product_name = opt_string(row, "fare_product_name");
        product.rider_category_id = opt_string(row, "rider_category_id");
        product.fare_media_id = opt_string(row, "fare_media_id");
        Ok(product)
    }
}

impl CsvRecord for FareLegRule {
    const FILE_NAME: &'static str = "fare_leg_rules.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut rule = FareLegRule::new(row.req("fare_product_id")?);
        rule.leg_group_id = opt_string(row, "leg_group_id");
        rule.network_id = opt_string(row, "network_id");
        rule.from_area_id = opt_string(row, "from_area_id");
        rule.to_area_id = opt_string(row, "to_area_id");
        rule.from_timeframe_group_id = opt_string(row, "from_timeframe_group_id");
        rule.to_timeframe_group_id = opt_string(row, "to_timeframe_group_id");
        rule.rule_priority = row.opt_num("rule_priority", "a non-negative integer")?;
        Ok(rule)
    }
}

impl CsvRecord for FareLegJoinRule {
    const FILE_NAME: &'static str = "fare_leg_join_rules.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut rule = FareLegJoinRule::new(row.req("from_network_id")?, row.req("to_network_id")?);
        rule.from_stop_id = opt_string(row, "from_stop_id");
        rule.to_stop_id = opt_string(row, "to_stop_id");
        Ok(rule)
    }
}

impl CsvRecord for FareTransferRule {
    const FILE_NAME: &'static str = "fare_transfer_rules.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut rule = FareTransferRule::new(row.req_code(
            "fare_transfer_type",
            FareTransferType::from_code,
            "code 0-2",
        )?);
        rule.from_leg_group_id = opt_string(row, "from_leg_group_id");
        rule.to_leg_group_id = opt_string(row, "to_leg_group_id");
        rule.transfer_count = row.opt_num("transfer_count", "-1 or a positive integer")?;
        rule.duration_limit = row.opt_num("duration_limit", "seconds")?;
        rule.duration_limit_type = row.opt_code(
            "duration_limit_type",
            DurationLimitType::from_code,
            "code 0-3",
        )?;
        rule.fare_product_id = opt_string(row, "fare_product_id");
        Ok(rule)
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::CurrencyAmount;
    use crate::model::{
        FareAttributeV1, FareLegRule, FareMedia, FareMediaType, FareProduct, FareRuleV1,
        FareTransferRule, FareTransferType, FareTransfers, PaymentMethod, RiderCategory, Timeframe,
    };
    use crate::parsers::ParseError;
    use crate::parsers::csv::{
        read, read_path,
        test_support::{feed_file, model},
    };

    #[test]
    fn test_sample_fare_attributes() -> Result<(), ParseError> {
        let fares: Vec<FareAttributeV1> = read_path(feed_file("fare_attributes.txt"))?;
        assert_eq!(fares.len(), 2);
        assert_eq!(fares[0].fare_id, "p");
        assert_eq!(
            fares[0].price,
            CurrencyAmount::parse("1.25").map_err(model)?
        );
        assert_eq!(fares[0].currency_type, "USD");
        assert_eq!(fares[0].payment_method, PaymentMethod::OnBoard);
        assert_eq!(fares[0].transfers, FareTransfers::NotAllowed);
        assert!(fares[0].transfer_duration.is_none());
        Ok(())
    }

    #[test]
    fn test_sample_fare_rules() -> Result<(), ParseError> {
        let rules: Vec<FareRuleV1> = read_path(feed_file("fare_rules.txt"))?;
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].fare_id, "p");
        assert_eq!(rules[0].route_id.as_deref(), Some("AB"));
        Ok(())
    }

    #[test]
    fn test_empty_transfers_means_unlimited() -> Result<(), ParseError> {
        let data = "\
fare_id,price,currency_type,payment_method,transfers
base,57.00,RUB,1,
";
        let fares: Vec<FareAttributeV1> = read("fare_attributes.txt", data.as_bytes())?;
        assert_eq!(fares[0].transfers, FareTransfers::Unlimited);
        Ok(())
    }

    #[test]
    fn test_timeframes() -> Result<(), ParseError> {
        let data = "\
timeframe_group_id,start_time,end_time,service_id
peak,07:00:00,10:00:00,weekday
";
        let timeframes: Vec<Timeframe> = read("timeframes.txt", data.as_bytes())?;
        assert_eq!(timeframes[0].start_time, Some(7 * 3600));
        assert_eq!(timeframes[0].end_time, Some(10 * 3600));
        Ok(())
    }

    #[test]
    fn test_rider_categories() -> Result<(), ParseError> {
        let data = "\
rider_category_id,rider_category_name,is_default_fare_category
adult,Adult,1
child,Child,0
";
        let categories: Vec<RiderCategory> = read("rider_categories.txt", data.as_bytes())?;
        assert!(categories[0].is_default_fare_category);
        assert!(!categories[1].is_default_fare_category);
        Ok(())
    }

    #[test]
    fn test_fare_media_and_products() -> Result<(), ParseError> {
        let media: Vec<FareMedia> = read(
            "fare_media.txt",
            "fare_media_id,fare_media_type\ncard,2\n".as_bytes(),
        )?;
        assert_eq!(media[0].fare_media_type, FareMediaType::TransitCard);

        let products: Vec<FareProduct> = read(
            "fare_products.txt",
            "fare_product_id,amount,currency\nsingle,57.00,RUB\n".as_bytes(),
        )?;
        assert_eq!(
            products[0].amount,
            CurrencyAmount::parse("57.00").map_err(model)?
        );
        Ok(())
    }

    #[test]
    fn test_fare_leg_and_transfer_rules() -> Result<(), ParseError> {
        let legs: Vec<FareLegRule> = read(
            "fare_leg_rules.txt",
            "leg_group_id,network_id,fare_product_id\nmetro_leg,metro,single\n".as_bytes(),
        )?;
        assert_eq!(legs[0].leg_group_id.as_deref(), Some("metro_leg"));

        let transfers: Vec<FareTransferRule> = read(
            "fare_transfer_rules.txt",
            "from_leg_group_id,to_leg_group_id,fare_transfer_type,duration_limit,\
duration_limit_type\nmetro_leg,metro_leg,0,5400,1\n"
                .as_bytes(),
        )?;
        assert_eq!(
            transfers[0].fare_transfer_type,
            FareTransferType::FromLegPlusTransfer
        );
        assert_eq!(transfers[0].duration_limit, Some(5400));
        Ok(())
    }
}
