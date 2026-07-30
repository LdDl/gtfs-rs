//! `levels.txt` writer.

use crate::model::Level;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Level {
    const FILE_NAME: &'static str = "levels.txt";

    const HEADER: &'static [&'static str] = &["level_id", "level_index", "level_name"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.level_id.clone(),
            self.level_index.to_string(),
            self.level_name.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Level;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_level_fields() {
        let concourse = Level::new("L-1", -1.0).with_name("Concourse");
        let fields = concourse.fields();
        assert_eq!(fields.len(), Level::HEADER.len());
        assert_eq!(fields[0], "L-1");
        assert_eq!(fields[1], "-1");
        assert_eq!(fields[2], "Concourse");
        // absent optional name renders as an empty field
        let bare = Level::new("L0", 0.0);
        assert_eq!(bare.fields()[2], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_level_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let levels = vec![Level::new("L-1", -1.5).with_name("Concourse")];
        let mut out = Vec::new();
        write("levels.txt", &levels, &mut out)?;
        let parsed: Vec<Level> = read("levels.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].level_id, "L-1");
        assert_eq!(parsed[0].level_index, -1.5);
        assert_eq!(parsed[0].level_name.as_deref(), Some("Concourse"));
        Ok(())
    }
}
