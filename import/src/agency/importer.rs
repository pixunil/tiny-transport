use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use super::{Agency, AgencyId, AgencyRecord};
use crate::utils::{progress::elapsed, Dataset};
use crate::Line;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
        mut lines: HashMap<AgencyId, Vec<Line>>,
    ) -> Result<Vec<Agency>, Box<dyn Error>> {
        let mut agencies = Vec::new();

        let records = dataset.read_csv("agency.txt", "Importing agencies")?;
        let started = Instant::now();
        for result in records {
            let record: AgencyRecord = result?;
            let agency = record.import(&mut lines);
            agencies.push(agency);
        }

        eprintln!(
            "Imported {} agencies in {:.2}s",
            agencies.len(),
            elapsed(started)
        );
        Ok(agencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agency::fixtures::*;
    use crate::map;

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            agency:
                agency_id, agency_name;
                1,         "Public Transport"
        );

        let lines = map! {
            "1" => vec![lines::blue()],
        };
        let agencies = Importer::import(&mut dataset, lines).unwrap();
        assert_eq!(agencies, vec![agencies::pubtrans(vec![lines::blue()])]);
    }
}
