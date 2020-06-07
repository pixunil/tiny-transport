use chrono::NaiveDate;

use crate::create_id_type;
use crate::location::Linearizer;
use crate::trip::Route;
use simulation::line::Kind;
use simulation::Color;

create_id_type!(LineId);

#[derive(Debug, PartialEq)]
pub struct Line {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }

    pub fn routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.iter()
    }

    pub(crate) fn store(&self, date: NaiveDate, linearizer: &mut Linearizer) -> storage::Line {
        let route = self
            .routes()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let nodes = route.store_nodes(linearizer);
        let trains = route.store_trains(date);
        storage::Line::new(
            self.name.clone(),
            self.color.clone(),
            self.kind,
            nodes,
            trains,
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

    #[test]
    fn test_store() {
        let mut line = lines::tram_12();
        line.color = Kind::Tram.color();
        line.routes = vec![routes::tram_12::oranienburger_tor_am_kupfergraben()];
        let date = NaiveDate::from_ymd(2019, 1, 1);
        let mut linearizer = Linearizer::new();
        assert_eq!(
            line.store(date, &mut linearizer),
            storage::fixtures::lines::tram_12(&linearizer.location_ids())
        );
    }
}
