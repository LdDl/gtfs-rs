//! `levels.txt` reader.

use crate::model::Level;
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row, opt_string};

impl CsvRecord for Level {
    const FILE_NAME: &'static str = "levels.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut level = Level::new(
            row.req("level_id")?,
            row.req_finite("level_index", "a level index")?,
        );
        level.level_name = opt_string(row, "level_name");
        Ok(level)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Level;
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_levels() -> Result<(), ParseError> {
        let data = "level_id,level_index,level_name\nL-1,-1.0,Concourse\n";
        let levels: Vec<Level> = read("levels.txt", data.as_bytes())?;
        assert_eq!(levels[0].level_index, -1.0);
        assert_eq!(levels[0].level_name.as_deref(), Some("Concourse"));
        Ok(())
    }
}
