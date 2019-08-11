use std::error::Error;
use std::collections::HashMap;

use super::utils::*;
use super::line::Line;

#[derive(Debug, PartialEq)]
pub struct Agency {
    pub name: String,
    pub lines: Vec<Line>,
}

impl Agency {
    fn new(record: AgencyRecord, lines: &mut HashMap<Id, Vec<Line>>) -> Agency {
        let lines = lines.remove(&record.agency_id)
            .unwrap_or_else(Vec::new);
        Agency {
            name: record.agency_name,
            lines,
        }
    }
}

pub fn from_csv(dataset: &mut impl Dataset, mut lines: HashMap<Id, Vec<Line>>) -> Result<Vec<Agency>, Box<dyn Error>> {
    let mut agencies = Vec::new();
    let mut reader = dataset.read_csv("agency.txt")?;
    for result in reader.deserialize() {
        let agency = Agency::new(result?, &mut lines);
        agencies.push(agency);
    }

    Ok(agencies)
}

#[derive(Debug, Deserialize)]
struct AgencyRecord {
    agency_id: Id,
    agency_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::line::tests::blue_line;

    fn agency_record() -> AgencyRecord {
        AgencyRecord {
            agency_id: "1".into(),
            agency_name: "Public Transport".into(),
        }
    }

    fn agency(lines: Vec<Line>) -> Agency {
        Agency {
            name: "Public Transport".into(),
            lines: lines,
        }
    }

    fn lines() -> HashMap<Id, Vec<Line>> {
        let mut lines = HashMap::new();
        lines.insert("1".into(), vec![blue_line()]);
        lines
    }

    #[test]
    fn test_import_agency_without_lines() {
        let mut lines = HashMap::new();
        assert_eq!(Agency::new(agency_record(), &mut lines), agency(vec![]));
    }

    #[test]
    fn test_import_agency_with_line() {
        let mut lines = lines();
        assert_eq!(Agency::new(agency_record(), &mut lines), agency(vec![blue_line()]));
        assert!(lines.is_empty());
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            agency:
                agency_id, agency_name;
                1,         "Public Transport"
        );

        let agencies = from_csv(&mut dataset, lines()).unwrap();
        assert_eq!(agencies, vec![agency(vec![blue_line()])]);
    }
}
