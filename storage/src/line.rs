use std::rc::Rc;

use serde_derive::{Deserialize, Serialize};

use crate::node::Node;
use crate::train::Train;
use simulation::line::Kind;
use simulation::Color;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    name: String,
    color: Color,
    kind: Kind,
    nodes: Vec<Node>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(
        name: String,
        color: Color,
        kind: Kind,
        nodes: Vec<Node>,
        trains: Vec<Train>,
    ) -> Line {
        Line {
            name,
            color,
            kind,
            nodes,
            trains,
        }
    }

    pub fn load(self, stations: &[Rc<simulation::Station>]) -> simulation::Line {
        let kind = self.kind;
        let nodes = self
            .nodes
            .into_iter()
            .map(|node| node.load(&stations))
            .collect::<Vec<_>>();
        let trains = self
            .trains
            .into_iter()
            .map(|train| train.load(kind, &nodes))
            .collect();

        simulation::Line::new(self.name, self.color, kind, nodes, trains)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::ops::Index;

    use super::*;
    use crate::fixtures::{nodes, trains};

    macro_rules! lines {
        (trains $line:ident, $route:ident, [$( $hour:expr, $minute:expr );* $(;)?]) => {
            $( trains::$line::$route($hour, $minute) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $upstream:ident, $upstream_times:tt, $downstream:ident, $downstream_times:tt);* $(;)?) => {
            $(
                pub fn $line<'a>(station_ids: &impl Index<&'a str, Output = usize>) -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Kind::$kind.color(),
                        kind: Kind::$kind,
                        nodes: nodes::$line(station_ids),
                        trains: vec![
                            lines!(trains $line, $upstream, $upstream_times),
                            lines!(trains $line, $downstream, $downstream_times),
                        ],
                    }
                }
            )*
        };
    }

    lines! {
        tram_12:            "12",           Tram,
            oranienburger_tor_am_kupfergraben, [9, 2.0],
            am_kupfergraben_oranienburger_tor, [8, 34.0];
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::lines;
    use crate::stations_with_ids;

    #[test]
    fn test_load() {
        let (stations, station_ids) = stations_with_ids![
            oranienburger_tor,
            friedrichstr,
            universitaetsstr,
            am_kupfergraben,
            georgenstr_am_kupfergraben,
        ];
        let line = lines::tram_12(&station_ids);
        assert_eq!(line.load(&stations), simulation::fixtures::lines::tram_12());
    }
}
