use std::rc::Rc;

use na::Point2;

use crate::station::Station;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    position: Point2<f32>,
    kind: Kind,
}

impl Node {
    pub fn new(position: Point2<f32>, kind: Kind) -> Node {
        Node { position, kind }
    }

    pub fn position(&self) -> Point2<f32> {
        self.position
    }

    pub fn is_stop(&self) -> bool {
        match self.kind {
            Kind::Waypoint => false,
            Kind::Stop { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Waypoint,
    Stop { at: Rc<Station> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::stations;

    #[test]
    fn test_getters() {
        let node = Node {
            position: Point2::new(-111.0, -1115.0),
            kind: Kind::Stop {
                at: Rc::new(stations::friedrichstr()),
            },
        };
        assert_eq!(node.position(), Point2::new(-111.0, -1115.0));
        assert!(node.is_stop());
    }
}
