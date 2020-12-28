use std::rc::Rc;

use serde_derive::{Deserialize, Serialize};

use crate::path::{Segment, SegmentedPath};
use crate::schedule::Schedule;
use crate::train::Train;
use simulation::line::Kind;
use simulation::Color;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    name: String,
    color: Color,
    kind: Kind,
    path: SegmentedPath,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(
        name: String,
        color: Color,
        kind: Kind,
        path: SegmentedPath,
        trains: Vec<Train>,
    ) -> Line {
        Line {
            name,
            color,
            kind,
            path,
            trains,
        }
    }

    pub(crate) fn add_to_station_infos(
        &self,
        segments: &[Segment],
        station_infos: &mut Vec<Vec<Kind>>,
    ) {
        let station_ids = self.path.nodes(segments).filter_map(|node| node.station());
        for station_id in station_ids {
            let station_info = &mut station_infos[station_id];
            if !station_info.contains(&self.kind) {
                station_info.push(self.kind);
            }
        }
    }

    pub fn load(
        self,
        stations: &[Rc<simulation::Station>],
        segments: &[Segment],
        schedules: &[Schedule],
    ) -> simulation::Line {
        let kind = self.kind;
        let nodes = self
            .path
            .nodes(segments)
            .map(|node| node.load(&stations))
            .collect::<Vec<_>>();
        let trains = self
            .trains
            .into_iter()
            .map(|train| train.load(kind, &nodes, schedules))
            .collect();

        simulation::Line::new(self.name, self.color, kind, nodes, trains)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::ops::Index;

    use super::*;
    use crate::fixtures::{paths, trains};
    use common::time;

    macro_rules! lines {
        (@trains $line:ident, $route:ident, [$( $( $(:)? $time:literal )* ),* $(,)?], $schedule_ids:expr) => {
            $( trains::$line::$route(time!($($time),*), $schedule_ids) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $route:ident, $times:tt);* $(;)?) => {
            $(
                pub fn $line<'a>(
                    segment_ids: &impl Index<&'a str, Output = usize>,
                    schedule_ids: &impl Index<&'a str, Output = usize>,
                ) -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Kind::$kind.color(),
                        kind: Kind::$kind,
                        path: paths::$line::$route(segment_ids),
                        trains: vec![
                            lines!(@trains $line, $route, $times, schedule_ids),
                        ],
                    }
                }
            )*
        };
    }

    lines! {
        s3:                 "S3",           SuburbanRailway,
            hackescher_markt_bellevue, [7:24:54];
        u6:                 "U6",           UrbanRailway,
            naturkundemuseum_franzoesische_str, [5:55:40];
        tram_12:            "12",           Tram,
            oranienburger_tor_am_kupfergraben, [9:01:40];
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::fixtures::lines;
    use common::{fixtures_with_ids, map};

    #[test]
    fn test_add_tram_to_station_infos() {
        let station_ids: HashMap<&str, usize> = map! {
            "oranienburger_tor" => 1,
            "friedrichstr" => 2,
            "universitaetsstr" => 3,
            "am_kupfergraben" => 4,
        };
        let (segments, segment_ids) = fixtures_with_ids!(segments::{
            oranienburger_tor_friedrichstr,
            universitaetsstr_am_kupfergraben,
        }, (&station_ids));
        let schedule_ids: HashMap<&str, usize> = map! {
            "oranienburger_tor_am_kupfergraben" => 0,
            "am_kupfergraben_oranienburger_tor" => 1,
        };
        let mut station_infos = vec![Vec::new(); 6];
        let line = lines::tram_12(&segment_ids, &schedule_ids);
        line.add_to_station_infos(&segments, &mut station_infos);
        let station_ids = station_ids.values().copied().collect::<Vec<_>>();
        for (station_id, station_info) in station_infos.into_iter().enumerate() {
            let expect_contained = station_ids.contains(&station_id);
            assert_eq!(station_info.contains(&Kind::Tram), expect_contained);
        }
    }

    #[test]
    #[ignore]
    fn test_load() {
        let (stations, station_ids) = fixtures_with_ids!(simulation::stations::{
            oranienburger_tor,
            friedrichstr,
            universitaetsstr,
            am_kupfergraben,
            georgenstr_am_kupfergraben,
        } with Rc);
        let (segments, segment_ids) = fixtures_with_ids!(segments::{
            oranienburger_tor_friedrichstr,
            universitaetsstr_am_kupfergraben,
            am_kupfergraben_georgenstr,
        }, (&station_ids));
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
           oranienburger_tor_am_kupfergraben,
           am_kupfergraben_oranienburger_tor,
        });
        let line = lines::tram_12(&segment_ids, &schedule_ids);
        assert_eq!(
            line.load(&stations, &segments, &schedules),
            simulation::fixtures::lines::tram_12()
        );
    }
}
