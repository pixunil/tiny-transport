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
            incomplete_line.add_color_when_applicable(&colors);
        }

        action.complete("Imported line colors");
        Ok(())
    }

    pub(crate) fn id_mapping(&self) -> &HashMap<LineId, usize> {
        &self.id_mapping
    }

    pub(crate) fn line_count(&self) -> usize {
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
    use crate::fixtures::{lines, paths, routes};
    use test_utils::{assert_eq_alternate, map};

    #[test]
    fn test_import() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "S1",             109;
                2,        1,         "S42",            109;
                3,        2,         "U4",             400;
                4,        2,         "12",             900
        );

        let importer = Importer::import_lines(&mut dataset).unwrap();
        assert_eq!(importer.line_count(), 4);
        assert_eq_alternate!(
            importer.id_mapping(),
            &map! {
                "1" => 0,
                "2" => 1,
                "3" => 2,
                "4" => 3,
            }
        );
    }

    #[test]
    fn test_import_deduplication() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "S1",             109;
                2,        1,         "S1",             109
        );

        let importer = Importer::import_lines(&mut dataset).unwrap();
        assert_eq!(importer.line_count(), 1);
        assert_eq_alternate!(
            importer.id_mapping(),
            &map! {
                "1" => 0,
                "2" => 0,
            }
        );
    }

    #[test]
    fn test_finish() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "S1",             109;
                2,        1,         "S42",            109;
                3,        2,         "U4",             400;
                4,        2,         "12",             900
            colors:
                line,         color;
                "S1",         "#dc6ba6";
                "S42",        "#cc6112";
                "U4",         "#ffd900"
        );

        let importer = Importer::import(&mut dataset).unwrap();
        let (_, segment_ids) = paths::tram_12::segments();
        let lines = importer
            .finish(vec![
                vec![],
                vec![],
                vec![],
                vec![routes::tram_12::oranienburger_tor_am_kupfergraben(
                    &segment_ids,
                )],
            ])
            .unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[&"1".into()].contains(&lines::s1()));
        assert!(lines[&"1".into()].contains(&lines::s42()));
        assert!(lines[&"2".into()].contains(&lines::u4()));
        assert!(lines[&"2".into()].contains(&lines::tram_12_with_route(&segment_ids)));
    }
}
