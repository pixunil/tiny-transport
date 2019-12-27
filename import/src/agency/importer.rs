use std::error::Error;
use std::collections::HashMap;

use crate::utils::Dataset;
use crate::Line;
use super::{Agency, AgencyId, AgencyRecord};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset, mut lines: HashMap<AgencyId, Vec<Line>>) -> Result<Vec<Agency>, Box<dyn Error>> {
        let mut agencies = Vec::new();
        let mut reader = dataset.read_csv("agency.txt")?;
        for result in reader.deserialize() {
            let record: AgencyRecord = result?;
            let agency = record.import(&mut lines);
            agencies.push(agency);
        }

        Ok(agencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::line_;

    #[macro_export]
    macro_rules! agency {
        ($name:literal, [$($line:ident),*]) => (
            Agency::new($name.to_string(), vec![$(line_!($line)),*])
        );
        (pubtrans, [$($line:ident),*]) => (
            agency!("Public Transport", [$($line),*])
        );
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            agency:
                agency_id, agency_name;
                1,         "Public Transport"
        );

        let mut lines = HashMap::new();
        lines.insert("1".into(), vec![line_!(blue)]);
        let agencies = Importer::import(&mut dataset, lines).unwrap();
        assert_eq!(agencies, vec![agency!(pubtrans, [blue])]);
    }
}
