use std::collections::HashMap;

use simulation::Color;
use crate::agency::AgencyId;
use crate::trip::Route;
use super::{Line, Kind};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct IncompleteLine {
    agency_id: AgencyId,
    name: String,
    color: Option<Color>,
    kind: Kind,
}

impl IncompleteLine {
    pub(super) fn new(agency_id: AgencyId, name: String, kind: Kind) -> Self {
        Self {
            agency_id,
            name,
            color: None,
            kind,
        }
    }

    pub(super) fn add_color_when_applicable(&mut self, colors: &HashMap<String, Color>) {
        match self.kind {
            Kind::Railway | Kind::SuburbanRailway | Kind::UrbanRailway => {
                self.color = colors.get(&self.name).cloned();
            },
            _ => {},
        }
    }

    pub(super) fn finish(self, routes: Vec<Route>, lines: &mut HashMap<AgencyId, Vec<Line>>) {
        let line = Line::new(self.name, self.color.unwrap_or(self.kind.color()), self.kind, routes);
        lines.entry(self.agency_id)
            .or_insert_with(Vec::new)
            .push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! incomplete_line {
        ($name:expr, $kind:ident) => (
            super::IncompleteLine::new("1".into(), $name.to_string(), Kind::$kind)
        );
        (blue) => (
            incomplete_line!("Blue Line", SuburbanRailway)
        );
        (blue-replacement) => (
            incomplete_line!("Blue Line", Bus)
        );
    }

    fn colors() -> HashMap<String, Color> {
        let mut colors = HashMap::new();
        colors.insert("Blue Line".to_string(), Color::new(0, 0, 255));
        colors
    }

    #[test]
    fn test_add_color_to_applicable() {
        let mut line = incomplete_line!(blue);
        line.add_color_when_applicable(&colors());
        assert_eq!(line.color, Some(Color::new(0, 0, 255)));
    }

    #[test]
    fn test_add_color_to_unapplicable() {
        let mut line = incomplete_line!(blue-replacement);
        line.add_color_when_applicable(&colors());
        assert_eq!(line.color, None);
    }
}
