use std::error::Error;
use std::collections::HashMap;

use super::utils::*;
use super::line::Line;

#[derive(Debug, PartialEq)]
pub(crate) struct Agency {
    name: String,
    lines: Vec<Line>,
}

impl Agency {
    fn new(name: String, lines: Vec<Line>) -> Agency {
        Agency { name, lines }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn lines(&self) -> &[Line] {
        &self.lines
    }
}

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset, mut lines: HashMap<Id, Vec<Line>>) -> Result<Vec<Agency>, Box<dyn Error>> {
        let mut agencies = Vec::new();
        let mut reader = dataset.read_csv("agency.txt")?;
        for result in reader.deserialize() {
            let agency = Self::from_record(result?, &mut lines);
            agencies.push(agency);
        }

        Ok(agencies)
    }

    fn from_record(record: AgencyRecord, lines: &mut HashMap<Id, Vec<Line>>) -> Agency {
        let lines = lines.remove(&record.agency_id)
            .unwrap_or_else(Vec::new);
        Agency::new(record.agency_name, lines)
    }
}


#[derive(Debug, Deserialize)]
struct AgencyRecord {
    agency_id: Id,
    agency_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::line_;

    macro_rules! agency {
        ($name:literal, [$($line:ident),*]) => (
            Agency::new($name.to_string(), vec![$(line_!($line)),*])
        );
        (pubtrans, [$($line:ident),*]) => (
            agency!("Public Transport", [$($line),*])
        );
    }

    fn agency_record() -> AgencyRecord {
        AgencyRecord {
            agency_id: "1".to_string(),
            agency_name: "Public Transport".to_string(),
        }
    }

    fn lines() -> HashMap<Id, Vec<Line>> {
        let mut lines = HashMap::new();
        lines.insert("1".to_string(), vec![line_!(blue)]);
        lines
    }

    #[test]
    fn test_import_agency_without_lines() {
        let mut lines = HashMap::new();
        assert_eq!(Importer::from_record(agency_record(), &mut lines), agency!(pubtrans, []));
    }

    #[test]
    fn test_import_agency_with_line() {
        let mut lines = lines();
        assert_eq!(Importer::from_record(agency_record(), &mut lines), agency!(pubtrans, [blue]));
        assert!(lines.is_empty());
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            agency:
                agency_id, agency_name;
                1,         "Public Transport"
        );

        let agencies = Importer::import(&mut dataset, lines()).unwrap();
        assert_eq!(agencies, vec![agency!(pubtrans, [blue])]);
    }
}
