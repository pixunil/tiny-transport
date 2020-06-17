use std::fmt;
use std::rc::Rc;

use itertools::Itertools;
use ordered_float::NotNan;

use crate::coord::{debug_position, transform, Point};
use crate::location::{Linearizer, Location};
use simulation::{Direction, Directions};

#[derive(PartialEq)]
pub struct Node {
    position: Point,
    kind: Kind,
    in_directions: Directions,
}

impl Node {
    pub(super) fn new(position: Point, in_directions: Directions) -> Self {
        Self {
            position,
            kind: Kind::Waypoint,
            in_directions,
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub(super) fn distance_to(&self, location: &Location) -> NotNan<f64> {
        let distance = na::distance(&self.position, &location.position());
        NotNan::new(distance).unwrap()
    }

    pub fn location(&self) -> Option<&Rc<Location>> {
        match &self.kind {
            Kind::Waypoint => None,
            Kind::Stop { location } => Some(&location),
        }
    }

    pub fn in_directions(&self) -> Directions {
        self.in_directions
    }

    pub(super) fn segment_weights(nodes: &[Self], direction: Direction) -> Vec<f64> {
        let mut weights = nodes
            .iter()
            .filter(|node| node.in_directions.allows(direction))
            .filter(|node| node.location().is_some())
            .tuple_windows()
            .map(|(before, after)| na::distance(&before.position, &after.position))
            .collect::<Vec<_>>();
        if direction == Direction::Downstream {
            weights.reverse();
        }
        weights
    }

    pub(super) fn make_stop(&mut self, location: Rc<Location>) {
        self.kind = Kind::Stop { location };
    }

    pub(super) fn can_be_merged(&self, other: &Self) -> bool {
        self.position == other.position
            && self.kind == other.kind
            && self.in_directions == Directions::UpstreamOnly
            && other.in_directions == Directions::DownstreamOnly
    }

    pub(super) fn merge(&mut self, other: Self) {
        assert!(self.can_be_merged(&other));
        self.in_directions = Directions::Both;
    }

    pub(super) fn store(&self, linerarizer: &mut Linearizer) -> storage::Node {
        let kind = match self.kind {
            Kind::Waypoint => storage::NodeKind::Waypoint,
            Kind::Stop { ref location } => storage::NodeKind::Stop {
                at: linerarizer.retrieve(location),
            },
        };
        let position = transform(self.position);
        storage::Node::new(position, kind, self.in_directions)
    }
}

#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let position = debug_position(self.position, formatter.alternate());
        match &self.kind {
            Kind::Waypoint => formatter
                .debug_struct("Waypoint")
                .field("position", &position)
                .field("in_directions", &self.in_directions)
                .finish(),
            Kind::Stop { location } => formatter
                .debug_struct("Stop")
                .field("position", &position)
                .field("location", &location)
                .field("in_directions", &self.in_directions)
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
pub(super) mod fixtures {
    use super::*;
    use crate::coord::project;
    use crate::location::fixtures::*;

    macro_rules! nodes {
        (kind $location:ident) => ( Kind::Stop { location: Rc::new(locations::$location()) });
        (kind) => ( Kind::Waypoint );
        (node Both, $position:expr, $kind:expr, $in_directions:ident) => (
            Node {
                position: $position,
                kind: $kind,
                in_directions: Directions::$in_directions,
            }
        );
        (node UpstreamOnly, $position:expr, $kind:expr, DownstreamOnly) => ( None );
        (node DownstreamOnly, $position:expr, $kind:expr, UpstreamOnly) => ( None );
        (node $single_direction:ident, $position:expr, $kind:expr, $in_directions:ident) => (
            Some(Node {
                position: $position,
                kind: $kind,
                in_directions: Directions::$single_direction,
            })
        );
        ($($line:ident: $($lat:literal, $lon:literal, $in_directions:ident $(, $location:ident)?);* $(;)?)*) => (
            $(
                pub(in crate::trip) fn $line(directions: Directions) -> Vec<Node> {
                    match directions {
                        Directions::Both => vec![$(
                            nodes!(node Both, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*],
                        Directions::UpstreamOnly => vec![$(
                            nodes!(node UpstreamOnly, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*].into_iter().filter_map(|node| node).collect(),
                        Directions::DownstreamOnly => vec![$(
                            nodes!(node DownstreamOnly, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*].into_iter().filter_map(|node| node).collect(),
                    }
                }
            )*
        );
    }

    nodes! {
        circle:
            52.549, 13.388, Both,           gesundbrunnen;
            52.503, 13.469, Both,           ostkreuz;
            52.475, 13.366, Both,           suedkreuz;
            52.501, 13.283, Both,           westkreuz;
            52.549, 13.388, Both,           gesundbrunnen;
        s3:
            52.523, 13.402, Both,           hackescher_markt;
            52.521, 13.386, Both,           friedrichstr;
            52.525, 13.369, Both,           hauptbahnhof;
            52.520, 13.347, Both,           bellevue;
        tram_12:
            52.525, 13.388, Both,           oranienburger_tor;
            52.524, 13.388, Both;
            52.522, 13.388, DownstreamOnly;
            52.521, 13.388, Both;
            52.520, 13.388, Both,           friedrichstr;
            52.519, 13.388, UpstreamOnly;
            52.519, 13.389, Both;
            52.519, 13.390, Both;
            52.519, 13.391, UpstreamOnly;
            52.519, 13.392, UpstreamOnly,   universitaetsstr;
            52.520, 13.390, DownstreamOnly;
            52.520, 13.391, DownstreamOnly;
            52.520, 13.393, DownstreamOnly, georgenstr_am_kupfergraben;
            52.521, 13.394, DownstreamOnly;
            52.521, 13.395, DownstreamOnly;
            52.520, 13.396, DownstreamOnly;
            52.519, 13.396, Both,           am_kupfergraben;
        bus_m41:
            52.505, 13.382, Both,           anhalter_bahnhof;
            52.506, 13.380, Both;
            52.507, 13.380, UpstreamOnly,   abgeordnetenhaus;
            52.507, 13.379, UpstreamOnly;
            52.507, 13.379, DownstreamOnly, abgeordnetenhaus;
            52.508, 13.378, Both;
            52.508, 13.377, DownstreamOnly;
            52.509, 13.377, Both,           potsdamer_platz_bus_stresemannstr;
            52.510, 13.377, Both,           potsdamer_platz_vossstr;
            52.511, 13.377, Both;
            52.512, 13.377, Both;
            52.512, 13.376, Both;
            52.512, 13.374, Both;
            52.511, 13.374, DownstreamOnly;
            52.511, 13.372, Both;
            52.511, 13.371, Both;
            52.512, 13.371, Both;
            52.513, 13.371, Both;
            52.514, 13.371, UpstreamOnly;
            52.514, 13.370, DownstreamOnly;
            52.516, 13.371, Both;
            52.518, 13.372, UpstreamOnly;
            52.519, 13.372, UpstreamOnly;
            52.520, 13.373, UpstreamOnly;
            52.521, 13.373, UpstreamOnly;
            52.518, 13.371, DownstreamOnly;
            52.520, 13.372, DownstreamOnly;
            52.521, 13.372, Both;
            52.5257,13.368, UpstreamOnly,   hauptbahnhof;
            52.526, 13.368, UpstreamOnly,   hauptbahnhof;
            52.527, 13.368, UpstreamOnly;
            52.528, 13.368, UpstreamOnly;
            52.522, 13.372, DownstreamOnly;
            52.5262,13.368, DownstreamOnly, hauptbahnhof;
            52.526, 13.369, DownstreamOnly, hauptbahnhof;
            52.527, 13.369, Both;
        bus_114:
            52.422, 13.178, UpstreamOnly, wannsee;
            52.421, 13.178, UpstreamOnly, wannsee;
            52.421, 13.177, UpstreamOnly;
            52.421, 13.176, UpstreamOnly;
            52.420, 13.175, UpstreamOnly, wannseebruecke;
            52.420, 13.174, UpstreamOnly;
            52.421, 13.174, UpstreamOnly;
            52.421, 13.173, UpstreamOnly;
            52.421, 13.172, UpstreamOnly;
            52.421, 13.171, UpstreamOnly;
            52.421, 13.170, UpstreamOnly;
            52.421, 13.169, UpstreamOnly;
            52.421, 13.168, UpstreamOnly;
            52.421, 13.167, UpstreamOnly, am_kleinen_wannsee;
            52.421, 13.166, UpstreamOnly;
            52.421, 13.165, UpstreamOnly;
            52.422, 13.165, UpstreamOnly;
            52.422, 13.164, UpstreamOnly;
            52.423, 13.163, UpstreamOnly;
            52.423, 13.162, UpstreamOnly;
            52.424, 13.162, UpstreamOnly, seglerweg;
            52.425, 13.161, UpstreamOnly;
            52.426, 13.161, UpstreamOnly;
            52.427, 13.162, UpstreamOnly, koblanckstr;
            52.428, 13.162, UpstreamOnly;
            52.428, 13.163, UpstreamOnly;
            52.429, 13.164, UpstreamOnly, liebermann_villa;
            52.430, 13.164, UpstreamOnly;
            52.430, 13.165, UpstreamOnly;
            52.431, 13.165, UpstreamOnly;
            52.432, 13.165, UpstreamOnly, am_grossen_wannsee;
            52.433, 13.164, UpstreamOnly, haus_der_wannsee_konferenz;
            52.432, 13.163, UpstreamOnly;
            52.432, 13.162, UpstreamOnly;
            52.431, 13.162, UpstreamOnly;
            52.431, 13.161, UpstreamOnly;
            52.430, 13.161, UpstreamOnly, zum_heckeshorn;
            52.429, 13.160, UpstreamOnly;
            52.428, 13.160, UpstreamOnly;
            52.427, 13.160, UpstreamOnly, strasse_zum_loewen;
            52.427, 13.159, UpstreamOnly;
            52.426, 13.160, UpstreamOnly;
            52.424, 13.160, UpstreamOnly, seglerweg;
            52.421, 13.162, UpstreamOnly;
            52.420, 13.162, UpstreamOnly, conradstr;
            52.420, 13.166, UpstreamOnly;
            52.420, 13.167, UpstreamOnly, am_kleinen_wannsee;
            52.421, 13.168, UpstreamOnly;
            52.421, 13.170, UpstreamOnly;
            52.421, 13.171, UpstreamOnly;
            52.421, 13.172, UpstreamOnly;
            52.421, 13.173, UpstreamOnly;
            52.420, 13.174, UpstreamOnly;
            52.420, 13.175, UpstreamOnly, wannseebruecke;
            52.420, 13.176, UpstreamOnly;
            52.421, 13.176, UpstreamOnly;
            52.421, 13.177, UpstreamOnly;
            52.421, 13.178, UpstreamOnly, wannsee;
            52.422, 13.179, UpstreamOnly, wannsee;
            52.422, 13.180, UpstreamOnly;
            52.422, 13.179, UpstreamOnly;
            52.422, 13.178, UpstreamOnly;
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::coord::project;
    use crate::trip::fixtures::*;

    #[test]
    fn test_getters() {
        let node = nodes::tram_12(Directions::Both).remove(4);
        assert_eq!(node.position(), project(52.520, 13.388));
        assert_eq!(node.location(), Some(&Rc::new(locations::friedrichstr())));
        assert_eq!(node.in_directions(), Directions::Both);
    }

    #[test]
    fn test_distance_to() {
        let node = nodes::tram_12(Directions::Both).remove(4);
        assert_relative_eq!(
            node.distance_to(&locations::friedrichstr()).into_inner(),
            67.86,
            epsilon = 0.01
        );
    }

    #[test]
    fn test_make_stop() {
        let mut node = Node::new(project(52.520, 13.388), Directions::Both);
        node.make_stop(Rc::new(locations::friedrichstr()));
        assert_eq!(node, nodes::tram_12(Directions::Both)[4]);
    }

    #[test]
    fn test_merge() {
        let mut upstream_node = nodes::tram_12(Directions::UpstreamOnly).remove(3);
        let downstream_node = nodes::tram_12(Directions::DownstreamOnly).remove(4);
        assert!(upstream_node.can_be_merged(&downstream_node));
        upstream_node.merge(downstream_node);
        assert_eq!(upstream_node, nodes::tram_12(Directions::Both)[4])
    }

    #[test]
    fn test_can_not_be_merged() {
        let upstream_node = nodes::tram_12(Directions::UpstreamOnly).remove(4);
        let mut downstream_node = Node::new(project(52.520, 13.388), Directions::DownstreamOnly);
        assert!(!upstream_node.can_be_merged(&downstream_node));
        downstream_node.make_stop(Rc::new(locations::oranienburger_tor()));
        assert!(!upstream_node.can_be_merged(&downstream_node));
    }

    #[test]
    fn test_store() {
        let mut linearizer = Linearizer::new();
        assert_eq!(
            nodes::tram_12(Directions::Both)
                .into_iter()
                .map(|node| node.store(&mut linearizer))
                .collect::<Vec<_>>(),
            storage::fixtures::nodes::tram_12(&linearizer.location_ids())
        );
    }
}
