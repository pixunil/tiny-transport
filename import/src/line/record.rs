use std::collections::HashMap;

use itertools::Itertools;

use serde_derive::Deserialize;

use super::{IncompleteLine, LineId};
use crate::agency::AgencyId;
use crate::deserialize;
use simulation::line::Kind;
use simulation::Color;

#[derive(Debug, Deserialize)]
pub(super) struct LineRecord {
    #[serde(rename = "route_id")]
    line_id: LineId,
    agency_id: AgencyId,
    route_short_name: String,
    #[serde(rename = "route_type", deserialize_with = "deserialize::line_kind")]
    line_kind: Kind,
}

impl LineRecord {
    pub(super) fn deduplicate(
        self,
        id_mapping: &mut HashMap<LineId, usize>,
        incomplete_lines: &mut Vec<IncompleteLine>,
    ) {
        let incomplete_line =
            IncompleteLine::new(self.agency_id, self.route_short_name, self.line_kind);
        let position = match incomplete_lines
            .iter()
            .find_position(|other| &&incomplete_line == other)
        {
            Some((position, _)) => position,
            None => {
                incomplete_lines.push(incomplete_line);
                incomplete_lines.len() - 1
            }
        };
        id_mapping.insert(self.line_id, position);
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct LineColorRecord {
    line: String,
    #[serde(deserialize_with = "deserialize::color")]
    color: Color,
}

impl LineColorRecord {
    pub(super) fn import(self, colors: &mut HashMap<String, Color>) {
        colors.insert(self.line, self.color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::incomplete_lines;
    use test_utils::map;

    fn u4_line_record() -> LineRecord {
        LineRecord {
            line_id: "1".into(),
            agency_id: "pubtransport".into(),
            route_short_name: "U4".to_string(),
            line_kind: Kind::UrbanRailway,
        }
    }

    #[test]
    fn test_import() {
        let mut id_mapping = HashMap::new();
        let mut incomplete_lines = Vec::new();
        u4_line_record().deduplicate(&mut id_mapping, &mut incomplete_lines);
        assert_eq!(
            id_mapping,
            map! {
                "1" => 0,
            }
        );
        assert_eq!(incomplete_lines, [incomplete_lines::u4()]);
    }

    #[test]
    fn test_deduplicate() {
        let mut id_mapping = HashMap::new();
        let mut incomplete_lines = Vec::new();

        let mut duplicated = u4_line_record();
        duplicated.line_id = "2".into();
        u4_line_record().deduplicate(&mut id_mapping, &mut incomplete_lines);
        duplicated.deduplicate(&mut id_mapping, &mut incomplete_lines);
        assert_eq!(
            id_mapping,
            map! {
                "1" => 0,
                "2" => 0,
            }
        );
        assert_eq!(incomplete_lines, [incomplete_lines::u4()]);
    }
}
