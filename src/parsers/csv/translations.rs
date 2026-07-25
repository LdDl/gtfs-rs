//! `translations.txt` reader.

use super::{CsvRecord, Row, opt_string};
use crate::model::{TableName, Translation};
use crate::parsers::ParseError;

impl CsvRecord for Translation {
    const FILE_NAME: &'static str = "translations.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let raw_table = row.req("table_name")?;
        let table_name = match TableName::from_name(raw_table) {
            Some(table_name) => table_name,
            None => return Err(row.invalid("table_name", raw_table, "a GTFS table name")),
        };
        let mut translation = Translation::new(
            table_name,
            row.req("field_name")?,
            row.req("language")?,
            row.req("translation")?,
        );
        translation.record_id = opt_string(row, "record_id");
        translation.record_sub_id = opt_string(row, "record_sub_id");
        translation.field_value = opt_string(row, "field_value");
        Ok(translation)
    }
}

#[cfg(test)]
mod tests {
    use super::super::read;
    use crate::model::{TableName, Translation};
    use crate::parsers::{ParseError, ParseErrorKind};

    #[test]
    fn test_translations() -> Result<(), ParseError> {
        let data = "\
table_name,field_name,language,translation,record_id
stops,stop_name,ru,Центральная,S1
";
        let translations: Vec<Translation> = read("translations.txt", data.as_bytes())?;
        assert_eq!(translations[0].table_name, TableName::Stops);
        assert_eq!(translations[0].translation, "Центральная");
        assert_eq!(translations[0].record_id.as_deref(), Some("S1"));
        Ok(())
    }

    #[test]
    fn test_unknown_table_name_is_rejected() {
        let data = "\
table_name,field_name,language,translation
fare_rules,fare_id,en,whatever
";
        let result: Result<Vec<Translation>, ParseError> =
            read("translations.txt", data.as_bytes());
        let Err(err) = result else {
            panic!("expected an invalid table_name error");
        };
        assert_eq!(err.field.as_deref(), Some("table_name"));
        assert!(matches!(err.kind, ParseErrorKind::Invalid { .. }));
    }
}
