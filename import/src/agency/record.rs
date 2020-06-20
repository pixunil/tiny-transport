use std::collections::HashMap;

use serde_derive::Deserialize;

use super::{Agency, AgencyId};
use crate::Line;

#[derive(Debug, Deserialize)]
pub(super) struct AgencyRecord {
    agency_id: AgencyId,
    agency_name: String,
}

impl AgencyRecord {
    pub(super) fn import(self, lines: &mut HashMap<AgencyId, Vec<Line>>) -> Agency {
        let lines = lines.remove(&self.agency_id).unwrap_or_else(Vec::new);
        Agency::new(self.agency_name, lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{agencies, lines};
    use test_utils::map;

    fn agency_record() -> AgencyRecord {
        AgencyRecord {
            agency_id: "1".into(),
            agency_name: "Public Transport".to_string(),
        }
    }

    #[test]
    fn test_import_agency_without_lines() {
        let mut lines = map! {};
        assert_eq!(
            agency_record().import(&mut lines),
            agencies::pubtrans(vec![])
        );
    }

    #[test]
    fn test_import_agency_with_line() {
        let mut lines = map! {
            "1" => vec![lines::u4()],
        };
        assert_eq!(
            agency_record().import(&mut lines),
            agencies::pubtrans(vec![lines::u4()])
        );
        assert!(lines.is_empty());
    }
}
