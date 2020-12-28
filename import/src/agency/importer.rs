use std::collections::HashMap;
use std::error::Error;

use super::{Agency, AgencyId, AgencyRecord};
use crate::utils::{Action, Dataset};
use crate::Line;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
        mut lines: HashMap<AgencyId, Vec<Line>>,
    ) -> Result<Vec<Agency>, Box<dyn Error>> {
        let mut agencies = Vec::new();

        let action = Action::start("Importing agencies");
        for result in action.read_csv(dataset, "agency.txt")? {
            let record: AgencyRecord = result?;
            let agency = record.import(&mut lines);
            agencies.push(agency);
        }
        action.complete(&format!("Imported {} agencies", agencies.len()));
        Ok(agencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{agencies, lines};
    use common::map;

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            agency:
                agency_id, agency_name;
                1,         "Public Transport"
        );

        let lines = map! {
            "1" => vec![lines::u4()],
        };
        let agencies = Importer::import(&mut dataset, lines).unwrap();
        assert_eq!(agencies, vec![agencies::pubtrans(vec![lines::u4()])]);
    }
}
