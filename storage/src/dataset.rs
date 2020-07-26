use std::iter::repeat_with;
use std::rc::Rc;

use serde_derive::{Deserialize, Serialize};

use crate::line::Line;
use crate::schedule::Schedule;
use crate::station::Station;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    stations: Vec<Station>,
    schedules: Vec<Schedule>,
    lines: Vec<Line>,
}

impl Dataset {
    pub fn new(stations: Vec<Station>, schedules: Vec<Schedule>, lines: Vec<Line>) -> Self {
        Self {
            stations,
            schedules,
            lines,
        }
    }

    pub fn load(self) -> simulation::Dataset {
        let mut station_infos = repeat_with(Vec::new).take(self.stations.len()).collect();
        for line in &self.lines {
            line.add_to_station_infos(&mut station_infos);
        }
        let station_kinds = station_infos
            .into_iter()
            .map(|line_kinds| simulation::station::Kind::from_line_kinds(&line_kinds));
        let stations = self
            .stations
            .into_iter()
            .zip(station_kinds)
            .map(|(station, kind)| Rc::new(station.load(kind)))
            .collect::<Vec<_>>();
        let schedules = &self.schedules;
        let lines = self
            .lines
            .into_iter()
            .map(|line| line.load(&stations, schedules))
            .collect();
        simulation::Dataset::new(stations, lines)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::lines;
    use crate::fixtures_with_ids;

    macro_rules! datasets {
        ( $( $dataset:ident => {
                stations: [ $($station:ident),* $(,)? ],
                schedules: [ $($schedule:ident),* $(,)? ],
                lines: [ $($line:ident),* $(,)? ],
            } ),* $(,)? ) => (
            $(
                pub fn $dataset() -> Dataset {
                    let (stations, station_ids) = fixtures_with_ids!(stations::{$($station),*});
                    let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{$($schedule),*});
                    Dataset {
                        stations,
                        schedules,
                        lines: vec![ $( lines::$line(&station_ids, &schedule_ids) ),* ],
                    }
                }
            )*
        );
    }

    datasets! {
        hauptbahnhof_friedrichstr => {
            stations: [
                hauptbahnhof, friedrichstr, hackescher_markt, bellevue,
                naturkundemuseum, franzoesische_str, oranienburger_tor,
                universitaetsstr, am_kupfergraben, georgenstr_am_kupfergraben,
            ],
            schedules: [
                hackescher_markt_bellevue, bellevue_hackescher_markt,
                naturkundemuseum_franzoesische_str, franzoesische_str_naturkundemuseum,
                oranienburger_tor_am_kupfergraben, am_kupfergraben_oranienburger_tor,
            ],
            lines: [u6, s3, tram_12],
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::datasets;

    #[test]
    fn test_load() {
        let dataset = datasets::hauptbahnhof_friedrichstr();
        assert_eq!(
            dataset.load(),
            simulation::fixtures::datasets::hauptbahnhof_friedrichstr()
        );
    }
}
