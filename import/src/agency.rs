use std::error::Error;
use std::collections::HashMap;
use std::path::PathBuf;

use super::utils::*;
use super::line::Line;

#[derive(Debug)]
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

pub fn from_csv(path: &mut PathBuf, mut lines: HashMap<Id, Vec<Line>>) -> Result<Vec<Agency>, Box<Error>> {
    let mut agencies = Vec::new();

    path.set_file_name("agency.txt");
    let mut reader = csv::Reader::from_path(&path)?;
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