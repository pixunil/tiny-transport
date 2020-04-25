use chrono::NaiveDate;

use crate::create_id_type;
use crate::trip::Route;
use simulation::line::Kind;
use simulation::Color;

create_id_type!(LineId);

#[derive(Debug, PartialEq)]
pub(crate) struct Line {
    name: String,
    color: Color,
    kind: Kind,
    routes: Vec<Route>,
}

impl Line {
    pub(crate) fn new(name: String, color: Color, kind: Kind, routes: Vec<Route>) -> Line {
        Line {
            name,
            color,
            kind,
            routes,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.iter()
    }

    pub(crate) fn freeze(&self, date: NaiveDate) -> (Color, serialization::Line) {
        let route = self
            .routes()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let nodes = route.freeze_nodes();
        let trains = route.freeze_trains(date);
        let color = self.color.clone();
        (
            color,
            serialization::Line::new(self.name.clone(), self.kind, nodes, trains),
        )
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    macro_rules! lines {
        ($($line:ident: $name:expr, $kind:ident, $color:expr);* $(;)?) => (
            $(
                pub(crate) fn $line() -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Color::new($color.0, $color.1, $color.2),
                        kind: Kind::$kind,
                        routes: Vec::new(),
                    }
                }
            )*
        );
    }

    lines! {
        s42:                "S42",          SuburbanRailway,    (204, 97, 18);
        u4:                 "U4",           UrbanRailway,       (255, 217, 0);
        tram_12:            "12",           Tram,               (136, 112, 171);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line::fixtures::*;

    #[test]
    fn test_getters() {
        let line = lines::u4();
        assert_eq!(line.name(), "U4");
        assert_eq!(line.kind(), Kind::UrbanRailway);
    }
}
