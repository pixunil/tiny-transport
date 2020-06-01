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
        let stations = self.stations.into_iter().map(Station::load).collect();
        let lines = self.lines.into_iter().map(Line::load).collect();
        simulation::Dataset::new(stations, lines)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::{lines, stations};

    pub fn tram_12() -> Dataset {
        Dataset {
            stations: vec![
                stations::oranienburger_tor(),
                stations::friedrichstr(),
                stations::universitaetsstr(),
                stations::am_kupfergraben(),
                stations::georgenstr_am_kupfergraben(),
            ],
            lines: vec![lines::tram_12()],
        }
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
