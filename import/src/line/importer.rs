use std::error::Error;
use std::collections::HashMap;
use std::time::Instant;

use crate::utils::{Dataset, progress::elapsed};
use crate::agency::AgencyId;
use crate::trip::Route;
use super::{Line, LineId, IncompleteLine, LineRecord, LineColorRecord};

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

        let records = dataset.read_csv("routes.txt", "Importing lines")?;
        let started = Instant::now();
        for result in records {
            let record: LineRecord = result?;
            record.deduplicate(&mut id_mapping, &mut incomplete_lines);
        }

        eprintln!("Imported {} lines in {:.2}s", incomplete_lines.len(), elapsed(started));
        Ok(Self { id_mapping, incomplete_lines })
    }

    fn import_colors(&mut self, dataset: &mut impl Dataset) -> Result<(), Box<dyn Error>> {
        let mut colors = HashMap::new();

        let records = dataset.read_csv("colors.txt", "Importing colors")?;
        let started = Instant::now();
        for result in records {
            let record: LineColorRecord = result?;
            record.import(&mut colors);
        }

        for incomplete_line in &mut self.incomplete_lines {
            incomplete_line.add_color_when_applicable(&mut colors);
        }

        eprintln!("Imported line colors in {:.2}s", elapsed(started));
        Ok(())
    }

    pub(crate) fn id_mapping(&self) -> &HashMap<LineId, usize> {
        &self.id_mapping
    }

    pub(crate) fn num_lines(&self) -> usize {
        self.incomplete_lines.len()
    }

    pub(crate) fn finish(self, mut routes: Vec<Vec<Route>>) -> Result<HashMap<AgencyId, Vec<Line>>, Box<dyn Error>> {
        let mut lines = HashMap::new();
        for incomplete_line in self.incomplete_lines.into_iter().rev() {
            incomplete_line.finish(routes.pop().unwrap(), &mut lines);
        }

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{map, dataset};
    use crate::line::fixtures::*;

    #[test]
    fn test_deduplication() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "Blue Line",      109;
                2,        1,         "Blue Line",      109
        );

        let importer = Importer::import_lines(&mut dataset).unwrap();
        let id_mapping = importer.id_mapping;
        assert_eq!(id_mapping, map! {
            "1" => 0,
            "2" => 0,
        });
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "Blue Line",      109;
                2,        1,         "Green Line",     109
            colors:
                line,         color;
                "Blue Line",  "#0000ff";
                "Green Line", "#00ff00"
        );

        let importer = Importer::import(&mut dataset).unwrap();
        let lines = importer.finish(vec![Vec::new(), Vec::new()]).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[&"1".into()].contains(&lines::blue()));
        assert!(lines[&"1".into()].contains(&lines::green()));
    }
}
