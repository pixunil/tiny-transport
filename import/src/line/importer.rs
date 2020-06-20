use std::collections::HashMap;
use std::error::Error;

use super::{IncompleteLine, Line, LineColorRecord, LineId, LineRecord};
use crate::agency::AgencyId;
use crate::trip::Route;
use crate::utils::{Action, Dataset};

pub(crate) struct Importer {
    id_mapping: HashMap<LineId, usize>,
    incomplete_lines: Vec<IncompleteLine>,
}

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset) -> Result<Importer, Box<dyn Error>> {
        let mut importer = Self::import_lines(dataset)?;
        importer.import_colors(dataset)?;
        Ok(importer)
    }

    fn import_lines(dataset: &mut impl Dataset) -> Result<Self, Box<dyn Error>> {
        let mut id_mapping = HashMap::new();
        let mut incomplete_lines = Vec::new();

        let action = Action::start("Importing lines");
        for result in action.read_csv(dataset, "routes.txt")? {
            let record: LineRecord = result?;
            record.deduplicate(&mut id_mapping, &mut incomplete_lines);
        }
        action.complete(&format!("Imported {} lines", incomplete_lines.len()));
        Ok(Self {
            id_mapping,
            incomplete_lines,
        })
    }

    fn import_colors(&mut self, dataset: &mut impl Dataset) -> Result<(), Box<dyn Error>> {
        let mut colors = HashMap::new();

        let action = Action::start("Importing colors");
        for result in action.read_csv(dataset, "colors.txt")? {
            let record: LineColorRecord = result?;
            record.import(&mut colors);
        }

        for incomplete_line in &mut self.incomplete_lines {
            incomplete_line.add_color_when_applicable(&mut colors);
        }

        action.complete("Imported line colors");
        Ok(())
    }

    pub(crate) fn id_mapping(&self) -> &HashMap<LineId, usize> {
        &self.id_mapping
    }

    pub(crate) fn num_lines(&self) -> usize {
        self.incomplete_lines.len()
    }

    pub(crate) fn finish(
        self,
        mut routes: Vec<Vec<Route>>,
    ) -> Result<HashMap<AgencyId, Vec<Line>>, Box<dyn Error>> {
        let mut lines = HashMap::new();
        let mut action = Action::start("Adding routes to lines");
        let incomplete_lines = self.incomplete_lines.into_iter().rev();
        for incomplete_line in action.wrap_iter(incomplete_lines) {
            incomplete_line.finish(routes.pop().unwrap(), &mut lines);
        }
        action.complete("Added routes to lines");

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset;
    use crate::fixtures::lines;
    use test_utils::map;

    #[test]
    fn test_deduplication() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "Blue Line",      109;
                2,        1,         "Blue Line",      109
        );

        let importer = Importer::import_lines(&mut dataset).unwrap();
        assert_eq!(
            importer.id_mapping,
            map! {
                "1" => 0,
                "2" => 0,
            }
        );
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "S42",            109;
                2,        1,         "U4",             400
            colors:
                line,         color;
                "S42",        "#cc6112";
                "U4",         "#ffd900"
        );

        let importer = Importer::import(&mut dataset).unwrap();
        let lines = importer.finish(vec![Vec::new(), Vec::new()]).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[&"1".into()].contains(&lines::s42()));
        assert!(lines[&"1".into()].contains(&lines::u4()));
    }
}
