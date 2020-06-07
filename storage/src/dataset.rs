use std::rc::Rc;

use serde_derive::{Deserialize, Serialize};

use crate::line::Line;
use crate::station::Station;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    stations: Vec<Station>,
    lines: Vec<Line>,
}

impl Dataset {
    pub fn new(stations: Vec<Station>, lines: Vec<Line>) -> Self {
        Self { stations, lines }
    }

    pub fn load(self) -> simulation::Dataset {
        let stations = self
            .stations
            .into_iter()
            .map(Station::load)
            .map(|station| Rc::new(station))
            .collect::<Vec<_>>();
        let lines = self
            .lines
            .into_iter()
            .map(|line| line.load(&stations))
            .collect();
        simulation::Dataset::new(stations, lines)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::collections::HashMap;

    use super::*;
    use crate::fixtures::{lines, stations};

    macro_rules! datasets {
        ( $( $dataset:ident => {
                stations: [ $($station:ident),* $(,)? ],
                lines: [ $($line:ident),* $(,)? ],
            } ),* $(,)? ) => (
            $(
                pub fn $dataset() -> Dataset {
                    let station_ids = vec![ $( stringify!($station)),* ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, identifier)| (identifier, i))
                        .collect::<HashMap<_, _>>();
                    Dataset {
                        stations: vec![ $( stations::$station() ),* ],
                        lines: vec![ $( lines::$line(&station_ids) ),* ],
                    }
                }
            )*
        );
    }

    datasets! {
        tram_12 => {
            stations: [
                oranienburger_tor, friedrichstr, universitaetsstr, am_kupfergraben,
                georgenstr_am_kupfergraben,
            ],
            lines: [tram_12],
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::datasets;

    #[test]
    fn test_load() {
        let dataset = datasets::tram_12();
        assert_eq!(dataset.load(), simulation::fixtures::datasets::tram_12());
    }
}
