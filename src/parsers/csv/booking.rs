//! `booking_rules.txt` reader (GTFS-Flex).

use super::{CsvRecord, Row, opt_string};
use crate::model::{BookingRule, BookingType};
use crate::parsers::ParseError;

impl CsvRecord for BookingRule {
    const FILE_NAME: &'static str = "booking_rules.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut rule = BookingRule::new(
            row.req("booking_rule_id")?,
            row.req_code("booking_type", BookingType::from_code, "code 0-2")?,
        );
        rule.prior_notice_duration_min = row.opt_num("prior_notice_duration_min", "minutes")?;
        rule.prior_notice_duration_max = row.opt_num("prior_notice_duration_max", "minutes")?;
        rule.prior_notice_last_day = row.opt_num("prior_notice_last_day", "days")?;
        rule.prior_notice_last_time = row.opt_time("prior_notice_last_time")?;
        rule.prior_notice_start_day = row.opt_num("prior_notice_start_day", "days")?;
        rule.prior_notice_start_time = row.opt_time("prior_notice_start_time")?;
        rule.prior_notice_service_id = opt_string(row, "prior_notice_service_id");
        rule.message = opt_string(row, "message");
        rule.pickup_message = opt_string(row, "pickup_message");
        rule.drop_off_message = opt_string(row, "drop_off_message");
        rule.phone_number = opt_string(row, "phone_number");
        rule.info_url = opt_string(row, "info_url");
        rule.booking_url = opt_string(row, "booking_url");
        Ok(rule)
    }
}

#[cfg(test)]
mod tests {
    use super::super::read;
    use crate::model::{BookingRule, BookingType};
    use crate::parsers::ParseError;

    #[test]
    fn test_booking_rules() -> Result<(), ParseError> {
        let data = "\
booking_rule_id,booking_type,prior_notice_duration_min,prior_notice_last_time
br1,1,60,17:00:00
";
        let rules: Vec<BookingRule> = read("booking_rules.txt", data.as_bytes())?;
        assert_eq!(rules[0].booking_type, BookingType::SameDay);
        assert_eq!(rules[0].prior_notice_duration_min, Some(60));
        assert_eq!(rules[0].prior_notice_last_time, Some(17 * 3600));
        Ok(())
    }
}
