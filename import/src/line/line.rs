use chrono::NaiveDate;

use crate::create_id_type;
use crate::location::Linearizer;
use crate::trip::{Route, Scheduler};
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

    pub(crate) fn store(
        &self,
        date: NaiveDate,
        linearizer: &mut Linearizer,
        scheduler: &mut Scheduler,
    ) -> storage::Line {
        let route = self
            .routes()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let nodes = route.store_nodes(linearizer);
        let trains = route.store_trains(date, scheduler);
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
    use crate::fixtures::routes;

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
        s1:                 "S1",           SuburbanRailway,    (220, 107, 166);
        s42:                "S42",          SuburbanRailway,    (204,  97,  18);
        u4:                 "U4",           UrbanRailway,       (255, 217,   0);
        tram_12:            "12",           Tram,               (136, 112, 171);
    }

    pub(crate) fn tram_12_with_route() -> Line {
        let mut line = tram_12();
        line.color = Kind::Tram.color();
        line.routes
            .push(routes::tram_12::oranienburger_tor_am_kupfergraben());
        line
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::fixtures::lines;
    use test_utils::map;

    #[test]
    fn test_getters() {
        let line = lines::u4();
        assert_eq!(line.name(), "U4");
        assert_eq!(line.kind(), Kind::UrbanRailway);
    }

    #[test]
    fn test_store() {
        let line = lines::tram_12_with_route();
        let date = NaiveDate::from_ymd(2019, 1, 1);
        let mut linearizer = Linearizer::new();
        let mut scheduler = Scheduler::new();
        let schedule_ids: HashMap<&str, usize> = map! {
            "oranienburger_tor_am_kupfergraben" => 0,
            "am_kupfergraben_oranienburger_tor" => 1,
        };
        assert_eq!(
            line.store(date, &mut linearizer, &mut scheduler),
            storage::fixtures::lines::tram_12(&linearizer.location_ids(), &schedule_ids)
        );
    }
}
