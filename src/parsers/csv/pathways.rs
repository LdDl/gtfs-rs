//! `pathways.txt` reader.

use super::{CsvRecord, Row, opt_string};
use crate::model::{Pathway, PathwayMode};
use crate::parsers::ParseError;

impl CsvRecord for Pathway {
    const FILE_NAME: &'static str = "pathways.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut pathway = Pathway::new(
            row.req("pathway_id")?,
            row.req("from_stop_id")?,
            row.req("to_stop_id")?,
            row.req_code("pathway_mode", PathwayMode::from_code, "code 1-7")?,
            row.req_bool01("is_bidirectional")?,
        );
        pathway.length = row.opt_num("length", "meters")?;
        pathway.traversal_time = row.opt_num("traversal_time", "seconds")?;
        pathway.stair_count = row.opt_num("stair_count", "an integer")?;
        pathway.max_slope = row.opt_num("max_slope", "a slope ratio")?;
        pathway.min_width = row.opt_num("min_width", "meters")?;
        pathway.signposted_as = opt_string(row, "signposted_as");
        pathway.reversed_signposted_as = opt_string(row, "reversed_signposted_as");
        Ok(pathway)
    }
}

#[cfg(test)]
mod tests {
    use super::super::read;
    use crate::model::{Pathway, PathwayMode};
    use crate::parsers::ParseError;

    #[test]
    fn test_pathways() -> Result<(), ParseError> {
        let data = "\
pathway_id,from_stop_id,to_stop_id,pathway_mode,is_bidirectional,stair_count
pw1,entrance_1,platform_2,2,0,-15
";
        let pathways: Vec<Pathway> = read("pathways.txt", data.as_bytes())?;
        assert_eq!(pathways[0].pathway_mode, PathwayMode::Stairs);
        assert!(!pathways[0].is_bidirectional);
        // negative stair count: going down in the from-to direction
        assert_eq!(pathways[0].stair_count, Some(-15));
        Ok(())
    }
}
