use std::rc::Rc;

use na::Point2;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    position: Point2<f32>,
    kind: Kind,
}

impl Node {
    pub fn new(position: Point2<f32>, kind: Kind) -> Self {
        Self { position, kind }
    }

    pub(crate) fn station(&self) -> Option<usize> {
        match self.kind {
            Kind::Waypoint => None,
            Kind::Stop { at } => Some(at),
        }
    }

    pub fn load(self, stations: &[Rc<simulation::Station>]) -> simulation::path::Node {
        simulation::path::Node::new(self.position, self.kind.load(stations))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    Waypoint,
    Stop { at: usize },
}

impl Kind {
    fn load(self, stations: &[Rc<simulation::Station>]) -> simulation::path::NodeKind {
        match self {
            Self::Waypoint => simulation::path::NodeKind::Waypoint,
            Self::Stop { at } => simulation::path::NodeKind::Stop {
                at: stations[at].clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let node = Node {
            position: Point2::new(-111.0, -1115.0),
            kind: Kind::Stop { at: 1 },
        };
        let stations = vec![
            Rc::new(simulation::fixtures::stations::oranienburger_tor()),
            Rc::new(simulation::fixtures::stations::friedrichstr()),
        ];

        let expected = simulation::path::Node::new(
            Point2::new(-111.0, -1115.0),
            simulation::path::NodeKind::Stop {
                at: Rc::clone(&stations[1]),
            },
        );
        assert_eq!(node.load(&stations), expected);
    }
}
