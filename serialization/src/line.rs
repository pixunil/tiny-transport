use serde_derive::{Serialize, Deserialize};

use simulation::Color;
use simulation::LineNode;
use super::train::Train;

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    name: String,
    nodes: Vec<LineNode>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, nodes: Vec<LineNode>, trains: Vec<Train>) -> Line {
        Line {
            name,
            nodes,
            trains,
        }
    }

    fn unfreeze(self) -> simulation::Line {
        let nodes = self.nodes;
        let trains = self.trains.into_iter()
            .map(|train| train.unfreeze(&nodes))
            .collect();

        simulation::Line::new(self.name, nodes, trains)
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

    pub fn unfreeze(self) -> simulation::LineGroup {
        let lines = self.lines.into_iter()
            .map(|line| line.unfreeze())
            .collect();
        simulation::LineGroup::new(self.color, lines)
    }
}
