use std::collections::HashMap;

use serde_derive::Deserialize;

use crate::Line;
use super::{Agency, AgencyId};

#[derive(Debug, Deserialize)]
pub(super) struct AgencyRecord {
    agency_id: AgencyId,
    agency_name: String,
}

impl AgencyRecord {
    pub(super) fn import(self, lines: &mut HashMap<AgencyId, Vec<Line>>) -> Agency {
        let lines = lines.remove(&self.agency_id)
            .unwrap_or_else(Vec::new);
        Agency::new(self.agency_name, lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::map;
    use crate::agency::fixtures::*;

    fn agency_record() -> AgencyRecord {
        AgencyRecord {
            agency_id: "1".into(),
            agency_name: "Public Transport".to_string(),
        }
    }

    #[test]
    fn test_import_agency_without_lines() {
        let mut lines = map! {};
        assert_eq!(agency_record().import(&mut lines), agencies::pubtrans(vec![]));
    }

    #[test]
    fn test_import_agency_with_line() {
        let mut lines = map! {
            "1" => vec![lines::blue()],
        };
        assert_eq!(agency_record().import(&mut lines), agencies::pubtrans(vec![lines::blue()]));
        assert!(lines.is_empty());
    }
}
