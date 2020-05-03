use serde_derive::{Deserialize, Serialize};

use super::train::Train;
use simulation::line::Kind;
use simulation::{Color, Node};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    name: String,
    kind: Kind,
    nodes: Vec<Node>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, kind: Kind, nodes: Vec<Node>, trains: Vec<Train>) -> Line {
        Line {
            name,
            kind,
            nodes,
            trains,
        }
    }

    fn load(self) -> simulation::Line {
        let kind = self.kind;
        let nodes = self.nodes;
        let trains = self
            .trains
            .into_iter()
            .map(|train| train.load(kind, &nodes))
            .collect();

        simulation::Line::new(self.name, kind, nodes, trains)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineGroup {
    color: Color,
    lines: Vec<Line>,
}

impl LineGroup {
    pub fn new(color: Color, lines: Vec<Line>) -> LineGroup {
        LineGroup { color, lines }
    }

    pub fn load(self) -> simulation::LineGroup {
        let lines = self.lines.into_iter().map(|line| line.load()).collect();
        simulation::LineGroup::new(self.color, lines)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::*;
    use simulation::fixtures::nodes;

    macro_rules! lines {
        (trains $line:ident, $route:ident, [$( $hour:expr, $minute:expr );* $(;)?]) => {
            $( trains::$line::$route($hour, $minute) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $upstream:ident, $upstream_times:tt, $downstream:ident, $downstream_times:tt);* $(;)?) => {
            $(
                pub fn $line() -> Line {
                    Line {
                        name: $name.to_string(),
                        kind: Kind::$kind,
                        nodes: nodes::$line(),
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
