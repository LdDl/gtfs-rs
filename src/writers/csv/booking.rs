//! `booking_rules.txt` writer (GTFS-Flex).

use crate::model::BookingRule;
use crate::writers::csv::CsvWrite;

impl CsvWrite for BookingRule {
    const FILE_NAME: &'static str = "booking_rules.txt";

    const HEADER: &'static [&'static str] = &[
        "booking_rule_id",
        "booking_type",
        "prior_notice_duration_min",
        "prior_notice_duration_max",
        "prior_notice_last_day",
        "prior_notice_last_time",
        "prior_notice_start_day",
        "prior_notice_start_time",
        "prior_notice_service_id",
        "message",
        "pickup_message",
        "drop_off_message",
        "phone_number",
        "info_url",
        "booking_url",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.booking_rule_id.clone(),
            self.booking_type.code().to_string(),
            self.prior_notice_duration_min
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.prior_notice_duration_max
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.prior_notice_last_day
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.prior_notice_last_time
                .map(crate::misc::format_gtfs_time)
                .unwrap_or_default(),
            self.prior_notice_start_day
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.prior_notice_start_time
                .map(crate::misc::format_gtfs_time)
                .unwrap_or_default(),
            self.prior_notice_service_id.clone().unwrap_or_default(),
            self.message.clone().unwrap_or_default(),
            self.pickup_message.clone().unwrap_or_default(),
            self.drop_off_message.clone().unwrap_or_default(),
            self.phone_number.clone().unwrap_or_default(),
            self.info_url.clone().unwrap_or_default(),
            self.booking_url.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{BookingRule, BookingType};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_booking_rule_fields() {
        let mut rule =
            BookingRule::new("br1", BookingType::PriorDays).with_phone_number("+7 495 000-00-00");
        rule.prior_notice_last_day = Some(1);
        rule.prior_notice_last_time = Some(17 * 3600);
        let fields = rule.fields();
        assert_eq!(fields.len(), BookingRule::HEADER.len());
        assert_eq!(fields[0], "br1");
        assert_eq!(fields[1], "2");
        assert_eq!(fields[4], "1");
        // seconds since midnight render as an HH:MM:SS time
        assert_eq!(fields[5], "17:00:00");
        assert_eq!(fields[7], "");
        assert_eq!(fields[12], "+7 495 000-00-00");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_booking_rule_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut rule =
            BookingRule::new("br1", BookingType::SameDay).with_booking_url("https://book.example");
        rule.prior_notice_duration_min = Some(60);
        rule.prior_notice_last_time = Some(17 * 3600);
        rule.message = Some("Call ahead".to_string());
        let mut out = Vec::new();
        write("booking_rules.txt", &[rule], &mut out)?;
        let parsed: Vec<BookingRule> = read("booking_rules.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].booking_rule_id, "br1");
        assert_eq!(parsed[0].booking_type, BookingType::SameDay);
        assert_eq!(parsed[0].prior_notice_duration_min, Some(60));
        assert_eq!(parsed[0].prior_notice_last_time, Some(17 * 3600));
        assert_eq!(parsed[0].message.as_deref(), Some("Call ahead"));
        assert_eq!(
            parsed[0].booking_url.as_deref(),
            Some("https://book.example")
        );
        assert_eq!(parsed[0].prior_notice_start_time, None);
        Ok(())
    }
}
