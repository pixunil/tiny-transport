use std::fmt;
use std::rc::Rc;

use crate::coord::{transform, Point, PointDebug};
use crate::location::{Linearizer, Location};
use simulation::Directions;

#[derive(PartialEq)]
pub struct Node {
    position: Point,
    kind: Kind,
}

impl Node {
    pub(crate) fn new(position: Point, location: Option<Rc<Location>>) -> Self {
        Self {
            position,
            kind: match location {
                Some(location) => Kind::Stop { location },
                None => Kind::Waypoint,
            },
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn location(&self) -> Option<&Rc<Location>> {
        match &self.kind {
            Kind::Waypoint => None,
            Kind::Stop { location } => Some(&location),
        }
    }

    pub(crate) fn store(&self, linerarizer: &mut Linearizer) -> storage::Node {
        let kind = match self.kind {
            Kind::Waypoint => storage::NodeKind::Waypoint,
            Kind::Stop { ref location } => storage::NodeKind::Stop {
                at: linerarizer.retrieve(location),
            },
        };
        let position = transform(self.position);
        storage::Node::new(position, kind, Directions::Both)
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let position = PointDebug::new(self.position, if formatter.alternate() { 6 } else { 3 });
        match &self.kind {
            Kind::Waypoint => formatter
                .debug_struct("Waypoint")
                .field("position", &position)
                .finish(),
            Kind::Stop { location } => formatter
                .debug_struct("Stop")
                .field("position", &position)
                .field("location", &location)
                .finish(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum Kind {
    Waypoint,
    Stop { location: Rc<Location> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::project;
    use crate::fixtures::{locations, paths};
    use common::assert_eq_alternate;

    #[test]
    fn test_getters() {
        let location = Rc::new(locations::friedrichstr());
        let node = Node {
            position: project(52.520, 13.388),
            kind: Kind::Stop {
                location: Rc::clone(&location),
            },
        };
        assert_eq!(node.position(), project(52.520, 13.388));
        assert_eq!(node.location(), Some(&location));
    }

    #[test]
    #[ignore]
    fn test_store() {
        let (segments, segment_ids) = paths::tram_12::segments();
        let mut linearizer = Linearizer::new();
        assert_eq_alternate!(
            paths::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids)
                .nodes(&segments)
                .map(|node| node.store(&mut linearizer))
                .collect::<Vec<_>>(),
            storage::fixtures::nodes::tram_12(&linearizer.location_ids())
        );
    }
}
