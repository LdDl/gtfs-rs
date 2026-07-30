//! `translations.txt` writer.

use crate::model::Translation;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Translation {
    const FILE_NAME: &'static str = "translations.txt";

    const HEADER: &'static [&'static str] = &[
        "table_name",
        "field_name",
        "language",
        "translation",
        "record_id",
        "record_sub_id",
        "field_value",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.table_name.name().to_string(),
            self.field_name.clone(),
            self.language.clone(),
            self.translation.clone(),
            self.record_id.clone().unwrap_or_default(),
            self.record_sub_id.clone().unwrap_or_default(),
            self.field_value.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{TableName, Translation};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let translation =
            Translation::new(TableName::Stops, "stop_name", "ru", "Центральная").for_record("S1");
        let fields = translation.fields();
        assert_eq!(Translation::HEADER.len(), fields.len());
        assert_eq!(fields[0], "stops");
        assert_eq!(fields[1], "stop_name");
        assert_eq!(fields[2], "ru");
        assert_eq!(fields[3], "Центральная");
        assert_eq!(fields[4], "S1");
        assert_eq!(fields[5], "");
        assert_eq!(fields[6], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let translations = vec![
            Translation::new(TableName::Stops, "stop_name", "ru", "Центральная").for_record("S1"),
            Translation::new(TableName::Routes, "route_long_name", "en", "Central Line")
                .for_field_value("Центральная линия"),
        ];
        let mut out = Vec::new();
        write("translations.txt", &translations, &mut out)?;
        let parsed: Vec<Translation> = read("translations.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].table_name, TableName::Stops);
        assert_eq!(parsed[0].field_name, "stop_name");
        assert_eq!(parsed[0].language, "ru");
        assert_eq!(parsed[0].translation, "Центральная");
        assert_eq!(parsed[0].record_id.as_deref(), Some("S1"));
        assert_eq!(parsed[0].field_value, None);
        assert_eq!(parsed[1].table_name, TableName::Routes);
        assert_eq!(parsed[1].record_id, None);
        assert_eq!(parsed[1].field_value.as_deref(), Some("Центральная линия"));
        Ok(())
    }
}
