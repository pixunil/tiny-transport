use chrono::NaiveDate;

use crate::create_id_type;
use crate::path::{IndexMap, Segment};
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
        segments: &[Segment],
        index_map: &mut IndexMap,
        scheduler: &mut Scheduler,
    ) -> storage::Line {
        let route = self
            .routes()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let path = route.path().store(index_map);
        let trains = route.store_trains(date, segments, scheduler);
        storage::Line::new(
            self.name.clone(),
            self.color.clone(),
            self.kind,
            path,
            trains,
        )
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::ops::Index;

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

    pub(crate) fn tram_12_with_route<'a>(segments: &impl Index<&'a str, Output = usize>) -> Line {
        let mut line = tram_12();
        line.color = Kind::Tram.color();
        line.routes
            .push(routes::tram_12::oranienburger_tor_am_kupfergraben(segments));
        line
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::fixtures::{lines, paths};
    use common::map;

    #[test]
    fn test_getters() {
        let line = lines::u4();
        assert_eq!(line.name(), "U4");
        assert_eq!(line.kind(), Kind::UrbanRailway);
    }

    #[test]
    fn test_store() {
        let (segments, segment_ids) = paths::tram_12::segments();
        let line = lines::tram_12_with_route(&segment_ids);
        let date = NaiveDate::from_ymd(2019, 1, 1);
        let mut index_map = IndexMap::new();
        let mut scheduler = Scheduler::new();
        let segment_ids: HashMap<&str, usize> = map! {
            "oranienburger_tor_friedrichstr" => 0,
            "universitaetsstr_am_kupfergraben" => 1,
        };
        let schedule_ids: HashMap<&str, usize> = map! {
            "oranienburger_tor_am_kupfergraben" => 0,
            "am_kupfergraben_oranienburger_tor" => 1,
        };
        assert_eq!(
            line.store(date, &segments, &mut index_map, &mut scheduler),
            storage::fixtures::lines::tram_12(&segment_ids, &schedule_ids)
        );
    }
}
